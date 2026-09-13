# Sample documents

Two entirely synthetic specimen images, script-generated (no real document was scanned, photographed, or used as a template): `specimen-id-card.jpg` and `specimen-income-certificate.jpg`. Both cover every field the extraction schema pulls: age, annual income, state, category, occupation, land holding.

Use these for the demo recording and for exercising the document-scan path yourself without needing to supply a real ID. Fictional country name, placeholder name/numbers, and a visible "SPECIMEN: SYNTHETIC SAMPLE" watermark on both.

**Why not a real document, even one of the author's own or a stock/sample ID found online:** this repo, the demo video, and the screenshots on the "How it works" page are all public. Any real document — even a genuine but "unimportant-looking" one — puts a real name, a real ID number, or a real address into a public GitHub repo permanently, with no way to fully retract it once it's been cloned, indexed, or cached. That's the exact category of harm LabhSathi's own architecture is built to avoid for its users (see [ADR 0001](../adr/0001-event-driven-document-pipeline.md)); shipping a real document as a "sample" in the repo would contradict that on day one. Script-generating a synthetic specimen instead makes this a non-issue rather than a risk to manage.
