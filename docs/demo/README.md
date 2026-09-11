# Demo walkthrough

`walkthrough.mp4` (also `walkthrough-raw.webm`) is a real, automated capture
of the golden path against the local Docker Compose stack -- actual Kafka,
Redis, ocr-worker, and a real call to Anthropic's vision API against
`docs/sample-documents/specimen-income-certificate.jpg`:

1. Main screen, scan panel receives the specimen document.
2. Real extraction (age, income, state, category, land holding) fills the
   form; age wasn't on that document type, so it's completed manually --
   showing auto-fill and manual entry working together, not just the
   happy path.
3. Results screen, real match against the rule engine.
4. Privacy screen, the four-step pipeline explainer.

This is a raw capture (Playwright driving a real browser, no narration or
edits) -- a solid base for the actual submission video, not the finished
deliverable. Voiceover/editing still needed before this goes in the
6-slide deck or the submission itself.
