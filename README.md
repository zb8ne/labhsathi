# LabhSathi (लाभ साथी — "benefit companion")

**Track:** Jan Jeevan (Bit N Build Hackathon 2026)

**Know what you're entitled to.**

## Problem

Most Indians eligible for central welfare schemes (PM-KISAN, Ayushman Bharat, old-age/disability pension, scholarships, housing assistance, etc.) never claim them. The barrier isn't willingness — it's that eligibility rules are scattered across dozens of scheme PDFs and portals, and nobody translates "my situation" into "here's what you qualify for and what to bring." Multiple field studies on scheme awareness among rural/urban poor and elderly populations report large gaps between eligibility and actual enrollment (see Sources below).

On top of that, the standard advice ("upload your Aadhaar/income proof to this portal") asks people to hand over sensitive documents to yet another system, which is itself a trust barrier for a population already wary of data misuse.

## Solution

LabhSathi is a two-step flow:

1. **Structured self-assessment** — a short form (age, income, occupation, category, disability status, land holding, etc.) is evaluated against a curated eligibility-rules table for central government schemes. Matches come back with a plain-language reason, the benefit, and the exact documents needed.
2. **Optional document auto-fill, privacy-first** — instead of storing uploaded ID/income/land documents, the app extracts only the handful of structured fields the form needs from a one-time vision-API read, then discards the image. Nothing about the document — not the image, not OCR text, not a hash — is written to disk, logged, or persisted anywhere.

## Architecture

An event-driven system, not a single request/response call, because the privacy claim needs to be structurally enforced rather than asserted:

```
frontend (React) ──▶ api-gateway (Rust/Axum) ──▶ Kafka: document.jobs.submitted ──▶ ocr-worker (Rust)
                            │                                                              │
                            └── Redis (60s image handoff, 300s status) ◀────────────────────┘
                                                                    │
                                                       Kafka: document.jobs.completed
```

- **api-gateway** — synchronous eligibility matching; document uploads write to Redis with a 60s TTL and publish to `document.jobs.submitted` (no image data in the event), returning a `job_id` immediately.
- **ocr-worker** — Kafka consumer group, fetches the image from Redis by `job_id` (deleted on read, not just on TTL), downsamples it, calls the vision API once, publishes `document.jobs.completed`. Stateless and horizontally scaled — this is the service the k8s HPA targets.
- **labhsathi-core** — the shared Rust domain crate: the eligibility rule engine and the Kafka event schemas. The event schemas have no field capable of holding image bytes, enforced by the struct definitions themselves — see [`docs/adr/0001-event-driven-document-pipeline.md`](docs/adr/0001-event-driven-document-pipeline.md) for the full reasoning, including the honest limits on that claim.
- **Kafka** (single-broker KRaft) and **Redis** (ephemeral handoff cache, persistence off even in dev) connect the two services.

See [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md) for naming rules and [`infra/k8s/labhsathi/README.md`](infra/k8s/labhsathi/README.md) for the Helm chart / Kubernetes deployment.

## Tech stack

- **Backend:** Rust (Axum, Tokio, rdkafka, redis-rs)
- **Frontend:** React + TypeScript + Tailwind (Vite)
- **Event backbone:** Apache Kafka (KRaft mode)
- **Cache:** Redis (ephemeral handoff only — see the ADR for why this distinction is load-bearing, not cosmetic)
- **Vision/OCR:** Anthropic Claude API (vision), called once per document, never persisted
- **Local dev:** Docker Compose (`infra/docker/docker-compose.yml`)
- **Deployment:** Kubernetes via Helm (`infra/k8s/labhsathi`)

## Running it

**Local dev (Docker Compose):**

```bash
export ANTHROPIC_API_KEY=sk-ant-...   # only needed for document auto-fill; matching works without it
cd infra/docker
docker compose up --build
```

Frontend: http://localhost:5173 · api-gateway: http://localhost:8080

## Testing

```bash
cargo test --workspace          # rule engine, event schemas, media validation, kafka offset ordering
cd frontend && npm test         # auto-fill's no-overwrite behavior
```

`cargo test --workspace` skips three Redis integration tests (`services/api-gateway/src/redis_cache.rs`) that need a live Redis -- Compose's own `redis` service isn't published to the host by default, so point one at localhost first:

```bash
docker run --rm -d --name redis-test -p 6379:6379 redis:7-alpine
cargo test -p api-gateway -- --ignored   # REDIS_URL defaults to redis://localhost:6379
docker stop redis-test
```

**Without Docker**, each service can run standalone against a local Kafka + Redis:

```bash
cargo run -p api-gateway   # KAFKA_BROKERS / REDIS_URL / PORT env vars, see .env.example
cargo run -p ocr-worker    # + ANTHROPIC_API_KEY
cd frontend && npm install && npm run dev
```

## Scope & honesty note

The scheme database here is a demonstration set, not exhaustive — a production version would ingest the full central + state catalog and keep it current against official notifications. Eligibility rules are simplified approximations of real criteria; the app frames matches as "worth checking," not a final determination, and lists the relevant documents so you can verify on the official portal before relying on a match.

The event-driven pipeline is real and running, not a diagram — but read [`docs/adr/0001`](docs/adr/0001-event-driven-document-pipeline.md) for exactly where the guarantees are solid (no field anywhere can hold image bytes) versus where they're honestly bounded (a free-text field is length-capped, not literally typed-impossible to misuse).

## Sources used while building this

- [Registered, But Not Reached — India Forum](https://www.theindiaforum.in/forum/registered-not-reached-why-indias-welfare-schemes-informal-workers-fall-short)
- [Awareness and utilization of government schemes — IJCMPH](https://www.ijcmph.com/index.php/ijcmph/article/download/7591/4832)
- [Bridging the gap: accessibility and awareness of government schemes](https://www.ovid.com/jnls/jfmpc/fulltext/10.4103/jfmpc.jfmpc_295_24~bridging-the-gap-promoting-accessibility-and-awareness-of)
- [PM-KISAN eligibility criteria](https://www.angelone.in/knowledge-center/savings-schemes/eligibility-for-pm-kisan-samman-nidhi-yojana)
- [Documents required for PM-JAY](https://www.hexahealth.com/blog/documents-required-for-pmjay-scheme)

## Future scope

- Ingest the full central + state scheme catalog (data.gov.in has partial APIs)
- Multi-language form (most affected users are not English-first)
- SMS/WhatsApp front-end for low-smartphone-literacy users
- KEDA-based (consumer-lag) autoscaling for ocr-worker in production — see `infra/k8s/labhsathi/values-prod.yaml`
- 3-broker Kafka for production durability — see the ADR
