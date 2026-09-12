# LabhSathi (लाभ साथी: "benefit companion")

**Track:** Jan Jeevan (Bit N Build Hackathon 2026)

**Know what you're entitled to.**

**Live app:** https://labhsathi.info · English + हिन्दी, light/dark theme.

## Problem

Most Indians eligible for central welfare schemes (PM-KISAN, Ayushman Bharat, old-age/disability pension, scholarships, housing assistance, etc.) never claim them. The barrier isn't willingness. It's that eligibility rules are scattered across dozens of scheme PDFs and portals, and nobody translates "my situation" into "here's what you qualify for and what to bring." Multiple field studies on scheme awareness among rural/urban poor and elderly populations report large gaps between eligibility and actual enrollment (see Sources below).

On top of that, the standard advice ("upload your Aadhaar/income proof to this portal") asks people to hand over sensitive documents to yet another system, which is itself a trust barrier for a population already wary of data misuse.

## Solution

LabhSathi is a two-step flow:

1. **Structured self-assessment**: a short form (age, income, occupation, category, disability status, land holding, etc.) is evaluated against a curated eligibility catalog for central government schemes. Every scheme comes back in one of two states, never a silent guess: **a match**, with a plain-language reason, the benefit, and the exact documents needed; or **needs more info**, when an unanswered optional field (land holding, disability percentage, urban/rural area) is the only thing standing between the applicant and a real answer, with a direct link back to the exact form field that would resolve it. A scheme is only ever left out silently when something the applicant *did* answer clearly rules it out.
2. **Optional document auto-fill, privacy-first**: instead of storing uploaded ID/income/land documents, the app extracts only the handful of structured fields the form needs from a one-time vision-API read, then discards the image. Nothing about the document (not the image, not OCR text, not a hash) is written to disk, logged, or persisted anywhere. Extracted values are shown as an editable suggestion (never silently overwriting something the user already typed), and free-form model wording is normalized against the form's own dropdown vocabulary rather than injected verbatim.

Once matched, the results screen turns discovery into a next step: a have/still-need document checklist, a link to the scheme's official portal where one's been verified, and a browser-only print view of the whole assessment (nothing is sent to a server to produce it).

## Architecture

An event-driven system, not a single request/response call, because the privacy claim needs to be structurally enforced rather than asserted:

```
frontend (React) ──▶ api-gateway (Rust/Axum) ──▶ Kafka: document.jobs.submitted ──▶ ocr-worker (Rust)
                            │                                                              │
                            └── Redis (60s image handoff, 300s status) ◀────────────────────┘
                                                                    │
                                                       Kafka: document.jobs.completed
```

- **api-gateway**: synchronous eligibility matching; document uploads write to Redis with a 60s TTL and publish to `document.jobs.submitted` (no image data in the event), returning a `job_id` immediately.
- **ocr-worker**: Kafka consumer group, fetches the image from Redis by `job_id` (deleted on read, not just on TTL), downsamples it, calls the vision API once, publishes `document.jobs.completed`. Stateless and horizontally scaled: this is the service the k8s HPA targets.
- **labhsathi-core**: the shared Rust domain crate: the eligibility rule engine and the Kafka event schemas. The event schemas have no field capable of holding image bytes, enforced by the struct definitions themselves; see [`docs/adr/0001-event-driven-document-pipeline.md`](docs/adr/0001-event-driven-document-pipeline.md) for the full reasoning, including the honest limits on that claim.
- **Kafka** (single-broker KRaft) and **Redis** (ephemeral handoff cache, persistence off even in dev) connect the two services.

See [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md) for naming rules and [`infra/k8s/labhsathi/README.md`](infra/k8s/labhsathi/README.md) for the Helm chart / Kubernetes deployment. The Helm chart has been deployed and verified for real, including watching its `ocr-worker` HPA scale under real vision-API load, and three further reproducible reliability checks (OCR outage doesn't block matching, a real failure reaches a terminal state, replica count vs. throughput); see [`docs/engineering/reliability-proof.md`](docs/engineering/reliability-proof.md).

## Scheme data: a real database behind a real API, still hand-curated

Direct answer, asked plainly during this build: **the *content* is hardcoded: curated by hand, not synced against any government API.** What changed is where that content lives and how it's reached: a real Postgres database, accessed through a dedicated internal service, not a JSON file compiled into the binary. If a scheme's income cap changes next year, someone still has to notice and update a row, but now that's an `UPDATE` statement against a running database, not a code change requiring a rebuild.

- **`catalog-service`** (`services/catalog-service`): a small Rust/Axum service that owns the scheme data in Postgres (one JSONB row per scheme) and exposes it over its own internal HTTP API (`GET /schemes`). On first boot against an empty database it seeds itself from the same reviewed dataset this repo always shipped.
- **`api-gateway`** calls `catalog-service` on every `/api/match` request (no caching: an edit in Postgres is live on the very next request) and hands the result to `labhsathi-core`'s matching logic, which doesn't know or care whether its input came from a file, an HTTP call, or anywhere else.
- **`labhsathi-core`** still holds the actual eligibility logic and the shared `SchemeFacts`/`SchemeCriteria` types both services depend on. Most of the 43 schemes are evaluated by a small declarative `criteria` block right in their data (age/income bounds, occupation, category, gender, etc.). Adding a new scheme this way needs no Rust change, just a new database row. A handful of the original schemes (PM-KISAN, the NSAP pensions, PMAY's urban/rural split) instead have a hand-written Rust rule, because their eligibility needed the three-state match/needs-info distinction above; the declarative engine doesn't yet express "ask a follow-up question," only match/no-match.

Every catalog entry states the date it was last checked and links to its real source where one exists, so the app never claims to *be* the authority; it points at one. What this doesn't buy: nothing here is verified against the live government portals on any kind of schedule. Before trusting a specific number for anything real, follow the `source_url`.

## Tech stack

- **Backend:** Rust (Axum, Tokio, rdkafka, redis-rs, sqlx)
- **Frontend:** React + TypeScript + Tailwind (Vite)
- **Event backbone:** Apache Kafka (KRaft mode)
- **Cache:** Redis (ephemeral handoff only: see the ADR for why this distinction is load-bearing, not cosmetic)
- **Scheme catalog:** PostgreSQL, behind `catalog-service`, the one datastore in this system meant to persist (unlike Redis)
- **Vision/OCR:** Anthropic Claude API (vision), called once per document, never persisted
- **Local dev:** Docker Compose (`infra/docker/docker-compose.yml`)
- **Deployment:** Railway (the always-on public link above) · Kubernetes via Helm (`infra/k8s/labhsathi`), verified against a local `kind` cluster with a real HPA scaling demo
- **Secrets:** Doppler in every environment: no plaintext secret in the repo, a Compose file, or a checked-in k8s manifest; the Helm chart references a Secret by name rather than templating one from values

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

`cargo test --workspace` skips integration tests that need a live database -- three Redis tests (`services/api-gateway/src/redis_cache.rs`) and two Postgres tests (`services/catalog-service/src/main.rs`). Compose's own `redis`/`postgres` services aren't published to the host by default, so point standalone ones at localhost first:

```bash
docker run --rm -d --name redis-test -p 6379:6379 redis:7-alpine
cargo test -p api-gateway -- --ignored   # REDIS_URL defaults to redis://localhost:6379
docker stop redis-test

docker run --rm -d --name postgres-test -p 5433:5432 \
  -e POSTGRES_DB=labhsathi_catalog -e POSTGRES_USER=labhsathi -e POSTGRES_PASSWORD=labhsathi_dev_only \
  postgres:16-alpine
DATABASE_URL=postgres://labhsathi:labhsathi_dev_only@localhost:5433/labhsathi_catalog \
  cargo test -p catalog-service -- --ignored --test-threads=1   # see the note in main.rs for why --test-threads=1
docker stop postgres-test
```

**Without Docker**, each service can run standalone against a local Kafka + Redis + Postgres:

```bash
cargo run -p catalog-service  # DATABASE_URL, see .env.example -- seeds itself on first boot
cargo run -p api-gateway   # KAFKA_BROKERS / REDIS_URL / CATALOG_SERVICE_URL / PORT env vars, see .env.example
cargo run -p ocr-worker    # + ANTHROPIC_API_KEY
cd frontend && npm install && npm run dev
```

## Scope & honesty note

The scheme catalog (43 entries, central government only) is curated and hardcoded, not exhaustive and not live-synced; see [Scheme data](#scheme-data-a-real-database-behind-a-real-api-still-hand-curated) above for exactly what that means and why. Eligibility rules are simplified approximations of real criteria; the app frames matches as "worth checking," never a final determination, distinguishes a real match from "we need one more answer to tell," and lists the relevant documents so you can verify on the official portal before relying on a match.

The event-driven pipeline is real and running, not a diagram, but read [`docs/adr/0001`](docs/adr/0001-event-driven-document-pipeline.md) for exactly where the guarantees are solid (no field anywhere can hold image bytes) versus where they're honestly bounded (a free-text field is length-capped, not literally typed-impossible to misuse).

## Demo & pitch deck

- [`docs/demo/walkthrough.mp4`](docs/demo/walkthrough.mp4): a real, unscripted capture of the golden path against the live stack (real Kafka/Redis/ocr-worker, a real vision-API call against a synthetic specimen document)
- [`docs/pitch-deck/index.html`](docs/pitch-deck/index.html): the 6-slide submission deck, open directly in a browser

## Sources used while building this

- [Registered, But Not Reached: India Forum](https://www.theindiaforum.in/forum/registered-not-reached-why-indias-welfare-schemes-informal-workers-fall-short)
- [Awareness and utilization of government schemes: IJCMPH](https://www.ijcmph.com/index.php/ijcmph/article/download/7591/4832)
- [Bridging the gap: accessibility and awareness of government schemes](https://www.ovid.com/jnls/jfmpc/fulltext/10.4103/jfmpc.jfmpc_295_24~bridging-the-gap-promoting-accessibility-and-awareness-of)
- [PM-KISAN eligibility criteria](https://www.angelone.in/knowledge-center/savings-schemes/eligibility-for-pm-kisan-samman-nidhi-yojana)
- [Documents required for PM-JAY](https://www.hexahealth.com/blog/documents-required-for-pmjay-scheme)

## Future scope

- A real process for keeping scheme data current against official notifications, instead of a one-time hardcoded pass: periodic re-verification of every `source_url` and `review_date` at minimum; ingesting the full central + state catalog (data.gov.in has partial APIs) is the further version of this
- Extend the three-state (match / needs-info) eligibility model to the declarative criteria engine, so schemes that only have JSON criteria today can also ask a follow-up question instead of only ever matching or not
- A real household-member model: today's form mixes an individual's age/occupation with household income and a daughter's age without ever asking whose circumstances a question describes; this is a deliberately separate increment, not a quick fix
- Hindi translation for the dynamically-generated per-scheme "why you matched" sentences (everything else in the UI is already fully bilingual)
- SMS/WhatsApp front-end for low-smartphone-literacy users
- KEDA-based (consumer-lag) autoscaling for ocr-worker in production; see `infra/k8s/labhsathi/values-prod.yaml`
- 3-broker Kafka for production durability; see the ADR
