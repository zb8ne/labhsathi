# ADR 0001: Event-driven document pipeline

## Status

Accepted.

## Context

The earlier prototype (`scheme-setu`) called Claude's vision API synchronously, in the same HTTP request handler that received the upload. Three problems with that, in order of how much they actually matter:

1. **It blocks a request thread on an external network call.** The vision API call is the slowest thing this system does by a wide margin. A synchronous design ties up one of the server's request-handling resources for the full duration of that call, for every upload, all the time.
2. **It can't scale the OCR path independently of the rest of the system.** Eligibility matching (`/api/match`) is pure, fast, in-memory Rust. Document extraction is slow and I/O-bound. Bolting them into the same process means the only way to handle more OCR load is to run more copies of the *whole* api-gateway, including the parts that didn't need scaling.
3. **It has no architectural boundary against accidentally persisting image bytes.** Nothing stops a future change (a debug log line, a "just cache it for a second" shortcut, a retry-with-backoff that keeps the buffer around) from quietly making the image outlive the request. The only thing enforcing "we never store your document" was code discipline.

## Decision

Split into three services connected by Kafka, with Redis as a short-lived handoff cache between them:

- **api-gateway** accepts the upload, writes the raw bytes to Redis under a generated `job_id` with a 60-second TTL, publishes `{job_id, mime_type, ts}` to `document.jobs.submitted`, and returns immediately. It never calls the vision API itself.
- **ocr-worker** consumes `document.jobs.submitted`, fetches the image out of Redis by `job_id` (using `GETDEL`, which removes the key in the same atomic operation that reads it), downsamples it, calls the vision API once, and publishes `{job_id, status, fields, ts}` to `document.jobs.completed`. It is stateless and horizontally scalable; this is the service the HPA scales.
- **Redis is a handoff cache, not a datastore.** This is enforced by more than a sentence in a doc: the raw-image key and the job-status key are *separate keys with separate TTLs* (60s for the image, 300s for a small JSON status blob), `GETDEL` means the image key cannot outlive its one legitimate read, and the local Docker Compose setup runs Redis with `--save ""`, so persistence is off even in dev and there's no snapshot file anywhere that could contain a document. If a future feature ever needed a result to survive longer than a few minutes, that's a sign the design has drifted and needs its own decision, not something to fix by quietly raising a TTL.
- **The privacy boundary is enforced at the event-schema type level**, in `labhsathi_core::events`: `DocumentJobSubmitted` and `DocumentJobCompleted` have no field capable of holding image bytes: no `Vec<u8>`, no `serde_json::Value`, no unbounded blob of any kind. The free-text extracted fields (`state`, `category`, `occupation`) use a length-capped `BoundedString` (≤64 bytes) rather than a bare `String`.

  **Honest limit on this claim:** a `BoundedString` is not *literally* incapable of holding something other than what it says: nothing at the type level distinguishes "the word Goa" from "64 bytes of something else." What the type system actually guarantees is narrower and still real: there is no field anywhere in these structs sized or typed for arbitrary binary data, and the code path that populates them only ever writes short strings the vision API's own structured-output schema produced. The bound makes misuse pointless (64 bytes can't usefully carry a document image) rather than provably impossible. We're stating this plainly instead of overselling "impossible."

## Rejected alternatives

- **Keep the synchronous in-process call** (what the prototype did). Rejected for the three reasons in Context: it was the status quo being moved away from, not a real contender.
- **A background thread / in-process channel instead of Kafka.** Would solve the blocking-request-thread problem but not the independent-scaling problem; the OCR work would still be tied to the same process and the same deploy unit as api-gateway. Doesn't produce a second, horizontally-scalable service; rejected.

## Consequences

- **Infra cost vs. differentiator value.** Running Kafka and Redis alongside two Rust services is real operational weight for a hackathon prototype. It's accepted here because the event-driven shape is the actual technical differentiator being demonstrated, not incidental architecture; a synchronous version of this product would be simpler to run and would also make the "we never store your document" claim weaker (asserted, not structurally enforced).
- **HPA on CPU is an imperfect proxy for ocr-worker's real bottleneck.** The vision API call is I/O-bound; only image decode/resize burns meaningful CPU. CPU-based autoscaling is the hackathon-scope default because it needs nothing beyond what Kubernetes ships with. The documented production upgrade is KEDA, scaling on Kafka consumer lag directly: the actual signal that matters (how many jobs are waiting, not how busy the CPU is).
- **A failed Kafka publish after a successful Redis write needs a compensating action.** api-gateway deletes the now-orphaned Redis key and returns 503 rather than leaving an image sitting in the cache with no worker ever told to look for it.
- **A crash mid-extraction must not lose or duplicate work silently.** ocr-worker only marks a Kafka offset as ready to commit *after* it has published a terminal `document.jobs.completed` event (success or failure) for that message. A crash before that point means the message redelivers to another consumer in the group. A redelivery that finds the Redis image key already gone (because the first attempt's `GETDEL` already consumed it) is treated as a normal, named failure case, not corruption, and still produces a terminal event, so no job is ever silently dropped.
