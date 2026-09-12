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

The scheme catalog lives in Postgres behind `catalog-service`, not in a compiled array. `catalog-service` only seeds the table from `services/labhsathi-core/data/schemes.json` when it's empty (`seed_if_empty` in `services/catalog-service/src/main.rs`), so a new entry added there needs either a fresh database or a manual `INSERT` against a running one to actually show up.

Most new schemes need no Rust change at all: add an entry to `data/schemes.json` with a `criteria` block (occupation, age/income bounds, category, gender, land requirement, etc.) and `match_schemes` (`services/labhsathi-core/src/schemes.rs`) evaluates it declaratively via `evaluate_criteria`. Every entry still needs `id`, `name`, `authority`, `benefit`, a `documents` list, `official_note`, `source_url`, and `review_date`.

A handful of schemes (PM-KISAN, the NSAP pensions, PMAY's urban/rural split, PM-SYM) have a hand-written arm in `evaluate_rule` instead, matched by `id`, because they need the three-state match/needs-info distinction and the declarative engine doesn't yet express "ask a follow-up question," only match/no-match. Reach for a rule arm only when declarative criteria genuinely can't express what the scheme needs.
