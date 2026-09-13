// State was a free-text field, so anything typed ("Atlantis", a typo, a
// sentence) went straight to /api/match and came back with results. It's
// now a closed list of India's 28 states and 8 union territories, and the
// canonical English name is the value sent to the API in both languages.

export interface IndianState {
  value: string;
  hi: string;
}

export const INDIAN_STATES: IndianState[] = [
  { value: "Andaman and Nicobar Islands", hi: "अंडमान और निकोबार द्वीपसमूह" },
  { value: "Andhra Pradesh", hi: "आंध्र प्रदेश" },
  { value: "Arunachal Pradesh", hi: "अरुणाचल प्रदेश" },
  { value: "Assam", hi: "असम" },
  { value: "Bihar", hi: "बिहार" },
  { value: "Chandigarh", hi: "चंडीगढ़" },
  { value: "Chhattisgarh", hi: "छत्तीसगढ़" },
  { value: "Dadra and Nagar Haveli and Daman and Diu", hi: "दादरा और नगर हवेली और दमन और दीव" },
  { value: "Delhi", hi: "दिल्ली" },
  { value: "Goa", hi: "गोवा" },
  { value: "Gujarat", hi: "गुजरात" },
  { value: "Haryana", hi: "हरियाणा" },
  { value: "Himachal Pradesh", hi: "हिमाचल प्रदेश" },
  { value: "Jammu and Kashmir", hi: "जम्मू और कश्मीर" },
  { value: "Jharkhand", hi: "झारखंड" },
  { value: "Karnataka", hi: "कर्नाटक" },
  { value: "Kerala", hi: "केरल" },
  { value: "Ladakh", hi: "लद्दाख" },
  { value: "Lakshadweep", hi: "लक्षद्वीप" },
  { value: "Madhya Pradesh", hi: "मध्य प्रदेश" },
  { value: "Maharashtra", hi: "महाराष्ट्र" },
  { value: "Manipur", hi: "मणिपुर" },
  { value: "Meghalaya", hi: "मेघालय" },
  { value: "Mizoram", hi: "मिज़ोरम" },
  { value: "Nagaland", hi: "नागालैंड" },
  { value: "Odisha", hi: "ओडिशा" },
  { value: "Puducherry", hi: "पुडुचेरी" },
  { value: "Punjab", hi: "पंजाब" },
  { value: "Rajasthan", hi: "राजस्थान" },
  { value: "Sikkim", hi: "सिक्किम" },
  { value: "Tamil Nadu", hi: "तमिलनाडु" },
  { value: "Telangana", hi: "तेलंगाना" },
  { value: "Tripura", hi: "त्रिपुरा" },
  { value: "Uttar Pradesh", hi: "उत्तर प्रदेश" },
  { value: "Uttarakhand", hi: "उत्तराखंड" },
  { value: "West Bengal", hi: "पश्चिम बंगाल" },
];

// Older official names and common document spellings a scan can return.
const ALIASES: Record<string, string> = {
  orissa: "Odisha",
  pondicherry: "Puducherry",
  uttaranchal: "Uttarakhand",
  "new delhi": "Delhi",
  "nct of delhi": "Delhi",
  "national capital territory of delhi": "Delhi",
  "j&k": "Jammu and Kashmir",
  "andaman and nicobar": "Andaman and Nicobar Islands",
  "dadra and nagar haveli": "Dadra and Nagar Haveli and Daman and Diu",
  "daman and diu": "Dadra and Nagar Haveli and Daman and Diu",
};

function key(raw: string): string {
  return raw.toLowerCase().replace(/&/g, " and ").replace(/\s+/g, " ").trim();
}

const LOOKUP = new Map<string, string>();
for (const s of INDIAN_STATES) {
  LOOKUP.set(key(s.value), s.value);
  LOOKUP.set(key(s.hi), s.value);
}
for (const [alias, value] of Object.entries(ALIASES)) {
  LOOKUP.set(key(alias), value);
}

/** Canonical state name for recognized wording (any case, "&" or "and",
 * Hindi or English, old spellings), or null -- never a guess. */
export function normalizeState(raw: string | null | undefined): string | null {
  if (!raw) return null;
  return LOOKUP.get(key(raw)) ?? null;
}
