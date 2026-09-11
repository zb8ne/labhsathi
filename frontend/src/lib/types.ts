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
}

export interface SchemeMatch {
  id: string;
  name: string;
  authority: string;
  benefit: string;
  reason: string;
  documents: string[];
  official_note: string;
  source_url: string | null;
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
