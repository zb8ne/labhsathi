// A vision model reads free-form document text, not a closed vocabulary --
// "Agricultural Labourer" and "farmer" and "Kisan" are all plausible things
// it could hand back for the same real-world occupation. Silently writing
// that raw string into a <select> whose only valid values are the exact
// keys in translations.ts' occupationOptions/categoryOptions produces a
// dropdown showing nothing selected (or, worse, a value that *looks*
// selected but isn't one of the options the matcher understands). Map
// recognized wording onto the canonical value; anything unrecognized is
// left for the person to pick themselves rather than guessed at.

const OCCUPATION_SYNONYMS: Record<string, string> = {
  farmer: "farmer",
  farming: "farmer",
  agriculture: "farmer",
  agricultural: "farmer",
  "agricultural worker": "farmer",
  "agricultural labourer": "farmer",
  "agricultural laborer": "farmer",
  cultivator: "farmer",
  kisan: "farmer",

  laborer: "laborer",
  labourer: "laborer",
  labour: "laborer",
  labor: "laborer",
  "unorganised worker": "laborer",
  "unorganized worker": "laborer",
  "daily wage worker": "laborer",
  "construction worker": "laborer",

  "self employed": "self_employed",
  "self-employed": "self_employed",
  self_employed: "self_employed",
  business: "self_employed",
  "small business owner": "self_employed",
  entrepreneur: "self_employed",

  salaried: "salaried",
  employee: "salaried",
  "private employee": "salaried",
  "government employee": "salaried",
  service: "salaried",

  unemployed: "unemployed",
  jobless: "unemployed",
  "not employed": "unemployed",

  student: "student",

  homemaker: "homemaker",
  housewife: "homemaker",
  "home maker": "homemaker",

  retired: "retired",
  pensioner: "retired",
};

const CATEGORY_SYNONYMS: Record<string, string> = {
  general: "general",
  gen: "general",
  unreserved: "general",

  sc: "sc",
  "scheduled caste": "sc",

  st: "st",
  "scheduled tribe": "st",

  obc: "obc",
  "other backward class": "obc",
  "other backward classes": "obc",

  minority: "minority",
};

function normalize(raw: string | null | undefined, table: Record<string, string>): string | null {
  if (!raw) return null;
  const key = raw.trim().toLowerCase();
  return table[key] ?? null;
}

export function normalizeOccupation(raw: string | null | undefined): string | null {
  return normalize(raw, OCCUPATION_SYNONYMS);
}

export function normalizeCategory(raw: string | null | undefined): string | null {
  return normalize(raw, CATEGORY_SYNONYMS);
}
