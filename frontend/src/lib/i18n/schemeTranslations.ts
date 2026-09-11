import type { Lang } from "./translations";
import type { SchemeMatch } from "../types";

/// Hindi metadata for every scheme id in labhsathi-core::schemes, keyed by
/// the same `id` the backend returns -- name/authority/benefit/documents/
/// official_note, all static per scheme, so translating them client-side
/// (rather than round-tripping a language param to the backend) is
/// straightforward and keeps the Rust rule engine language-agnostic.
///
/// NOT translated here, deliberately: `reason`, the dynamically-generated
/// "why you matched" sentence built in Rust with interpolated values
/// (category, income band, etc.). Translating that accurately needs
/// parallel Hindi sentence templates in the rule engine itself -- real,
/// bounded work, scoped as a following pass rather than done under the
/// same time pressure that risks a wrong or misleading translation of a
/// government-benefit eligibility reason. `reason` stays in English for
/// both languages until that lands; every other field on the card is
/// fully Hindi when `hi` is selected.
const schemeHi: Record<
  string,
  { name: string; authority: string; benefit: string; documents: string[]; official_note: string }
> = {
  "pm-kisan": {
    name: "पीएम-किसान (प्रधानमंत्री किसान सम्मान निधि)",
    authority: "कृषि एवं किसान कल्याण मंत्रालय",
    benefit: "₹6,000/वर्ष, 3 किस्तों में सीधी आय सहायता",
    documents: ["आधार कार्ड", "भूमि स्वामित्व रिकॉर्ड (खतौनी/खसरा)", "बैंक पासबुक", "पासपोर्ट साइज़ फ़ोटो"],
    official_note: "आधिकारिक अधिसूचना के अनुसार संस्थागत भूमिधारकों और कुछ उच्च-आय वर्गों (जैसे आयकरदाता, सरकारी कर्मचारी) को इससे बाहर रखा गया है।",
  },
  "ayushman-bharat": {
    name: "आयुष्मान भारत पीएम-जय",
    authority: "राष्ट्रीय स्वास्थ्य प्राधिकरण",
    benefit: "₹5,00,000/परिवार/वर्ष कैशलेस स्वास्थ्य बीमा",
    documents: ["आधार कार्ड", "राशन कार्ड / SECC परिवार आईडी", "आय प्रमाण पत्र"],
    official_note: "वास्तविक पात्रता केवल आय पर नहीं, बल्कि SECC 2011 की वंचन/व्यावसायिक श्रेणी पर आधारित है — आधिकारिक PM-JAY लाभार्थी पोर्टल पर अपने परिवार की स्थिति जांचें।",
  },
  pmjdy: {
    name: "प्रधानमंत्री जन धन योजना",
    authority: "वित्तीय सेवा विभाग",
    benefit: "शून्य-बैलेंस बैंक खाता, रुपे डेबिट कार्ड, दुर्घटना व जीवन बीमा कवर",
    documents: ["आधार कार्ड", "पते का प्रमाण (यदि आधार न हो)"],
    official_note: "किसी भी भारतीय निवासी के लिए खुला है; कोई आय या व्यवसाय की सीमा नहीं है।",
  },
  "nsap-ignoaps": {
    name: "राष्ट्रीय वृद्धावस्था पेंशन (IGNOAPS)",
    authority: "ग्रामीण विकास मंत्रालय (NSAP)",
    benefit: "मासिक पेंशन (₹200–1000+, राज्य अनुसार अतिरिक्त राशि)",
    documents: ["आधार कार्ड", "आयु प्रमाण", "बीपीएल / आय प्रमाण पत्र", "बैंक पासबुक"],
    official_note: "राज्य सरकारें केंद्रीय राशि में अतिरिक्त योगदान करती हैं; सटीक राशि राज्य अनुसार अलग होती है।",
  },
  "nsap-ignwps": {
    name: "राष्ट्रीय विधवा पेंशन (IGNWPS)",
    authority: "ग्रामीण विकास मंत्रालय (NSAP)",
    benefit: "मासिक पेंशन (₹300+, राज्य अनुसार अतिरिक्त राशि)",
    documents: ["आधार कार्ड", "आयु प्रमाण", "पति का मृत्यु प्रमाण पत्र", "बीपीएल / आय प्रमाण पत्र", "बैंक पासबुक"],
    official_note: "अधिकतर राज्यों में 40-79 वर्ष की विधवाओं को कवर करती है; कुछ राज्य 60 की उम्र में लाभार्थियों को IGNOAPS में स्थानांतरित कर देते हैं — अपने राज्य का नियम जांचें।",
  },
  "nsap-igndps": {
    name: "राष्ट्रीय विकलांगता पेंशन (IGNDPS)",
    authority: "ग्रामीण विकास मंत्रालय (NSAP)",
    benefit: "मासिक विकलांगता पेंशन",
    documents: ["आधार कार्ड", "विकलांगता प्रमाण पत्र (UDID, ≥80%)", "आय प्रमाण पत्र"],
    official_note: "सक्षम चिकित्सा प्राधिकरण से 80% या उससे अधिक प्रमाणित विकलांगता आवश्यक है।",
  },
  "nsp-scholarship": {
    name: "राष्ट्रीय छात्रवृत्ति पोर्टल — प्री/पोस्ट-मैट्रिक छात्रवृत्ति",
    authority: "सामाजिक न्याय एवं अधिकारिता मंत्रालय / अल्पसंख्यक कार्य मंत्रालय",
    benefit: "छात्रों के लिए ट्यूशन फ़ीस प्रतिपूर्ति + भरण-पोषण भत्ता",
    documents: ["आधार कार्ड", "जाति/समुदाय प्रमाण पत्र", "आय प्रमाण पत्र", "पिछले वर्ष की मार्कशीट", "बैंक पासबुक"],
    official_note: "श्रेणी (SC/ST/OBC/अल्पसंख्यक) के अनुसार अलग-अलग योजनाएं और आय सीमाएं लागू होती हैं — scholarships.gov.in पर संबंधित योजना की सीमा जांचें।",
  },
  "sukanya-samriddhi": {
    name: "सुकन्या समृद्धि योजना",
    authority: "वित्त मंत्रालय",
    benefit: "बेटी की शिक्षा/विवाह के लिए उच्च-ब्याज बचत खाता",
    documents: ["बेटी का जन्म प्रमाण पत्र", "अभिभावक का आधार व पैन", "पते का प्रमाण"],
    official_note: "बेटी के 10 वर्ष की होने से पहले खाता खुलवाना ज़रूरी है।",
  },
  "pmay-urban": {
    name: "प्रधानमंत्री आवास योजना — शहरी (PMAY-U)",
    authority: "आवास एवं शहरी कार्य मंत्रालय",
    benefit: "शहरी क्षेत्र में पक्का घर बनाने या खरीदने के लिए वित्तीय सहायता / ब्याज सब्सिडी",
    documents: ["आधार कार्ड", "आय प्रमाण पत्र", "भूमि दस्तावेज़ (यदि स्वामित्व हो)", "बैंक पासबुक"],
    official_note: "शहरी आय सीमाएं और आवेदन प्रक्रिया ग्रामीण संस्करण (PMAY-G) से अलग हैं — यह प्रविष्टि केवल शहरी क्षेत्रों के लिए है।",
  },
  "pmay-rural": {
    name: "प्रधानमंत्री आवास योजना — ग्रामीण (PMAY-G)",
    authority: "ग्रामीण विकास मंत्रालय",
    benefit: "ग्रामीण क्षेत्र में पक्का घर बनाने या खरीदने के लिए वित्तीय सहायता",
    documents: ["आधार कार्ड", "आय प्रमाण पत्र", "भूमि दस्तावेज़ (यदि स्वामित्व हो)", "बैंक पासबुक", "मनरेगा जॉब कार्ड (यदि हो)"],
    official_note: "ग्रामीण आय सीमाएं और लाभार्थी-चयन प्रक्रिया (SECC-आधारित) शहरी संस्करण (PMAY-U) से अलग हैं — यह प्रविष्टि केवल ग्रामीण क्षेत्रों के लिए है।",
  },
  pmmvy: {
    name: "प्रधानमंत्री मातृ वंदना योजना",
    authority: "महिला एवं बाल विकास मंत्रालय",
    benefit: "गर्भावस्था/स्तनपान (पहले जीवित बच्चे) के लिए ₹5,000 नकद सहायता",
    documents: ["आधार कार्ड", "एमसीपी कार्ड (मदर एंड चाइल्ड प्रोटेक्शन कार्ड)", "बैंक पासबुक"],
    official_note: "यह केवल पहले जीवित बच्चे पर लागू होती है; लाभ स्वास्थ्य जांच से जुड़ी किस्तों में दिया जाता है।",
  },
  "pm-sym": {
    name: "प्रधानमंत्री श्रम योगी मान-धन",
    authority: "श्रम एवं रोज़गार मंत्रालय",
    benefit: "60 वर्ष की आयु के बाद ₹3,000 मासिक पेंशन (असंगठित क्षेत्र)",
    documents: ["आधार कार्ड", "बैंक पासबुक (IFSC सहित)", "मोबाइल नंबर"],
    official_note: "18–40 वर्ष के असंगठित क्षेत्र के कामगारों के लिए, जिनकी मासिक आय ₹15,000 तक हो और जो EPFO/ESIC/NPS के अंतर्गत न आते हों।",
  },
};

/** Returns `scheme` as-is for English; swaps in Hindi metadata (name,
 * authority, benefit, documents, official_note) for Hindi, leaving
 * `reason` untranslated -- see the module doc for why. */
export function localizeScheme(scheme: SchemeMatch, lang: Lang): SchemeMatch {
  if (lang === "en") return scheme;
  const hi = schemeHi[scheme.id];
  if (!hi) return scheme; // unknown id (shouldn't happen) -- fail open to English rather than hide the scheme
  return { ...scheme, ...hi };
}
