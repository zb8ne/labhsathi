# 3-minute demo video: shot list

Persona used throughout (matches the pitch deck): **Meena, 34, agricultural worker, ₹1.4 lakh household income, one daughter (age 6), no bank account, lives in a kutcha house, Bihar.**

Style: calm product-launch pacing, 24-30fps, mostly 2-5s cuts, burned-in subtitles, minimal narration, no terminal footage over 2s, no scrolling code.

Fill Meena's profile on `https://labhsathi.info` before recording (occupation: Laborer / unorganised worker, age 34, income 140000, state Bihar, family size 2, daughter's age 6, check "Lives in a kutcha house" and "No bank account", area type Rural) so every take is consistent.

---

## 0:00-0:18 -- Meena and the fragmented problem

- 0:00-0:06: Black frame. Text card fades in: *"Meena needs help from three government departments."* Soft piano note on the fade-in, nothing else.
- 0:06-0:12: Text card: *"She only gets to explain her situation once."* (fades under the first line, doesn't cut hard)
- 0:12-0:18: Cut to three quick, silent stills (1.5-2s each): a scheme eligibility PDF, a crowded portal login screen, a physical queue/paperwork photo (use stock or the specimen documents in `docs/sample-documents/`, not a real portal screenshot). No narration yet -- let the visual pressure build.

## 0:18-0:42 -- Answering the brief directly: agriculture, health, finance

This beat exists to close a gap on its own, before a judge has to ask: Track 3's brief names smart irrigation, remote diagnostics, and micro-credit as its example problems. LabhSathi doesn't build any of those three things directly. What it does is make sure the people who need them actually find and qualify for them. Say that plainly instead of hoping nobody notices the gap.

- 0:18-0:24: Narration over the official Track 3 title card (screenshot it once, use it as a 2-3s establishing beat): *"Track 3 asks technology to reach everyday needs in agriculture, health, and finance."*
- 0:24-0:34: Three fast beats, each a track-brief phrase paired with a real, already-existing scheme LabhSathi surfaces for it (masked wipe between them, 3-3.5s each, small text card + scheme name, no invented visuals):
  - *"An irrigation subsidy already exists."* -> **PM Krishi Sinchayee Yojana**, real benefit on screen: "Up to 55% capital subsidy on drip and sprinkler irrigation."
  - *"Health coverage already exists."* -> **Ayushman Bharat PM-JAY**, real benefit on screen: "₹5,00,000/family/year cashless coverage."
  - *"A first bank account already exists."* -> **Jan Dhan Yojana**, real benefit on screen: "Zero-balance account, RuPay card, accident cover."
- 0:34-0:42: Land the reframe, narration over a Meena persona card: *"The gap isn't building these. It's knowing you qualify. LabhSathi is that missing layer, for one household at a time."*

Keep this honest in the edit: the three scheme names in the middle beat are catalog examples illustrating domain coverage, not necessarily Meena's own results (her real results are shown live in the next section and happen to span the same three domains anyway -- MGNREGA, PM-JAY, Jan Dhan/Atal Pension -- so don't blur the two into implying she personally matched PM Krishi Sinchayee Yojana if she didn't in the live take).

## 0:42-1:45 -- Complete working user journey (the core of the video)

Record this live against `labhsathi.info`, not a mockup.

1. **0:42-0:52** -- Main screen loads. Cursor moves to the scan panel. Narration: *"Meena can scan a document instead of typing everything."* Click Scan, upload a specimen document from `docs/sample-documents/`.
2. **0:52-1:05** -- Show the scan pipeline states (queued -> processing -> extracting) at real speed if it's fast, or a single confident cut if not. Fields populate into the form one by one. Narration: *"Six fields, extracted once."*
3. **1:05-1:20** -- Two fields remain empty (whatever the specimen doc doesn't cover, e.g. land holding / bank account). Cursor fills them in. Narration: *"She confirms the two things a document can't tell us."*
4. **1:20-1:35** -- Click "Find my schemes". Brief loading state (keep under 2s per the measured latency). Results screen appears: *"You may be eligible for 18 schemes."*
5. **1:35-1:45** -- Scroll past the category chips (Pensions, Health & Wellness, Women & Child, Employment & Livelihood, Housing & Shelter, Business & Artisans) slowly, letting the breadth read visually. No narration -- let it breathe.

## 1:45-2:15 -- Explainable results and official next steps

- 1:45-1:58: Slow push into one scheme card (Ayushman Bharat PM-JAY). Highlight the "Worth checking" badge and the one-sentence reason. Narration: *"Never just a list. Every match says why."*
- 1:58-2:08: Cut to the documents-needed row on the same card, then the "Official source" link. Narration: *"And exactly what to bring, verified against the real government portal."*
- 2:08-2:15: Quick cut to the print/checklist view. No narration -- caption only: *"One checklist. Every department."*

## 2:15-2:42 -- Event-driven architecture and privacy proof

- 2:15-2:22: Cut to the Privacy page's pipeline visual (Upload -> Extract -> Discard -> Fields only). Narration: *"The image is read once, then discarded, before the result ever comes back."*
- 2:22-2:30: 3-4 second reveal of the `document.jobs.completed` event schema (reuse the pitch deck's mono block, or screen-record the actual ADR text) with the "image: never a field" tag. This is the single technical beat -- keep it under 5s total per the plan.
- 2:30-2:42: One sentence, on a dark card, no visuals needed: *"If the OCR worker goes down, eligibility matching still works. The two are decoupled by design."* This is the graceful-degradation beat the plan calls out explicitly -- don't cut it for time.

## 2:42-3:00 -- Scale through CSCs/NGOs and closing

- 2:42-2:50: Pull back from Meena's single result to the "One household today, every CSC and NGO desk tomorrow" framing (reuse pitch deck slide 5 as a static card, 3-4s).
- 2:50-2:58: Logo + tagline card: *"From document to benefit checklist."*
- 2:58-3:00: Final card: `labhsathi.info` + `github.com/zb8ne/labhsathi`, held for 2 full seconds (don't rush the last frame -- judges pause here to write down the link).

---

## Music

Look for an instrumental, unlicensed-safe track (YouTube Audio Library, Pixabay Music, or Uppbeat's free tier) matching:
- 80-100 BPM, warm ambient synth or soft piano/plucks, no lyrics, no heavy percussion
- One controlled dynamic rise around 1:20-1:35 (the "18 schemes" reveal) -- pick a track that has a natural lift there, don't force it with an edit
- Duck it 10-15dB under narration; let it play solo (no narration) during 0:00-0:12, 1:35-1:45, and 2:58-3:00

Search terms that tend to work on those libraries: "calm technology," "warm corporate minimal," "gentle piano ambient."

## Sound design (keep it sparse)

- One soft "scan" chime when the document upload starts (0:42)
- A muted, short transition sound (not a whoosh) between the three domain cards (0:26-0:42)
- A restrained two-note "complete" tone when the 18-schemes result lands (1:20)
- Nothing else -- silence is doing work in this cut, don't fill it

## Recording checklist

- [ ] Reset Meena's profile fresh before the take (clear localStorage or use a private window) so the numbers match this shot list exactly
- [ ] Confirm `labhsathi.info` is on the latest deploy before recording (check the served JS/CSS asset hash against your last `git push`)
- [ ] Record at native resolution, crop/frame in the edit -- don't record already-cropped
- [ ] Burn in subtitles for every narration line (judges may watch muted)
- [ ] Final export: 1920x1080, 3:00 or under, MP4
