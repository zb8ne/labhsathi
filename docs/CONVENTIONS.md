# Conventions

## Rust

- Crates: kebab-case (`api-gateway`, `ocr-worker`, `labhsathi-core`).
- Modules: snake_case.
- All domain logic (eligibility rules, event schemas, the job-status record, shared media validation) lives in `labhsathi-core` only. `api-gateway` and `ocr-worker` depend on it; it depends on neither of them, and has no `axum`/`reqwest` dependency of its own, to keep it usable from either an HTTP handler or a Kafka consumer without dragging in the other's stack.

## Kafka topics

Naming rule: `domain.entity.event`, past-tense.

The actual topics in this system are `document.jobs.submitted` and `document.jobs.completed` (defined as constants in `labhsathi-core::events`). An earlier ticket used `document.processing.completed` as an illustrative example of the naming rule. That string was never implemented and isn't the real topic name. Don't "fix" the real topics to match it.

Message key = the job's `job_id` (UUID, as a string); every event for one job lands on the same partition and stays ordered. Message value = JSON-encoded `labhsathi_core::events::{DocumentJobSubmitted, DocumentJobCompleted}`.

## Kubernetes

Namespace: `labhsathi`. Resource naming: `labhsathi-<service>-<kind>` (e.g. `labhsathi-api-gateway-deployment`, `labhsathi-ocr-worker-hpa`).

## Docker images

No registry is needed for this project's deploy targets: Railway builds from source per-service, and `kind` loads images directly (`kind load docker-image`). If a registry is ever added, tag as `labhsathi/<service>:<git-sha>`.

## Commit style

Lowercase, informal, terse: describe what changed and why in a sentence or two, not a changelog. **No AI/Claude/Anthropic attribution in any commit, PR, or doc, ever**: no co-author trailer, no "generated with," no session link. This is a hard rule for this repo, not a default that can be relaxed for convenience.

One clarification on scope: *not volunteering* which tools were used to build this (e.g. no README line about it) is normal and expected. That's not what this rule is about. *Actively denying* it if someone asks directly would be a different thing, and isn't what "no attribution" means here. Keep that distinction: omission, not denial.

## Event-schema evolution

Additive and `Option`al only. Never widen a field on `DocumentJobSubmitted`, `DocumentJobCompleted`, or `ExtractedFields` toward `Vec<u8>` or `serde_json::Value`; that would reopen the exact gap the schema-level privacy boundary exists to close (see the ADR). If a genuinely new byte-shaped need ever comes up, it needs its own explicit design conversation, not a quiet field addition.

## Adding a new scheme (ZB8-12)

The scheme catalog lives in Postgres behind `catalog-service`, not in a compiled array. `catalog-service` upserts the full catalog from `services/labhsathi-core/data/schemes.json` on **every boot** (`sync_catalog` in `services/catalog-service/src/main.rs`, `INSERT ... ON CONFLICT (id) DO UPDATE`), so a new or edited entry there goes live on the next deploy with no manual database step. There is no supported path for editing a scheme directly against a running Postgres — that edit would be silently overwritten by `sync_catalog` on the next boot. `data/schemes.json` is the only source of truth; `api-gateway` caches what it reads from `catalog-service` for 30 seconds (`catalog_client.rs`), so a shipped edit is live within half a minute of the deploy completing.

Most new schemes need no Rust change at all: add an entry to `data/schemes.json` with a `criteria` block (occupation, age/income bounds, category, gender, land requirement, etc.) and `match_schemes` (`services/labhsathi-core/src/schemes.rs`) evaluates it declaratively via `evaluate_criteria`. Every entry still needs `id`, `name`, `authority`, `benefit`, a `documents` list, `official_note`, `source_url`, and `review_date`.

Twelve schemes (PM-KISAN, Ayushman Bharat, Jan Dhan, the three NSAP pensions, PM-SYM, PMMVY, Sukanya Samriddhi, NSP Scholarship, PMAY's urban/rural split) have a hand-written arm in `evaluate_rule` instead, matched by `id`, because they need the three-state match/needs-info distinction and the declarative engine doesn't yet express "ask a follow-up question," only match/no-match. A regression test (`hardcoded_rule_schemes_carry_no_dead_criteria_block` in `schemes.rs`) enforces that these twelve carry **no** `criteria` block in their data row — one would be dead data, since the compiled rule arm is what actually runs. Reach for a rule arm only when declarative criteria genuinely can't express what the scheme needs.
