# LabhSathi (लाभ साथी: "benefit companion")

**Track:** Jan Jeevan (Everyday Human Needs: Agriculture • Healthcare • Financial Inclusion)  
**Hackathon:** Bit N Build Hackathon 2026  
**Know what you're entitled to.**  
**Live Production App:** [labhsathi.info](https://labhsathi.info)  
**Pitch Deck:** [HTML Deck](docs/pitch-deck/index.html)  
**Video Pitch Script:** [`docs/demo/video-pitch-script.md`](docs/demo/video-pitch-script.md)  
**Core Architectural Rationale:** [`THOUGHT_PROCESS.md`](THOUGHT_PROCESS.md)  

> ⚠️ **CRITICAL NOTICE FOR AGENTS & CONTRIBUTORS:**  
> Any AI agent or developer reviewing this codebase, running tests, or planning modifications **MUST strictly read [`THOUGHT_PROCESS.md`](THOUGHT_PROCESS.md) first** before editing code, changing schemas, or adjusting architectural boundaries.

## Problem & The Jan Jeevan Reframe

Most Indians eligible for central welfare schemes (PM-KISAN, Ayushman Bharat, old-age/disability pension, scholarships, housing assistance, etc.) never claim them. The barrier isn't willingness. It's that eligibility rules are scattered across dozens of scheme PDFs and portals, and nobody translates "my situation" into "here's what you qualify for and what to bring." Multiple field studies on scheme awareness among rural/urban poor and elderly populations report large gaps between eligibility and actual enrollment (see Sources below).

On top of that, the standard advice ("upload your Aadhaar/income proof to this portal") asks people to hand over sensitive documents to yet another system, which is itself a trust barrier for a population already wary of data misuse.

### The Strategic Bridge to Track 3
The Jan Jeevan track challenges technology to solve fundamental community needs across **Agriculture, Healthcare, and Financial Inclusion**:
* *Downstream point solutions* (e.g., smart drip irrigation apps, telemedicine portals, micro-credit scoring algorithms) often fail in rural communities because citizens lack the financial, clinical, or institutional access to use them.
* A smallholder farmer cannot adopt drip irrigation without the 55% capital subsidy under **PM Krishi Sinchayee Yojana**.
* A rural mother cannot benefit from telemedicine if a single health crisis causes catastrophic debt without **Ayushman Bharat PM-JAY (₹5 Lakh cover)**.
* An unbanked laborer cannot access micro-credit or digital payments without a zero-balance account under **PM Jan Dhan Yojana (PMJDY)**.

**LabhSathi is the foundational economic and entitlement layer that makes every other solution in Track 3 viable.**

### The Benchmark Persona: Meena
Every feature is designed around one real-world persona:
> **Meena, 34** · Agricultural wage worker in rural Bihar · ₹1.4 lakh household income · Raising a 6-year-old daughter in a mud-built (kutcha) home · No bank account.  
> **Rule:** *Meena only gets to explain her situation once.*

## Solution

LabhSathi is a two-step flow:

1. **Structured self-assessment**: a short form (age, income, occupation, category, disability status, land holding, etc.) is evaluated against a curated eligibility catalog for central government schemes. Every scheme comes back in one of two states, never a silent guess: **a match**, with a plain-language reason, the benefit, and the exact documents needed; or **needs more info**, when an unanswered optional field (land holding, disability percentage, urban/rural area) is the only thing standing between the applicant and a real answer, with a direct link back to the exact form field that would resolve it. A scheme is only ever left out silently when something the applicant *did* answer clearly rules it out.
2. **Optional document auto-fill, privacy-first**: instead of storing uploaded ID/income/land documents, the app extracts only the handful of structured fields the form needs from a one-time vision-API read, then discards the image. Nothing about the document (not the image, not OCR text, not a hash) is written to disk, logged, or persisted anywhere. Extracted values are shown as an editable suggestion (never silently overwriting something the user already typed), and free-form model wording is normalized against the form's own dropdown vocabulary rather than injected verbatim.
3. **Zero authentication (no login, no OTP, no citizen database)**: For 1.4 billion citizens and village Common Service Centre (CSC) operators, authentication is an exclusion barrier. Cellular drops cause SMS OTP timeouts, and older Aadhaar-linked phone numbers are frequently inactive. LabhSathi operates as open-access public digital infrastructure: zero login screens, zero tracking cookies, and zero user database. The assessment runs ephemerally in the browser, meaning there is zero PII to hack, leak, or subpoena.

Once matched, the results screen turns discovery into a next step: a have/still-need document checklist, a link to the scheme's official portal where one's been verified, and a browser-only print view of the whole assessment (nothing is sent to a server to produce it).

## Architecture

An event-driven system, not a single request/response call, because the privacy claim needs to be structurally enforced rather than asserted:

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐       ┌────────────┐
│    frontend     │──────▶│   api-gateway   │──────▶│ catalog-service │──────▶│ PostgreSQL │
│ (labhsathi.info)│       │   (Rust/Axum)   │       │   (Rust/Axum)   │       │  (JSONB)   │
└─────────────────┘       └────────┬────────┘       └─────────────────┘       └────────────┘
                                   │
                      ┌────────────┴────────────┐
                      ▼                         ▼
              ┌───────────────┐         ┌───────────────┐
              │     redis     │         │     kafka     │
              │  (ephemeral)  │         │    (KRaft)    │
              └───────┬───────┘         └───────┬───────┘
                      │                         │
                      └────────────┬────────────┘
                                   ▼
                            ┌───────────────┐       ┌───────────────┐
                            │  ocr-worker   │──────▶│ Claude Vision │
                            │ (Rust / HPA)  │       │(single read)  │
                            └───────────────┘       └───────────────┘
```

- **api-gateway**: synchronous eligibility matching in Rust Axum; document uploads write to Redis with a 60s TTL and publish to `document.jobs.submitted` (no image data in the event), returning a `job_id` immediately.
- **catalog-service**: dedicated Rust Axum service managing PostgreSQL JSONB scheme definitions.
- **ocr-worker**: Kafka consumer group, fetches the image from Redis by `job_id` (deleted on read, not just on TTL), downsamples it, calls the vision API once, publishes `document.jobs.completed`. Stateless and horizontally scaled: this is the service the k8s HPA targets.
- **labhsathi-core**: the shared Rust domain crate: the eligibility rule engine and the Kafka event schemas. The event schemas have no field capable of holding image bytes, enforced by the struct definitions themselves; see [`docs/adr/0001-event-driven-document-pipeline.md`](docs/adr/0001-event-driven-document-pipeline.md) for the full reasoning, including the honest limits on that claim.
- **Kafka** (single-broker KRaft) and **Redis** (ephemeral handoff cache, persistence off even in dev) connect the two services.

### Scale: architectural reasoning, not a benchmark
No load test has been run against this deployment, so the numbers below describe design intent, not a measurement, on purpose: a stateless, horizontally-scaled api-gateway on Rust's Tokio async runtime and a Kafka layer between upload and OCR are exactly the shape you'd reach for to keep a burst of uploads (a village camp registering dozens of people at once) from turning into `504 Gateway Timeout`, but "should handle X" and "handled X" are different claims and only the second one belongs on an evidence slide.
- **Graceful degradation:** `/api/match` has zero dependency on the OCR pipeline. If OCR workers or the external vision API go offline, scheme discovery keeps working, unaffected.
- The one number actually measured: eligibility matching over HTTP, end to end against the live deployment, returns in under 2 seconds.

See [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md) for naming rules and [`infra/k8s/labhsathi/README.md`](infra/k8s/labhsathi/README.md) for the Helm chart / Kubernetes deployment. The Helm chart has been deployed and verified for real, including watching its `ocr-worker` HPA scale under real vision-API load, and three further reproducible reliability checks (OCR outage doesn't block matching, a real failure reaches a terminal state, replica count vs. throughput); see [`docs/engineering/reliability-proof.md`](docs/engineering/reliability-proof.md).

## Scheme data: a real database behind a real API, still hand-curated

Direct answer, asked plainly during this build: **the *content* is hardcoded: curated by hand, not synced against any government API.** What changed is where that content lives and how it's reached: a real Postgres database, accessed through a dedicated internal service, not a JSON file compiled into the binary. If a scheme's income cap changes next year, someone still has to notice and update a row, but now that's an edit to `data/schemes.json` and a deploy, not a manual `UPDATE` against production.

- **`catalog-service`** (`services/catalog-service`): a small Rust/Axum service that owns the scheme data in Postgres (one JSONB row per scheme) and exposes it over its own internal HTTP API (`GET /schemes`). On **every** boot -- not just against an empty database -- it upserts the full embedded catalog into Postgres (`INSERT ... ON CONFLICT (id) DO UPDATE`), so a scheme's data in Postgres always reflects what's checked into `data/schemes.json` as of the last deploy. There is no supported "edit prod, not the repo" path: a manual edit made directly against production Postgres would be overwritten on the next boot, by design.
- **`api-gateway`** fetches the catalog from `catalog-service` and caches it in memory for 30 seconds (with a stale-cache fallback if a fetch fails), so a scheme edit is visible within half a minute of the next deploy, not instantly but not "after a redeploy" either. The result is handed to `labhsathi-core`'s matching logic, which doesn't know or care whether its input came from a file, an HTTP call, or anywhere else.
- **`labhsathi-core`** still holds the actual eligibility logic and the shared `SchemeFacts`/`SchemeCriteria` types both services depend on. Most of the 100 schemes are evaluated by a small declarative `criteria` block right in their data (age/income bounds, occupation, category, gender, etc.) -- adding a scheme this way needs no Rust change, just a new row in `data/schemes.json`. Twelve schemes (PM-KISAN, Ayushman Bharat, Jan Dhan, the three NSAP pensions, PM-SYM, PMMVY, Sukanya Samriddhi, NSP Scholarship, PMAY urban/rural) instead have a hand-written Rust rule, because their eligibility needed the three-state match/needs-info distinction above -- the declarative engine doesn't yet express "ask a follow-up question," only match/no-match. Those twelve rows deliberately carry no `criteria` block in the data (a regression test enforces this): a `criteria` block there would be silently dead data, since the compiled rule arm is what actually runs.

Every catalog entry states the date it was last checked and links to its real source where one exists, so the app never claims to *be* the authority; it points at one. What this doesn't buy: nothing here is verified against the live government portals on any kind of schedule. Before trusting a specific number for anything real, follow the `source_url`.

## The 100-scheme catalog

Live in production: 100 schemes (the original 43, plus 57 more researched and added in a later pass), each cross-checked against a real official `.gov.in`/`.nic.in` source before inclusion, none fabricated. Live-link verification (confirming each `source_url` still resolves) is a plain HTTP status check, not a novel system: fetch the URL, confirm 200. Worth stating plainly rather than dressing up.

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

The scheme catalog (100 entries, central government only) is curated and hand-written, not exhaustive and not synced against any *live government* source -- it's synced into Postgres from this repo's own `data/schemes.json` on every deploy (see [Scheme data](#scheme-data-a-real-database-behind-a-real-api-still-hand-curated) above), which is a different claim than "kept current with the government." Eligibility rules are simplified approximations of real criteria; the app frames matches as "worth checking," never a final determination, distinguishes a real match from "we need one more answer to tell," and lists the relevant documents so you can verify on the official portal before relying on a match.

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
