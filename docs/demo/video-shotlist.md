# 3-minute demo video: shot list

Persona used throughout (matches the pitch deck): **Meena, 34, agricultural worker, ₹1.4 lakh household income, one daughter (age 6), no bank account, lives in a kutcha house, Bihar.**

Style: calm product-launch pacing, 24-30fps, mostly 2-5s cuts, burned-in subtitles, minimal narration, no terminal footage over 2s, no scrolling code.

Fill Meena's profile on `https://labhsathi.info` before recording (occupation: Laborer / unorganised worker, age 34, income 140000, state Bihar, family size 2, daughter's age 6, check "Lives in a kutcha house" and "No bank account", area type Rural) so every take is consistent.

---

## 0:00-0:18 -- Meena and the fragmented problem

- 0:00-0:06: Black frame. Text card fades in: *"Meena needs help from three government departments."* Soft piano note on the fade-in, nothing else.
- 0:06-0:12: Text card: *"She only gets to explain her situation once."* (fades under the first line, doesn't cut hard)
- 0:12-0:18: Cut to three quick, silent stills (1.5-2s each): a scheme eligibility PDF, a crowded portal login screen, a physical queue/paperwork photo (use stock or the specimen documents in `docs/sample-documents/`, not a real portal screenshot). No narration yet -- let the visual pressure build.

## 0:18-1:00 -- Answering the brief directly: one unified access problem, not three

This beat exists to close a gap on its own, before a judge has to ask: Track 3's brief names smart irrigation, remote diagnostics, and micro-credit scoring as its example problems -- three different builds for three different pillars. Look closer and all three share one unstated assumption: that the beneficiary already knows what's available and how to claim it. That's the actual bottleneck, sitting underneath all three, in the same place, for the same reason. It reframes the problem itself: not three disconnected verticals needing three disconnected products, but one unified access problem wearing three department names. Land that with a concrete stake, not an abstraction: a twenty-thousand-rupee medicine bill bankrupts a family that never heard a five-lakh-rupee grant already covers it. The technology to cover that bill existed the whole time. Say all of this plainly instead of hoping nobody notices the gap.

- 0:18-0:26: Narration over the official Track 3 title card (screenshot it once, use it as an establishing beat): *"Track 3 asks how technology reaches everyday needs in agriculture, health, and finance."*
- 0:26-0:36: Same three-pillar slide, but the brief's own phrase fades in under each pillar (smart irrigation / remote diagnostics / micro-credit scoring), then all three dim except one shared underline connecting them. Narration: *"Smart irrigation. Remote diagnostics. Micro-credit scoring. Three different builds. One shared assumption: that she already knows what exists, and how to claim it."*
- 0:36-0:48: Three fast beats, each a track-brief phrase paired with a real, already-existing scheme LabhSathi surfaces for it (masked wipe between them, no invented visuals):
  - **PM Krishi Sinchayee Yojana**, real benefit on screen: "Up to 55% capital subsidy on drip and sprinkler irrigation."
  - **Ayushman Bharat PM-JAY**, real benefit on screen: "₹5,00,000/family/year cashless coverage."
  - **Jan Dhan Yojana**, real benefit on screen: "Zero-balance account, RuPay card, accident cover."
  - Narration across all three: *"A fifty-five percent subsidy on drip irrigation. Five lakh rupees of cashless health cover. A zero-balance bank account. All three, already funded."*
- 0:48-1:00: Cut to a single stark card: a medicine strip and a ₹20,000 price tag, dissolving into the PM-JAY logo and ₹5,00,000. Narration: *"A twenty-thousand-rupee medicine bankrupts a family that never heard a five-lakh grant exists. The technology was never the gap. The awareness was."* Then the Meena persona card returns with one connecting line drawn under all three pillar icons: *"LabhSathi is that missing layer. One layer, under agriculture, health, and finance."*

Keep this honest in the edit: the three scheme names in the middle beat are catalog examples illustrating domain coverage, not necessarily Meena's own results (her real results are shown live in the next section and happen to span the same three domains anyway -- MGNREGA, PM-JAY, Jan Dhan/Atal Pension -- so don't blur the two into implying she personally matched PM Krishi Sinchayee Yojana if she didn't in the live take). The ₹20,000 medicine figure is illustrative (a plausible out-of-pocket cost for the kind of care PM-JAY covers), not a specific cited price -- keep it framed as an example, not a stat.

## 1:00-1:55 -- Complete working user journey (the core of the video)

Record this live against `labhsathi.info`, not a mockup.

1. **1:00-1:10** -- Main screen loads. Cursor moves to the scan panel. Narration: *"Meena scans one document instead of filling out ten."* Click Scan, upload a specimen document from `docs/sample-documents/`.
2. **1:10-1:23** -- Show the scan pipeline states (queued -> processing -> extracting) at real speed if it's fast, or a single confident cut if not. Fields populate into the form one by one. Narration: *"Six fields, extracted in seconds. No retyping."*
3. **1:23-1:36** -- Two fields remain empty (whatever the specimen doc doesn't cover, e.g. land holding / bank account). Cursor fills them in. Narration: *"She confirms two things a document can't tell us. Her housing. Her banking."*
4. **1:36-1:48** -- Click "Find my schemes". Brief loading state (keep under 2s per the measured latency). Results screen appears: *"One click. Over a hundred welfare rules checked. Twenty-eight schemes matched."*
5. **1:48-1:55** -- Scroll past the category chips (Pensions, Health & Wellness, Women & Child, Financial Inclusion, Employment & Livelihood, Agriculture & Farmers, Housing & Shelter, Business & Artisans), letting the breadth read visually. No narration -- let it breathe.

## 1:55-2:20 -- Explainable results and official next steps

- 1:55-2:07: Slow push into one scheme card (Ayushman Bharat PM-JAY). Highlight the "Worth checking" badge and the one-sentence reason. Narration: *"Every match tells her why. Ayushman Bharat: five lakh rupees, cashless. Not a guess. A reason."*
- 2:07-2:15: Pan to Jan Dhan Yojana and PM-SYM cards, show the document checklist. Narration: *"Jan Dhan and PM-SYM: her first bank account, and a path to a pension."* **Recording note:** neither card lives under the "Financial Inclusion" chip -- Jan Dhan is tagged Business & Artisans and PM-SYM is tagged Pensions & Social Security in the catalog. Find both from the **All** tab (verified live: both are real matches on Meena's profile), don't waste a take clicking the Financial Inclusion filter looking for them.
- 2:15-2:20: Quick cut to the "Official source" link and the print/checklist view. Narration: *"Verified links. One checklist. Every department."*

## 2:20-2:44 -- Event-driven architecture and privacy proof

- 2:20-2:30: Cut to the Privacy page's pipeline visual (Upload -> Extract -> Discard -> Fields only). Narration: *"Underneath: Kafka, Redis, Rust. The image is read once and discarded, before the result comes back. There's no field for it to leak into."*
- 2:30-2:36: 3-4 second reveal of the `document.jobs.completed` event schema (reuse the pitch deck's mono block, or screen-record the actual ADR text) with the "image: never a field" tag. This is the single technical beat -- keep it under 5s total per the plan.
- 2:36-2:44: One sentence, on a dark card, no visuals needed: *"If the vision pipeline goes down, matching doesn't stop. They're built apart, on purpose."* This is the graceful-degradation beat the plan calls out explicitly -- don't cut it for time.

## 2:44-3:00 -- Scale through CSCs/NGOs and closing

- 2:44-2:51: Pull back from Meena's single result to the "One household today, every CSC and NGO desk tomorrow" framing (reuse pitch deck slide 5 as a static card). Narration: *"For a phone. For a village kiosk. For an NGO worker helping fifty people a day."*
- 2:51-2:57: Logo + tagline card: *"LabhSathi. From document to benefit checklist."*
- 2:57-3:00: Final card: `labhsathi.info` + `github.com/zb8ne/labhsathi`, held to the very end (don't rush the last frame -- judges pause here to write down the link).

*All timecodes past 0:18 are re-derived to keep the full cut at 3:00 after adding the shared-assumption and bankruptcy-stakes beats (see `video-pitch-script.md`'s pacing note). Treat them as provisional until checked against the actual recorded voiceover length.*

---

## Music

Look for an instrumental, unlicensed-safe track (YouTube Audio Library, Pixabay Music, or Uppbeat's free tier) matching:
- 80-100 BPM, warm ambient synth or soft piano/plucks, no lyrics, no heavy percussion
- One controlled dynamic rise around 1:36-1:48 (the "28 schemes" reveal) -- pick a track that has a natural lift there, don't force it with an edit
- Duck it 10-15dB under narration; let it play solo (no narration) during 0:00-0:12, 1:48-1:55, and 2:57-3:00

Search terms that tend to work on those libraries: "calm technology," "warm corporate minimal," "gentle piano ambient."

## Sound design (keep it sparse)

- One soft "scan" chime when the document upload starts (1:00)
- A muted, short transition sound (not a whoosh) between the three domain cards (0:36-0:48)
- A restrained two-note "complete" tone when the 28-schemes result lands (1:36)
- Nothing else -- silence is doing work in this cut, don't fill it

## Recording checklist

- [ ] Reset Meena's profile fresh before the take (clear localStorage or use a private window) so the numbers match this shot list exactly
- [ ] Confirm `labhsathi.info` is on the latest deploy before recording (check the served JS/CSS asset hash against your last `git push`)
- [ ] Record at native resolution, crop/frame in the edit -- don't record already-cropped
- [ ] Burn in subtitles for every narration line (judges may watch muted)
- [ ] Final export: 1920x1080, 3:00 or under, MP4
