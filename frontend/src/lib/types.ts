// Hand-mirrored from labhsathi-core::schemes::{UserProfile, SchemeMatch} and
// labhsathi-core::status::JobStatusRecord. Codegen (ts-rs/specta) would
// remove the drift risk but is a stretch goal, not required for the
// hackathon build -- keep these in sync by hand when the Rust side changes.

export interface UserProfile {
  age: number;
  annual_income: number;
  occupation: string;
  state: string;
  gender: string;
  has_disability: boolean;
  disability_percentage: number | null;
  land_holding_acres: number | null;
  family_size: number;
  is_widow: boolean;
  category: string;
  is_student: boolean;
  has_bank_account: boolean;
  has_kutcha_house: boolean;
  is_pregnant_or_lactating_first_child: boolean;
  girl_child_age: number | null;
  area_type: string | null; // "urban" | "rural", if known -- routes PMAY to the urban/rural program
}

export type MatchStatus = "match" | "needs_info";

export interface SchemeMatch {
  id: string;
  name: string;
  authority: string;
  benefit: string;
  status: MatchStatus;
  reason: string;
  /** Stable UserProfile field key naming what would resolve a needs_info
   * card (e.g. "land_holding_acres") -- null for a real match. */
  missing_field: string | null;
  documents: string[];
  official_note: string;
  source_url: string | null;
  category_domain?: string | null;
}

export interface MatchResponse {
  matched_count: number;
  schemes: SchemeMatch[];
}

export interface ExtractedFields {
  age: number | null;
  annual_income: number | null;
  state: string | null;
  category: string | null;
  land_holding_acres: number | null;
  occupation: string | null;
}

export type JobProgress = "queued" | "processing" | "done" | "failed";

export interface JobStatusRecord {
  status: JobProgress;
  fields: ExtractedFields | null;
  error: string | null;
}

export const CATEGORY_LABELS: Record<string, string> = {
  agriculture: "Agriculture & Farmers",
  healthcare: "Health & Wellness",
  pensions: "Pensions & Social Security",
  education: "Education & Scholarships",
  business: "Business & Artisans",
  housing: "Housing & Shelter",
  women_child: "Women & Child",
  employment: "Employment & Livelihood",
  disability: "Disability Support",
  financial_inclusion: "Financial Inclusion",
};
