# Scheme Setu

**Track:** Jan Jeevan (Bit N Build Hackathon 2026)

## Problem

Most Indians eligible for central welfare schemes (PM-KISAN, Ayushman Bharat,
old-age/disability pension, scholarships, housing assistance, etc.) never
claim them. The barrier isn't willingness — it's that eligibility rules are
scattered across dozens of scheme PDFs and portals, and nobody translates
"my situation" into "here's what you qualify for and what to bring."
Multiple field studies on scheme awareness among rural/urban poor and
elderly populations report large gaps between eligibility and actual
enrollment (see Sources below).

On top of that, the standard advice ("upload your Aadhaar/income proof to
this portal") asks people to hand over sensitive documents to yet another
system, which is itself a trust barrier for a population already wary of
data misuse.

## Solution

Scheme Setu is a two-step web app:

1. **Structured self-assessment** — a short form (age, income, occupation,
   category, disability status, land holding, etc.) is evaluated against a
   curated eligibility-rules table for ten real central government schemes.
   Matches come back with a plain-language reason, the benefit, and the
   exact documents needed.
2. **Optional document auto-fill, privacy-first** — instead of storing
   uploaded ID/income/land documents, the app sends the image once to a
   vision model to extract only the handful of structured fields the form
   needs, returns them, and **discards the image immediately**. Nothing
   about the document — not the image, not OCR text, not a hash — is
   written to disk, logged, or persisted anywhere. This is the app's core
   differentiator: scheme discovery without a new place for your ID to live.

## Why this design

- **Rule-based matching, not black-box ML** — eligibility determinations
  for government benefits should be explainable. Every match shows exactly
  which fact triggered it.
- **Zero data retention by construction** — there's no database, no file
  storage, and no field in the codebase where a document or its extracted
  contents outlives a single request. This is easy to verify by reading
  `src/ocr.rs`.
- **Scales as a rules table, not a rewrite** — adding a new scheme means
  adding one entry to `src/schemes.rs`; the matching engine doesn't change.

## Tech stack

- **Backend:** Rust (Axum, Tokio)
- **Vision/OCR:** Anthropic Claude API (vision), called per-request, never persisted
- **Frontend:** Static HTML/CSS/vanilla JS (no build step, keeps the demo simple and fast to load)
- **Data:** In-memory Rust rule table (no database — nothing to leak)

## Running it

```bash
# 1. Set your Anthropic API key (only needed for the document auto-fill feature —
#    the core matching form works without it)
export ANTHROPIC_API_KEY=sk-ant-...

# 2. Build & run
cargo run

# 3. Open http://localhost:8080
```

> This project was scaffolded and code-reviewed with Claude's help; it has
> not been `cargo build`-verified in the assistant's sandbox because that
> sandbox has no network access to crates.io. Run `cargo build` locally
> first thing — fix any dependency-version drift before you start
> iterating, since crate versions will have moved since this was written.

## Scope & honesty note (for judges)

The scheme database here (10 schemes) is a demonstration set, not
exhaustive — a production version would ingest more schemes and keep them
current against official notifications. Eligibility rules are simplified
approximations of real criteria; the app frames matches as "worth checking,"
not a final determination, and links the relevant document list so users
can verify on the official portal.

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
- On-device (WASM) document field extraction to remove even the single
  network hop to the vision API
