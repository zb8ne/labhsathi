# LabhSathi — 3-Minute Video Pitch Script & Production Brief
**Track 3: Jan Jeevan (Agriculture • Healthcare • Financial Inclusion)**
**Pacing:** 24–30 fps, calm tech launch aesthetic, ~130 words per minute narration, intentional silence, burned-in subtitles.

---

## 🎯 The Strategic Reframe (Why This Pitch Wins Track 3)

> **The Judge's Potential Concern:**
> *"Track 3 lists smart irrigation, remote diagnostics, and micro-credit scoring. LabhSathi is a welfare eligibility engine. Did they deviate?"*
>
> **Our Winning Answer (Delivered in first 40s):**
> *"Point solutions fail when citizens can't afford or access them. A farmer cannot adopt smart drip irrigation without the PM Krishi Sinchayee capital subsidy. A rural family cannot benefit from remote healthcare if a single hospitalization bankrupts them without Ayushman Bharat. An unbanked laborer cannot use micro-credit or digital payments without a Jan Dhan zero-balance account. **LabhSathi is the foundational access layer underneath all three Jan Jeevan pillars.**"*

---

## ⏱️ Master Timetable & Shot-by-Shot Script

| Timecode | Section | Visual Scene & Action | Spoken Voiceover (Read Exactly) | On-Screen Text / Subtitles |
| :--- | :--- | :--- | :--- | :--- |
| **0:00 – 0:08** | **Hook** | Pure black. Clean serif typography fades in. Soft, warm ambient piano pluck. | *(Silence for 2s)*<br>"This is Meena. She is thirty-four, an agricultural wage worker in rural Bihar, raising a young daughter in a kutcha home." | **Meena, 34**<br>Agricultural worker • ₹1.4L income • Unbanked |
| **0:08 – 0:18** | **The Reality** | Fade into 3 quick, authentic stills (1.5s each): a complex 40-page gazette PDF, a dense portal captcha, a long physical queue outside a block office. | "She earns ₹1.4 lakh a year, and she has no bank account. Meena needs support from multiple departments. But she only gets to explain her life once." | *Scattered portals. Legal jargon. Missing documents.* |
| **0:18 – 0:28** | **The Track 3 Bridge** | Minimal slide showing the 3 Track 3 pillars: **Agriculture**, **Healthcare**, **Financial Inclusion**. Subtle highlight wipes across them. | "Track 3 asks how technology solves everyday human needs across agriculture, healthcare, and finance. Often, tech builds isolated point solutions." | **Jan Jeevan Track**<br>Agriculture • Healthcare • Financial Inclusion |
| **0:28 – 0:42** | **The 3-Domain Reframe** | Fast, clean 3-part card wipe showing real existing schemes: <br>1. PM Krishi Sinchayee Yojana (Irrigation)<br>2. Ayushman Bharat PM-JAY (Health)<br>3. PM Jan Dhan Yojana (Finance) | "An irrigation subsidy already exists.<br>Five lakh rupees of cashless health cover already exists.<br>A zero-balance bank account already exists.<br>The gap isn't building these. The gap is knowing you qualify. LabhSathi is that missing layer." | **The Missing Foundation**<br>From disconnected schemes to guaranteed access. |
| **0:42 – 0:54** | **Step 1: Document Scan** | Live recording on `labhsathi.info`. Cursor clicks **Scan Document**. Uploads sample income certificate. Subtle scan chime. | "Instead of filling dozens of forms across government portals, Meena simply scans one document." | *Document Scan • Optical Character Extraction* |
| **0:54 – 1:08** | **Step 2: Instant Extraction** | Input fields animate and populate cleanly with green indicators: Age 34, Occupation: Laborer, Income: ₹1,40,000, State: Bihar, Family: 2. | "Six fields populate automatically in under three seconds. No re-typing. No confusion." | *6 Fields Auto-Populated*<br>Age • Income • Occupation • State • Family Size |
| **1:08 – 1:22** | **Step 3: Missing Nuance** | Cursor scrolls to two unchecked items: *"Lives in a kutcha house"* and *"No bank account"*. Cursor checks both. Selects Area: *Rural*. | "She simply confirms the two things a piece of paper cannot tell us: her housing condition, and that she has no bank account." | *Human in the loop: 2 questions answered* |
| **1:22 – 1:35** | **Step 4: The Match** | Cursor clicks **Find My Schemes**. Restrained completion tone. Instant switch to Results screen: **18 Schemes Matched**. Background music swells slightly. | "With one click, LabhSathi evaluates hundreds of central welfare rules in under one second. Eighteen schemes matched." | **18 Verified Schemes Found**<br>Matched in < 250ms |
| **1:35 – 1:52** | **Domain 1 & 2: Agriculture & Healthcare** | Smooth camera push into: <br>1. **MGNREGA / Agriculture Livelihood**<br>2. **Ayushman Bharat PM-JAY** card. Expands card: shows *"Worth Checking"* badge and criteria explanation. | "Every match covers her life. Under Healthcare: Ayushman Bharat provides five lakh rupees in cashless hospital cover. Not a black-box guess—LabhSathi tells her exactly why she qualifies." | **Healthcare Domain**<br>Ayushman Bharat PM-JAY • ₹5,00,000 Cover<br>Transparent match criteria |
| **1:52 – 2:06** | **Domain 3: Financial Inclusion** | Camera pans to **PM Jan Dhan Yojana** and **Atal Pension Yojana / PM-SYM** cards. Shows checklist of required documents: Aadhaar, 2 photos. | "Under Financial Inclusion: Jan Dhan Yojana and PM-SYM give her an immediate path to open her first bank account and secure pension benefits. Complete with an exact document checklist." | **Financial Inclusion Domain**<br>PM Jan Dhan Yojana • Zero-Balance Account<br>Actionable Document Checklist |
| **2:06 – 2:16** | **Official Links & Verification** | Cursor hovers over **Official Source** link (`pmjay.gov.in`). Clicks **Print / Save Checklist**. | "Direct links to verified government portals. One printable checklist for every department." | *100% Sourced from Official Portals*<br>Downloadable Action Checklist |
| **2:16 – 2:32** | **Privacy & Architecture Proof** | Transition to clean dark architecture view. Highlight Kafka topic `document.jobs.completed`. Show event schema JSON with tag: `image: never a field`. Show Redis ephemeral handoff. | "Behind this is an event-driven architecture in Rust and Kafka. Privacy is structural: the document image is held ephemerally in Redis, processed, and discarded immediately. In our Kafka schemas, the raw image field does not even exist." | **Privacy by Architecture**<br>Raw image absent from Kafka schemas<br>Zero-persistence Redis handoff |
| **2:32 – 2:44** | **Graceful Failure & Scalability** | Architecture diagram shows OCR Worker decoupled from Matcher. | "Our OCR workers scale independently from eligibility matching. And if the vision pipeline ever goes offline, scheme discovery never stops. Graceful degradation by design." | **Resilient Engineering**<br>Independent HPA scaling • Graceful failure mode |
| **2:44 – 2:54** | **Scale & Grassroots Impact** | Pull back visual from phone screen to CSC (Common Service Centre) village kiosk operator and grassroots NGO worker using the tool. | "Designed not just for smartphones, but for village CSC operators and NGO field workers helping thousands of Meenas every day." | **Grassroots Scale**<br>Common Service Centres (CSC) • Village Kiosks • NGOs |
| **2:54 – 3:00** | **Closing Reveal** | Minimal brand screen: LabhSathi logo, tagline, live domain, and GitHub repository. Music resolves on a clean acoustic chord. Hold for 3 seconds. | "LabhSathi. From document to benefit checklist." *(Hold music to end)* | **LabhSathi**<br>`labhsathi.info` • `github.com/zb8ne/labhsathi` |

---

## 🎙️ Spoken Narration Script Only (For Voiceover Recording)

Use this clean script for reading into the microphone. Read at a measured, calm, confident pace:

> *"This is Meena. She is thirty-four, an agricultural wage worker in rural Bihar, raising a young daughter in a kutcha home. She earns ₹1.4 lakh a year, and she has no bank account. Meena needs support from multiple departments. But she only gets to explain her life once.*
>
> *Track 3 asks how technology solves everyday human needs across agriculture, healthcare, and finance. Often, tech builds isolated point solutions.*
>
> *An irrigation subsidy already exists.*
> *Five lakh rupees of cashless health cover already exists.*
> *A zero-balance bank account already exists.*
> *The gap isn't building these. The gap is knowing you qualify. LabhSathi is that missing layer.*
>
> *Instead of filling dozens of forms across government portals, Meena simply scans one document.*
>
> *Six fields populate automatically in under three seconds. No re-typing. No confusion.*
>
> *She simply confirms the two things a piece of paper cannot tell us: her housing condition, and that she has no bank account.*
>
> *With one click, LabhSathi evaluates hundreds of central welfare rules in under one second. Eighteen schemes matched.*
>
> *Every match covers her life. Under Healthcare: Ayushman Bharat provides five lakh rupees in cashless hospital cover. Not a black-box guess—LabhSathi tells her exactly why she qualifies.*
>
> *Under Financial Inclusion: Jan Dhan Yojana and PM-SYM give her an immediate path to open her first bank account and secure pension benefits. Complete with an exact document checklist.*
>
> *Direct links to verified government portals. One printable checklist for every department.*
>
> *Behind this is an event-driven architecture in Rust and Kafka. Privacy is structural: the document image is held ephemerally in Redis, processed, and discarded immediately. In our Kafka schemas, the raw image field does not even exist.*
>
> *Our OCR workers scale independently from eligibility matching. And if the vision pipeline ever goes offline, scheme discovery never stops. Graceful degradation by design.*
>
> *Designed not just for smartphones, but for village CSC operators and NGO field workers helping thousands of Meenas every day.*
>
> *LabhSathi. From document to benefit checklist."*

**Total Word Count:** 328 words  
**Pacing:** Exactly ~2 minutes 25 seconds of speech across a 3-minute video (leaves ~35 seconds of intentional silence, visual pacing, and sound cues).

---

## 🎬 Video Recording & Screen Capture Checklist

1. **Browser State:**
   - Open Chrome at `https://labhsathi.info` (1920x1080 resolution, 100% zoom).
   - Test in Incognito / Private window with clean local storage.
2. **Demo Data for Meena:**
   - **Age:** 34
   - **Gender:** Female
   - **Occupation:** Laborer / unorganised worker (Agricultural wage worker)
   - **Household Income:** ₹1,40,000
   - **State:** Bihar
   - **Family Size:** 2 (includes 1 daughter, age 6)
   - **Checkboxes:** Check *"Lives in a kutcha house"*, check *"No bank account"*.
   - **Area:** Rural
3. **Audio Track Style:**
   - Warm, ambient minimal electronic or gentle piano plucks (85–95 BPM).
   - Sidechain ducking: -12 dB to -15 dB whenever voiceover is active.
   - Boost volume by +4 dB during the 1:22–1:35 match reveal.
4. **Subtitles:**
   - Font: Inter or Roboto, medium weight, clean dark translucent background pill or crisp drop shadow.
   - Centered lower third.
