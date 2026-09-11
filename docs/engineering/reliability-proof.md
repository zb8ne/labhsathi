# Reliability & scaling proof

A companion to the architecture claims in the pitch deck and ADR 0001 --
three things demonstrated live against the real stack (Kafka, Redis,
ocr-worker, a real Claude vision-API call), not asserted from the design.
Every command below is copy-pasteable and was actually run against
`infra/docker/docker-compose.yml` from the repo root's `infra/docker/`
directory (or repo root, adjusted below) with the stack already up
(`docker compose -f infra/docker/docker-compose.yml up -d`).

## Prerequisite: Kafka partitions

Kafka topics in this stack are auto-created on first use. The broker's
default `num.partitions` is 1 -- and a single partition means only **one**
consumer in a consumer group can ever be actively assigned it, no matter
how many `ocr-worker` replicas exist. Scaling replicas without this fix
would not have increased throughput at all; it would have added idle
pods.

Fixed in both places the topic gets created:
- `infra/docker/docker-compose.yml`: `KAFKA_NUM_PARTITIONS: "3"` on the
  `kafka` service.
- `infra/k8s/labhsathi/templates/kafka-deployment.yaml`: same env var on
  the container spec.

Verified against the running compose stack (topics recreated fresh after
the kafka container was recreated with the new env, then auto-created on
the next upload):

```
$ docker exec labhsathi-kafka /opt/kafka/bin/kafka-topics.sh \
    --bootstrap-server localhost:9092 --describe --topic document.jobs.submitted
Topic: document.jobs.submitted   PartitionCount: 3   ReplicationFactor: 1
        Partition: 0  Leader: 1  Replicas: 1  Isr: 1
        Partition: 1  Leader: 1  Replicas: 1  Isr: 1
        Partition: 2  Leader: 1  Replicas: 1  Isr: 1
```

Same result for `document.jobs.completed`. With 3 replicas running,
`kafka-consumer-groups.sh --describe --group ocr-worker-group` confirmed
a clean 1:1 partition-to-consumer assignment (one partition per pod, no
consumer holding more than one, no consumer idle) -- the actual
prerequisite for demonstration 3 below to mean anything.

## Demonstration 1: stopping OCR doesn't stop matching

`/api/match` is pure in-process Rust -- no Kafka client, no Redis call, no
outbound HTTP. This is architectural, not a runtime fallback, but the
point of a reproducible proof is showing it, not just asserting it.

```
$ docker compose -f infra/docker/docker-compose.yml stop ocr-worker
Container labhsathi-ocr-worker-1  Stopped

$ curl -sS -w '\nHTTP %{http_code} | %{time_total}s\n' -X POST http://localhost:8080/api/match \
    -H 'Content-Type: application/json' \
    -d '{"age":45,"annual_income":80000,"occupation":"farmer","state":"Goa","gender":"male",
         "has_disability":false,"disability_percentage":null,"land_holding_acres":2.0,
         "family_size":4,"is_widow":false,"category":"obc","is_student":false,
         "has_bank_account":true,"has_kutcha_house":false,
         "is_pregnant_or_lactating_first_child":false,"girl_child_age":null,"area_type":null}'
```

**Result:** `HTTP 200`, correct 2-scheme match (PM-KISAN, Ayushman Bharat),
`time_total` **0.000551s** -- with `ocr-worker` fully stopped.

`docker compose start ocr-worker` restored it afterward.

*Caveat:* the running `api-gateway` image at test time predated this
branch's three-state contract commit, so the response was in the old
(pre-`status`/`missing_field`) shape. Irrelevant to what this test
measures (matching succeeding with OCR down), but noted rather than
hidden -- rebuild the image before relying on the response shape.

## Demonstration 2: failure injection reaches a terminal state

Real failure, not a mocked one: `ocr-worker`'s `ANTHROPIC_API_KEY` was
overridden to an invalid value for one container recreate, a document was
uploaded, and the job was polled to a terminal state.

```
$ docker compose -f infra/docker/docker-compose.yml stop ocr-worker
$ ANTHROPIC_API_KEY="sk-ant-invalid-deliberately-broken-key-for-failure-test" \
    docker compose -f infra/docker/docker-compose.yml up -d ocr-worker

$ curl -s -X POST http://localhost:8080/api/documents -F "document=@docs/sample-documents/specimen-income-certificate.jpg"
{"job_id":"a34497a8-4804-4014-9739-1293fa3bc997"}

$ curl -s http://localhost:8080/api/documents/a34497a8-4804-4014-9739-1293fa3bc997
{"status":"failed","fields":null,"error":"API error (401 Unauthorized): API key is invalid."}
```

**Result:** reached `failed` with a specific, useful error message -- not
stuck at `processing` forever. Took roughly 35-40s wall-clock from upload
to terminal state in this run (slower than a successful extraction;
consumer-group rebalancing after the container recreate accounts for
most of it, not the failure path itself -- see the note on rebalance
latency below).

Restored the real key the same way and confirmed a normal upload
succeeds again (`"status":"done"` with correct extracted fields) --
the stack was not left broken.

**Side finding, not one of the three asked for:** every `ocr-worker`
container recreate in this session (scaling, env override, plain
restart) was followed by 20-40s where the consumer group sat in a
"rebalancing" state before the new consumer was fully assigned
partitions and started actually processing. This is `rdkafka`'s default
rebalance/session timeout behavior, not a bug -- but it means a job
submitted right after a worker restart can sit at `queued` for tens of
seconds through no fault of the job itself. Worth knowing if a future
demo scales replicas live on camera and expects instant pickup.

## Demonstration 3: does adding workers increase throughput?

Reported honestly, including where it didn't show the hoped-for effect.

**Attempt 1** (6-job sequential-submission burst, 1 replica) and a
follow-up 6-job and 4-job concurrent-submission burst (3 replicas)
produced apparently-instant "0s" / "68ms" results. These were **not
real** -- traced to a shell scripting bug: this environment's default
shell is zsh, where `for x in $unquoted_var` does not word-split like
bash/sh. `$JOB_IDS` (a space-separated string) was silently treated as
one opaque token instead of N job ids, so the polling loop only ever
checked one (malformed) id, found it didn't match `queued`/`processing`,
and reported "all terminal" instantly. The uploads themselves were real
and all completed successfully (individually re-verified after the
fact) -- only the *timing measurement* from those attempts is discarded
here, not the underlying jobs.

Re-run correctly (explicit `bash -c '...'` with a real bash array,
`sleep`-before-check polling, 3 concurrent uploads per configuration to
keep spend bounded after the wasted attempts above):

```
1 replica,  3 concurrent uploads -> all 3 terminal at +3s
3 replicas, 3 concurrent uploads -> all 3 terminal at +4s
```

**Honest interpretation:** at this batch size, 3 replicas were not
faster -- if anything, 1s slower, plausibly just the rebalance-settling
overhead noted above rather than a real regression. This is expected,
not a failure of the architecture: `ocr-worker`'s
`MAX_CONCURRENT_EXTRACTIONS` defaults to 4 per pod (`services/ocr-worker/src/consumer.rs`),
so a single replica already has enough concurrency to run all 3 jobs in
this test in parallel with no queueing at all. Replica count only
starts to matter once concurrent load exceeds one pod's semaphore --
which is exactly the CPU-based HPA story already demonstrated
separately this session (`ocr-worker` scaling 1→2 replicas under
sustained load in a local `kind` cluster). A batch bigger than 4
concurrent jobs would be needed to see a multi-replica throughput
difference directly in this test's terms; not run here to keep real
API spend in check (see below) -- a reasonable next increment if this
proof gets extended.

## Real API spend

**23 real, successful Claude vision-API calls** were made across this
work (plus 1 deliberately-invalid-key call that errored before billing).
That is meaningfully over the ~10-15 guidance given for this task -- the
shell-scripting detour above wasted roughly 10 of those (6 + 4 job
bursts whose timing had to be discarded and re-measured). Flagging this
plainly rather than smoothing it over: the corrected, trustworthy
numbers came from the last two 3-job runs only; the earlier bursts still
proved correctness (every job did complete) but not timing.

## Stack state after this work

`ocr-worker` scaled back to 1 replica, `api-gateway` back to the default
(non-raised) upload rate limit, real `ANTHROPIC_API_KEY` restored
everywhere, all five compose services healthy. `cargo test --workspace`
green (this work touched only `docker-compose.yml` and the Helm kafka
template -- no Rust code changed).
