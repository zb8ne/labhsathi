# LabhSathi: 3-minute video pitch script and production brief

**Track 3: Jan Jeevan (Agriculture, Healthcare, Financial Inclusion)**
**Pacing:** 24-30 fps, calm tech launch aesthetic, roughly 110 words per minute narration (deliberately slower than average, this is a spoken-word-light cut), intentional silence, burned-in subtitles.

---

## The strategic reframe: why this pitch answers Track 3

Track 3's brief names smart irrigation, remote diagnostics, and micro-credit scoring as its example problems. LabhSathi doesn't build any of those three things directly. What it does is make sure the people who need them actually find and qualify for them: PM Krishi Sinchayee Yojana already funds irrigation subsidies, Ayushman Bharat already provides five lakh rupees of health cover, Jan Dhan already gives a first bank account. The barrier was never that these didn't exist. It was that nobody found them. LabhSathi is the access layer underneath all three Jan Jeevan pillars, addressed directly in the first 30 seconds so a judge never has to ask.

Look closer at the brief's own three examples and they share one unstated assumption: that the beneficiary already knows what's available and how to claim it. Smart irrigation, remote diagnostics, and micro-credit scoring are three separate technical builds for three separate pillars, but the actual bottleneck sits underneath all three, at the same layer, for the same reason. That reframes the problem itself: it's not three disconnected verticals needing three disconnected products. It's one unified access problem wearing three different department names. A twenty-thousand-rupee medicine bill can bankrupt a family that has never heard a five-lakh-rupee grant already covers it. The technology to cover that bill existed the entire time. The gap was never engineering. It was that nobody told her.

---

## Master timetable and shot-by-shot script

| Timecode | Section | Visual scene and action | Spoken voiceover (read exactly) | On-screen text / subtitles |
| :--- | :--- | :--- | :--- | :--- |
| **0:00-0:08** | Hook | Pure black. Clean serif type fades in. One soft piano note. | *(silence, 2s)* "This is Meena. Thirty-four. An agricultural wage worker in rural Bihar." | **Meena, 34**<br>Agricultural worker · ₹1.4L income · Unbanked |
| **0:08-0:18** | The reality | Three quick stills, 1.5s each: a dense scheme PDF, a portal login screen, a queue outside a block office. | "No bank account. Raising her daughter in a kutcha home. She needs three departments. She only gets to explain her life once." | *Scattered portals. Buried eligibility rules.* |
| **0:18-0:26** | The Track 3 bridge | Minimal slide, three pillars: Agriculture, Healthcare, Financial Inclusion. Subtle highlight wipe. | "Track 3 asks how technology reaches everyday needs in agriculture, health, and finance." | **Jan Jeevan Track**<br>Agriculture · Healthcare · Financial Inclusion |
| **0:26-0:36** | The shared assumption | Same three-pillar slide, but each pillar's brief phrase (smart irrigation / remote diagnostics / micro-credit scoring) fades in under its label, then all three dim except one shared underline. | "Smart irrigation. Remote diagnostics. Micro-credit scoring. Three different builds. One shared assumption: that she already knows what exists, and how to claim it." | **One hidden assumption**<br>She already knows it exists |
| **0:36-0:48** | The reframe | Three-card wipe, one real scheme per card: PM Krishi Sinchayee Yojana, Ayushman Bharat PM-JAY, PM Jan Dhan Yojana. | "A fifty-five percent subsidy on drip irrigation. Five lakh rupees of cashless health cover. A zero-balance bank account. All three, already funded." | **Already funded**<br>PMKSY · PM-JAY · Jan Dhan |
| **0:48-1:00** | The stakes | Cut to a single stark card: a medicine strip and a price tag, ₹20,000, dissolving into the PM-JAY logo and ₹5,00,000. | "A twenty-thousand-rupee medicine bankrupts a family that never heard a five-lakh grant exists. The technology was never the gap. The awareness was." | **The real gap**<br>Not engineering. Awareness. |
| **1:00-1:08** | The resolution | Meena persona card returns, now with a single connecting line drawn under all three pillar icons. | "LabhSathi is that missing layer. One layer, under agriculture, health, and finance." | **LabhSathi**<br>One access layer, three departments |
| **1:08-1:18** | Step 1: scan | Live on `labhsathi.info`. Click Scan. Upload a specimen document. Soft chime. | "Meena scans one document instead of filling out ten." | *Scan once, then discarded* |
| **1:18-1:30** | Step 2: extraction | Fields populate one by one, green highlight: age, income, occupation, state, family size. | "Six fields, extracted in seconds. No retyping." | **6 fields auto-filled** |
| **1:30-1:42** | Step 3: the two questions | Cursor checks "Lives in a kutcha house" and "No bank account". Selects Rural. | "She confirms two things a document can't tell us. Her housing. Her banking." | *Human confirms what a scan can't know* |
| **1:42-1:53** | Step 4: the match | Click Find My Schemes. Brief pause, then Results: 28 Schemes Matched. | "One click. Over a hundred welfare rules checked. Twenty-eight schemes matched." | **28 schemes matched**<br>Under 2 seconds |
| **1:53-2:07** | Healthcare, explained | Push into the Ayushman Bharat PM-JAY card. Highlight the reason text. | "Every match tells her why. Ayushman Bharat: five lakh rupees, cashless. Not a guess. A reason." | **Health cover, explained**<br>₹5,00,000 · Ayushman Bharat PM-JAY |
| **2:07-2:19** | Financial inclusion, explained | Pan to Jan Dhan Yojana and PM-SYM cards. Show the document checklist. | "Jan Dhan and PM-SYM: her first bank account, and a path to a pension." | **Financial inclusion**<br>Jan Dhan · PM-SYM · Document checklist |
| **2:19-2:27** | Official links | Hover the Official Source link. Click Print. | "Verified links. One checklist. Every department." | *Every match links to a real .gov.in source* |
| **2:27-2:41** | Privacy, by architecture | Dark architecture view. `document.jobs.completed` schema, tag: image never a field. | "Underneath: Kafka, Redis, Rust. The image is read once and discarded, before the result comes back. There's no field for it to leak into." | **Structural privacy**<br>No image field in either Kafka schema |
| **2:41-2:51** | Graceful failure | Diagram: ocr-worker decoupled from the matcher. | "If the vision pipeline goes down, matching doesn't stop. They're built apart, on purpose." | **Decoupled by design** |
| **2:51-2:57** | Scale | Pull back from a phone to a CSC kiosk operator and an NGO field worker. | "For a phone. For a village kiosk. For an NGO worker helping fifty people a day." | **Same flow. More desks.** |
| **2:57-3:00** | Close | Logo, tagline, live link, GitHub. Hold. | "LabhSathi. From document to benefit checklist." | **LabhSathi**<br>`labhsathi.info` · `github.com/zb8ne/labhsathi` |

*Timecodes past 0:18 are re-derived below (word count and measured audio length), not hand-guessed — see the pacing note after the narration block. Treat every boundary as provisional until checked against the actual recorded voiceover length in the edit.*

---

## Spoken narration only (for the ElevenLabs read)

Read at a measured, confident pace. Indian English accent, professional register, not a hard sell.

This is Meena. Thirty-four. An agricultural wage worker in rural Bihar. No bank account. Raising her daughter in a kutcha home.

She needs three departments. She only gets to explain her life once.

Track 3 asks how technology reaches everyday needs in agriculture, health, and finance.

Smart irrigation. Remote diagnostics. Micro-credit scoring. Three different builds. One shared assumption: that she already knows what exists, and how to claim it.

A fifty-five percent subsidy on drip irrigation. Five lakh rupees of cashless health cover. A zero-balance bank account. All three, already funded.

A twenty-thousand-rupee medicine bankrupts a family that never heard a five-lakh grant exists. The technology was never the gap. The awareness was.

LabhSathi is that missing layer. One layer, under agriculture, health, and finance.

Meena scans one document instead of filling out ten.

Six fields, extracted in seconds. No retyping.

She confirms two things a document can't tell us. Her housing. Her banking.

One click. Over a hundred welfare rules checked. Twenty-eight schemes matched.

Every match tells her why. Ayushman Bharat: five lakh rupees, cashless. Not a guess. A reason.

Jan Dhan and PM-SYM: her first bank account, and a path to a pension.

Verified links. One checklist. Every department.

Underneath: Kafka, Redis, Rust. The image is read once and discarded, before the result comes back. There's no field for it to leak into.

If the vision pipeline goes down, matching doesn't stop. They're built apart, on purpose.

For a phone. For a village kiosk. For an NGO worker helping fifty people a day.

LabhSathi. From document to benefit checklist.

**Word count:** 261 words.
**Recorded length (measured, Raj voice):** 2:08. That leaves 52 seconds of pure visual/silence room across the 3:00 video for the scan demo, the 28-schemes scroll, and the closing hold -- less slack than the earlier cut had, since this version now carries the full Track 3 justification (the shared-assumption argument and the bankruptcy stakes line) instead of skimming past it, but still comfortably under the runway. The master table's timecodes above were estimated at 110wpm before recording; Raj's actual pace runs closer to 122wpm, so treat the table's boundaries as directional and re-time each cut against `voiceover-raj.mp3` directly in the edit.

---

## Video recording and screen capture checklist

1. **Browser state:**
   - Chrome at `https://labhsathi.info`, 1920x1080, 100% zoom.
   - Private/incognito window with clean localStorage, so every take starts from the same blank form.
2. **Meena's profile:**
   - Age 34, Female, Occupation: Laborer / unorganised worker
   - Household income: ₹140,000, State: Bihar, Family size: 2, daughter's age 6
   - Check "Lives in a kutcha house" and "No bank account", Area: Rural
3. **Audio:**
   - Warm ambient synth or gentle piano, 85-95 BPM, no lyrics.
   - Duck the music 10-15dB under narration; let it play solo during the opening 8s, the 28-schemes reveal, and the closing hold.
4. **Subtitles:**
   - Medium-weight sans (Inter or the system font already used in the product), centered lower third, burned in for every line.
