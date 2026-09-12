use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Deserialize, Clone)]
pub struct UserProfile {
    pub age: u32,
    pub annual_income: u64, // household annual income in INR
    pub occupation: String, // "farmer" | "laborer" | "unemployed" | "student" | "self_employed" | "salaried" | "homemaker" | "retired"
    pub state: String,
    pub gender: String, // "male" | "female" | "other"
    pub has_disability: bool,
    pub disability_percentage: Option<u8>,
    pub land_holding_acres: Option<f64>,
    pub family_size: u32,
    pub is_widow: bool,
    pub category: String, // "general" | "sc" | "st" | "obc" | "minority"
    pub is_student: bool,
    pub has_bank_account: bool,
    pub has_kutcha_house: bool, // lives in a non-pucca / temporary dwelling
    pub is_pregnant_or_lactating_first_child: bool,
    pub girl_child_age: Option<u32>, // age of daughter, if applicable
    pub area_type: Option<String>, // "urban" | "rural", if known -- routes PMAY to the urban/rural program
}

/// A scheme can genuinely not match, genuinely match, or -- the state the
/// old binary Option<String> rule silently collapsed into "doesn't match"
/// -- have a real chance depending on an answer we don't have yet (a
/// farmer who hasn't entered a land holding shouldn't silently lose
/// PM-KISAN). `NoMatch` isn't a wire variant: those schemes are dropped
/// before the response is built, same as before.
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Match,
    NeedsInfo,
}

#[derive(Debug, Serialize, Clone)]
pub struct SchemeMatch {
    pub id: String,
    pub name: String,
    pub authority: String,
    pub benefit: String,
    pub status: MatchStatus,
    pub reason: String,
    /// Stable key naming the UserProfile field that would resolve this --
    /// only set when status is needs_info. The frontend maps this to the
    /// actual form control to highlight/scroll to, so it stays a machine
    /// key here rather than free text.
    pub missing_field: Option<&'static str>,
    pub documents: Vec<String>,
    pub official_note: String,
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_domain: Option<String>,
}

/// Declarative eligibility criteria that can be specified directly in `data/schemes.json`
/// without requiring compiled Rust code changes for new schemes. Serialize is needed here
/// (not just Deserialize) because catalog-service re-emits this same struct as its HTTP
/// response body -- it reads the row out of Postgres and hands it back out verbatim.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SchemeCriteria {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_age: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_annual_income: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_annual_income: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occupations: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_disability: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_disability_percentage: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_land: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_widow: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_student: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_no_bank_account: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_kutcha_house: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_pregnant_or_lactating: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_daughter_age: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_reason: Option<String>,
}

/// The catalog half of the scheme-data split: facts, documentation,
/// citation links, category domains, and optional declarative criteria.
/// This is the exact shape catalog-service stores in Postgres (as a single
/// JSONB column per row, keyed by `id`) and serves back over HTTP -- both
/// it and api-gateway depend on this crate specifically so there's one
/// definition of "what a scheme is," not a hand-kept-in-sync copy on each
/// side of the wire.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchemeFacts {
    pub id: String,
    pub name: String,
    pub authority: String,
    pub benefit: String,
    pub documents: Vec<String>,
    pub official_note: String,
    pub source_url: Option<String>,
    #[serde(default)]
    pub category_domain: Option<String>,
    #[serde(default)]
    pub criteria: Option<SchemeCriteria>,
    pub review_date: String,
    pub exclusions: Vec<String>,
}

/// The originally-embedded catalog -- now used as catalog-service's one-time
/// seed source for Postgres (see services/catalog-service) and as the fixture
/// this crate's own tests run against, rather than as the runtime data path.
/// api-gateway no longer reads this directly; it fetches from catalog-service.
const SEED_SCHEMES_JSON: &str = include_str!("../data/schemes.json");

pub fn seed_facts() -> Vec<SchemeFacts> {
    serde_json::from_str(SEED_SCHEMES_JSON).expect("data/schemes.json is checked in and must parse")
}

#[derive(Debug, PartialEq, Eq)]
enum RawStatus {
    Match,
    NeedsInfo,
    NoMatch,
}

struct RuleOutcome {
    status: RawStatus,
    reason: String,
    missing_field: Option<&'static str>,
}

fn matched(reason: impl Into<String>) -> RuleOutcome {
    RuleOutcome { status: RawStatus::Match, reason: reason.into(), missing_field: None }
}
fn not_matched(reason: impl Into<String>) -> RuleOutcome {
    RuleOutcome { status: RawStatus::NoMatch, reason: reason.into(), missing_field: None }
}
fn needs_info(reason: impl Into<String>, field: &'static str) -> RuleOutcome {
    RuleOutcome { status: RawStatus::NeedsInfo, reason: reason.into(), missing_field: Some(field) }
}

/// Evaluates declarative criteria from data/schemes.json against a UserProfile.
fn evaluate_criteria(f: &SchemeFacts, c: &SchemeCriteria, p: &UserProfile) -> RuleOutcome {
    // 1. Gender check
    if let Some(ref g) = c.gender {
        if &p.gender != g {
            return not_matched(format!("Requires applicant gender '{g}'."));
        }
    }

    // 2. First pregnancy / lactation check
    if let Some(req) = c.is_pregnant_or_lactating {
        if req && !p.is_pregnant_or_lactating_first_child {
            return not_matched("Requires first pregnancy or lactation.");
        }
    }

    // 3. Widow check
    if let Some(req) = c.is_widow {
        if req && !p.is_widow {
            return not_matched("Applies specifically to widows.");
        }
    }

    // 4. Student check
    if let Some(req) = c.is_student {
        if req && !p.is_student {
            return not_matched("Requires enrolled student status.");
        }
    }

    // 5. Bank account check
    if let Some(req_no_bank) = c.requires_no_bank_account {
        if req_no_bank && p.has_bank_account {
            return not_matched("Already reported having a bank account.");
        }
    }

    // 6. Occupations check
    if let Some(ref occs) = c.occupations {
        if !occs.iter().any(|o| o == &p.occupation) {
            return not_matched(format!("Occupation '{}' does not match eligible occupations.", p.occupation));
        }
    }

    // 7. Social categories check
    if let Some(ref cats) = c.categories {
        if !cats.iter().any(|cat| cat == &p.category) {
            return not_matched(format!("Category '{}' is not among eligible categories.", p.category));
        }
    }

    // 8. Age check (inclusive)
    if let Some(min) = c.min_age {
        if p.age < min {
            return not_matched(format!("Age {} is below minimum age threshold of {}.", p.age, min));
        }
    }
    if let Some(max) = c.max_age {
        if p.age > max {
            return not_matched(format!("Age {} is above maximum age limit of {}.", p.age, max));
        }
    }

    // 9. Income check
    if let Some(max_inc) = c.max_annual_income {
        if p.annual_income >= max_inc {
            return not_matched(format!("Household income ₹{} is at or above the cap of ₹{}.", p.annual_income, max_inc));
        }
    }
    if let Some(min_inc) = c.min_annual_income {
        if p.annual_income < min_inc {
            return not_matched(format!("Income ₹{} is below minimum threshold of ₹{}.", p.annual_income, min_inc));
        }
    }

    // 10. Land holding check (with NeedsInfo)
    if let Some(req_land) = c.requires_land {
        if req_land {
            match p.land_holding_acres {
                None => {
                    return needs_info(
                        "You reported farming as your occupation: tell us your land holding to check eligibility.",
                        "land_holding_acres",
                    );
                }
                Some(acres) if acres > 0.0 => {}
                Some(_) => return not_matched("Reported land holding is zero."),
            }
        }
    }

    // 11. Disability check (with NeedsInfo)
    if let Some(req_disability) = c.requires_disability {
        if req_disability && !p.has_disability {
            return not_matched("No disability reported.");
        }
    }
    if let Some(min_pct) = c.min_disability_percentage {
        if !p.has_disability {
            return not_matched("No disability reported.");
        }
        match p.disability_percentage {
            None => {
                return needs_info(
                    "You reported a disability: tell us the certified percentage to check eligibility.",
                    "disability_percentage",
                );
            }
            Some(pct) if pct >= min_pct => {}
            Some(pct) => return not_matched(format!("Certified disability ({}%) is below the required {}%.", pct, min_pct)),
        }
    }

    // 12. Kutcha house & area type check (with NeedsInfo)
    if let Some(req_kutcha) = c.requires_kutcha_house {
        if req_kutcha && !p.has_kutcha_house {
            return not_matched("Does not report a kutcha/temporary house.");
        }
    }
    if let Some(ref target_area) = c.area_type {
        if p.has_kutcha_house {
            match p.area_type.as_deref() {
                None => {
                    return needs_info(
                        "Tell us whether your residence is in an urban or rural area to route you to the right program.",
                        "area_type",
                    );
                }
                Some(a) if a == target_area => {}
                Some(_) => return not_matched(format!("Reported area type does not match {target_area}.")),
            }
        }
    }

    // 13. Daughter age check
    if let Some(max_daughter_age) = c.max_daughter_age {
        match p.girl_child_age {
            Some(age) if age < max_daughter_age => {}
            _ => return not_matched(format!("Requires a daughter under age {max_daughter_age}.")),
        }
    }

    // 14. State check
    if let Some(ref states) = c.states {
        if !states.is_empty() && !states.iter().any(|s| s.eq_ignore_ascii_case(&p.state)) {
            return not_matched(format!("Scheme applies to {} rather than {}.", states.join(", "), p.state));
        }
    }

    let reason = c.match_reason.clone().unwrap_or_else(|| {
        format!("Your reported profile matches the eligibility criteria for {}.", f.name)
    });

    matched(reason)
}

// NOTE ON ACCURACY: Eligibility rules below are simplified approximations of
// real central-government scheme criteria for prototype/demo purposes. Exact
// eligibility always depends on the latest official notification: the app
// surfaces these schemes as *candidates worth checking*, not a final
// determination. See README for sources used to build this table.
fn evaluate_rule(f: &SchemeFacts, p: &UserProfile) -> RuleOutcome {
    let id = f.id.as_str();
    match id {
        "pm-kisan" => {
            if p.occupation != "farmer" {
                return not_matched("Occupation isn't reported as farming.");
            }
            match p.land_holding_acres {
                None => needs_info(
                    "You reported farming as your occupation: tell us your land holding to check PM-KISAN eligibility.",
                    "land_holding_acres",
                ),
                Some(acres) if acres > 0.0 => matched(
                    "You reported farming as your occupation with land holdings: PM-KISAN provides direct income support to landholding farmer families.",
                ),
                Some(_) => not_matched("Reported land holding is zero."),
            }
        }

        "ayushman-bharat" => {
            if p.annual_income < 250_000 {
                matched("Your household income falls in the low-income band PM-JAY targets: worth checking your SECC beneficiary status.")
            } else {
                not_matched("Household income is at or above the ₹2.5L band this prototype checks.")
            }
        }

        "pmjdy" => {
            if !p.has_bank_account {
                matched("You reported not having a bank account: PM Jan Dhan Yojana gives you one with zero balance requirement plus insurance cover.")
            } else {
                not_matched("Already reported having a bank account.")
            }
        }

        "nsap-ignoaps" => {
            if p.age >= 60 && p.annual_income < 100_000 {
                matched("You're 60+ with household income below the BPL-linked threshold this scheme targets.")
            } else {
                not_matched("Doesn't meet the age and/or income threshold.")
            }
        }

        "nsap-ignwps" => {
            if p.is_widow && p.age >= 40 && p.age <= 79 && p.annual_income < 100_000 {
                matched("You reported being a widow in the 40-79 age band with income below the BPL-linked threshold this scheme targets.")
            } else {
                not_matched("Doesn't meet the widow/age/income criteria.")
            }
        }

        "nsap-igndps" => {
            if !p.has_disability {
                return not_matched("No disability reported.");
            }
            match p.disability_percentage {
                None => needs_info(
                    "You reported a disability: tell us the certified percentage to check IGNDPS eligibility.",
                    "disability_percentage",
                ),
                Some(pct) if pct >= 80 && p.age >= 18 && p.age <= 79 && p.annual_income < 100_000 => {
                    matched("Your reported disability level (80%+), age band, and income match IGNDPS criteria.")
                }
                Some(_) => not_matched("Disability percentage, age band, or income doesn't meet IGNDPS criteria."),
            }
        }

        "nsp-scholarship" => {
            if p.is_student && p.category != "general" && p.annual_income < 250_000 {
                matched(format!(
                    "You're a student from the {} category with household income under ₹2.5L: several NSP scholarships target exactly this.",
                    p.category.to_uppercase()
                ))
            } else {
                not_matched("Not a student, general category, or income above the cap.")
            }
        }

        "sukanya-samriddhi" => {
            // No separate "do you have a daughter" signal exists in the
            // profile -- girl_child_age doubles as both existence and age.
            // Treating "unset" as needs-info here would surface this
            // scheme to every household regardless of relevance, which
            // isn't what the missing-question UI is for. Revisit once the
            // household-member model (deferred, own increment) exists.
            match p.girl_child_age {
                Some(age) if age < 10 => matched(
                    "You have a daughter under 10: this scheme locks in a government-backed high interest rate for her future.",
                ),
                _ => not_matched("No daughter under 10 reported."),
            }
        }

        "pmay-urban" | "pmay-rural" => {
            let wants = if id == "pmay-urban" { "urban" } else { "rural" };
            if !p.has_kutcha_house || p.annual_income >= 300_000 {
                return not_matched("Doesn't report a kutcha house under the income cap this prototype checks.");
            }
            match p.area_type.as_deref() {
                None => needs_info(
                    "You reported a kutcha house under ₹3L income: tell us whether that's in an urban or rural area to route you to the right PMAY program.",
                    "area_type",
                ),
                Some(a) if a == wants => matched(format!(
                    "You reported living in a kutcha/temporary house in a {wants} area with income under ₹3L: {} funds pucca house construction for exactly this profile.",
                    if wants == "urban" { "PMAY-Urban" } else { "PMAY-Gramin" }
                )),
                Some(_) => not_matched(format!("Reported area type doesn't match {wants}.")),
            }
        }

        "pmmvy" => {
            if p.gender == "female" && p.is_pregnant_or_lactating_first_child {
                matched("You indicated a first pregnancy/lactation: PMMVY provides direct cash support tied to your checkups.")
            } else {
                not_matched("Gender or first-pregnancy/lactation flag doesn't match.")
            }
        }

        "pm-sym" => {
            let unorganised = matches!(p.occupation.as_str(), "laborer" | "self_employed" | "homemaker");
            if unorganised && p.age >= 18 && p.age <= 40 && p.annual_income < 180_000 {
                matched("You're in the 18–40 age band, working in the unorganised sector, within the income cap: this builds you a pension for later.")
            } else {
                not_matched("Occupation, age band, or income doesn't meet PM-SYM criteria.")
            }
        }

        // For any other scheme in data/schemes.json, evaluate declarative criteria:
        _ => {
            if let Some(ref c) = f.criteria {
                evaluate_criteria(f, c, p)
            } else {
                unreachable!("data/schemes.json declares scheme id `{id}` with neither a compiled rule arm nor declarative criteria")
            }
        }
    }
}

/// `facts` is supplied by the caller rather than loaded internally -- this
/// crate has no reqwest/sqlx/axum dependency (see docs/CONVENTIONS.md) and
/// deliberately doesn't know or care whether the caller got its catalog
/// from an embedded JSON file (tests, catalog-service's own seed step) or
/// a live HTTP call to catalog-service (api-gateway, production). The
/// matching logic itself is identical either way.
pub fn match_schemes(profile: &UserProfile, facts: &[SchemeFacts]) -> Vec<SchemeMatch> {
    let mut seen_missing_fields: HashSet<&'static str> = HashSet::new();

    facts
        .iter()
        .filter_map(|f| {
            let outcome = evaluate_rule(f, profile);
            let status = match outcome.status {
                RawStatus::Match => MatchStatus::Match,
                RawStatus::NeedsInfo => {
                    // pmay-urban and pmay-rural can both land here asking
                    // the identical follow-up question -- only surface it
                    // once rather than showing the same prompt twice.
                    if let Some(field) = outcome.missing_field {
                        if !seen_missing_fields.insert(field) {
                            return None;
                        }
                    }
                    MatchStatus::NeedsInfo
                }
                RawStatus::NoMatch => return None,
            };
            Some(SchemeMatch {
                id: f.id.clone(),
                name: f.name.clone(),
                authority: f.authority.clone(),
                benefit: f.benefit.clone(),
                status,
                reason: outcome.reason,
                missing_field: outcome.missing_field,
                documents: f.documents.clone(),
                official_note: f.official_note.clone(),
                source_url: f.source_url.clone(),
                category_domain: f.category_domain.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Matches nothing by construction -- every test below starts here and
    /// flips only the fields relevant to the one scheme under test, so a
    /// match can only be attributed to the field actually being tested.
    fn base_profile() -> UserProfile {
        UserProfile {
            age: 30,
            annual_income: 1_000_000,
            occupation: "salaried".into(),
            state: "Goa".into(),
            gender: "male".into(),
            has_disability: false,
            disability_percentage: None,
            land_holding_acres: None,
            family_size: 4,
            is_widow: false,
            category: "general".into(),
            is_student: false,
            has_bank_account: true,
            has_kutcha_house: false,
            is_pregnant_or_lactating_first_child: false,
            girl_child_age: None,
            area_type: None,
        }
    }

    // Parsed once for the whole test run -- match_schemes takes facts by
    // reference now (it no longer loads them itself), so every test needs
    // a catalog to hand it; this is the same seed data catalog-service
    // loads into Postgres on first boot.
    static TEST_FACTS: std::sync::LazyLock<Vec<SchemeFacts>> = std::sync::LazyLock::new(seed_facts);

    fn matches(p: &UserProfile) -> Vec<SchemeMatch> {
        match_schemes(p, &TEST_FACTS)
    }

    fn has(matches: &[SchemeMatch], id: &str) -> bool {
        matches.iter().any(|m| m.id == id)
    }

    fn find<'a>(matches: &'a [SchemeMatch], id: &str) -> &'a SchemeMatch {
        matches.iter().find(|m| m.id == id).unwrap_or_else(|| panic!("expected `{id}` in the response, got {matches:?}"))
    }

    #[test]
    fn base_profile_matches_only_genuinely_universal_schemes() {
        // base_profile is a comfortable, well-off salaried adult with no
        // special circumstances -- deliberately built to rule out every
        // means-tested or vulnerability-targeted scheme in the catalog.
        // It's no longer expected to match *zero* schemes now that the
        // catalog also includes formal-sector and universal-eligibility
        // programs that a profile like this genuinely does qualify for:
        // NPCDCS screens any adult 30+ regardless of income, and NPS's
        // all-citizen model is open to any citizen 18-70. Matching those
        // two is correct, not a leak -- the assertion below is deliberately
        // an explicit allowlist, not a raised count, so any *other* scheme
        // matching this profile still fails the test loudly.
        let ids: Vec<String> = matches(&base_profile()).into_iter().map(|m| m.id).collect();
        assert_eq!(ids.len(), 2, "unexpected match(es) for a profile designed to qualify for nothing means-tested: {ids:?}");
        assert!(ids.iter().any(|id| id == "npcdcs-ncd-screening"));
        assert!(ids.iter().any(|id| id == "nps-all-citizen"));
    }

    #[test]
    fn pm_kisan_requires_farming_occupation_and_land_holding() {
        let mut p = base_profile();
        p.occupation = "farmer".into();
        p.land_holding_acres = Some(2.0);
        let m = matches(&p);
        assert!(has(&m, "pm-kisan"));
        assert_eq!(find(&m, "pm-kisan").status, MatchStatus::Match);

        // boundary: the rule is `> 0.0`, not `>= 0.0`
        p.land_holding_acres = Some(0.0);
        assert!(!has(&matches(&p), "pm-kisan"));
    }

    #[test]
    fn pm_kisan_farmer_without_land_holding_needs_info_not_silent_rejection() {
        // The bug this replaces: land_holding_acres.unwrap_or(0.0) used to
        // make an unanswered field indistinguishable from "zero land," so
        // a farmer who just hadn't filled it in silently lost PM-KISAN.
        let mut p = base_profile();
        p.occupation = "farmer".into();
        p.land_holding_acres = None;
        let m = matches(&p);
        let entry = find(&m, "pm-kisan");
        assert_eq!(entry.status, MatchStatus::NeedsInfo);
        assert_eq!(entry.missing_field, Some("land_holding_acres"));
    }

    #[test]
    fn ayushman_bharat_income_boundary_is_exclusive() {
        let mut p = base_profile();
        p.annual_income = 249_999;
        assert!(has(&matches(&p), "ayushman-bharat"));

        p.annual_income = 250_000; // rule is `< 250_000`
        assert!(!has(&matches(&p), "ayushman-bharat"));
    }

    #[test]
    fn pmjdy_matches_only_without_a_bank_account() {
        let mut p = base_profile();
        p.has_bank_account = false;
        assert!(has(&matches(&p), "pmjdy"));

        p.has_bank_account = true;
        assert!(!has(&matches(&p), "pmjdy"));
    }

    #[test]
    fn ignoaps_age_boundary_is_inclusive_at_60() {
        let mut p = base_profile();
        p.age = 60;
        p.annual_income = 99_999;
        assert!(has(&matches(&p), "nsap-ignoaps"));

        p.age = 59;
        assert!(!has(&matches(&p), "nsap-ignoaps"));
    }

    #[test]
    fn ignwps_requires_widow_and_the_40_to_79_age_band() {
        let mut p = base_profile();
        p.is_widow = true;
        p.annual_income = 99_999;
        p.age = 40;
        assert!(has(&matches(&p), "nsap-ignwps"));
        p.age = 79;
        assert!(has(&matches(&p), "nsap-ignwps"));

        p.age = 39;
        assert!(!has(&matches(&p), "nsap-ignwps"), "below the band");
        p.age = 80;
        assert!(!has(&matches(&p), "nsap-ignwps"), "above the band");

        p.age = 50;
        p.is_widow = false;
        assert!(!has(&matches(&p), "nsap-ignwps"), "not a widow");
    }

    #[test]
    fn igndps_requires_80_percent_disability_in_the_18_to_79_age_band() {
        let mut p = base_profile();
        p.has_disability = true;
        p.disability_percentage = Some(80);
        p.age = 40;
        p.annual_income = 99_999;
        let m = matches(&p);
        assert!(has(&m, "nsap-igndps"));
        assert_eq!(find(&m, "nsap-igndps").status, MatchStatus::Match);

        // boundary: the rule is `>= 80`
        p.disability_percentage = Some(79);
        assert!(!has(&matches(&p), "nsap-igndps"));
    }

    #[test]
    fn igndps_disability_without_percentage_needs_info() {
        let mut p = base_profile();
        p.has_disability = true;
        p.disability_percentage = None;
        let m = matches(&p);
        let entry = find(&m, "nsap-igndps");
        assert_eq!(entry.status, MatchStatus::NeedsInfo);
        assert_eq!(entry.missing_field, Some("disability_percentage"));
    }

    #[test]
    fn igndps_no_disability_reported_is_not_needs_info() {
        // has_disability is a checkbox, not an optional field -- left
        // unchecked means "no," not "unknown," so this must stay a plain
        // non-match rather than asking a percentage question nobody needs.
        let p = base_profile();
        assert!(!has(&matches(&p), "nsap-igndps"));
    }

    #[test]
    fn nsp_scholarship_requires_non_general_category_and_income_under_2_5l() {
        let mut p = base_profile();
        p.is_student = true;
        p.category = "obc".into();
        p.annual_income = 249_999;
        assert!(has(&matches(&p), "nsp-scholarship"));

        p.category = "general".into();
        assert!(!has(&matches(&p), "nsp-scholarship"), "category must not be general");
    }

    #[test]
    fn sukanya_samriddhi_daughter_age_boundary_is_exclusive_at_10() {
        let mut p = base_profile();
        p.girl_child_age = Some(9);
        assert!(has(&matches(&p), "sukanya-samriddhi"));

        p.girl_child_age = Some(10); // rule is `< 10`
        assert!(!has(&matches(&p), "sukanya-samriddhi"));
    }

    #[test]
    fn sukanya_samriddhi_unset_age_is_not_needs_info() {
        // Deliberately not surfaced as needs-info -- see the comment on
        // this rule arm for why (no existence signal separate from age).
        let p = base_profile();
        assert!(!has(&matches(&p), "sukanya-samriddhi"));
    }

    #[test]
    fn pmay_splits_into_distinct_urban_and_rural_programs() {
        let mut p = base_profile();
        p.has_kutcha_house = true;
        p.annual_income = 299_999;

        p.area_type = Some("urban".into());
        let m = matches(&p);
        assert!(has(&m, "pmay-urban"));
        assert!(!has(&m, "pmay-rural"));
        assert_eq!(find(&m, "pmay-urban").status, MatchStatus::Match);

        p.area_type = Some("rural".into());
        let m = matches(&p);
        assert!(has(&m, "pmay-rural"));
        assert!(!has(&m, "pmay-urban"));

        p.annual_income = 300_000; // rule is `< 300_000`
        assert!(!has(&matches(&p), "pmay-rural"));
    }

    #[test]
    fn pmay_unknown_area_type_asks_once_not_twice() {
        let mut p = base_profile();
        p.has_kutcha_house = true;
        p.annual_income = 299_999;
        p.area_type = None;

        let m = matches(&p);
        let pmay_entries: Vec<_> = m.iter().filter(|s| s.id.starts_with("pmay-")).collect();
        assert_eq!(pmay_entries.len(), 1, "urban and rural share the same missing question -- must not show it twice");
        assert_eq!(pmay_entries[0].status, MatchStatus::NeedsInfo);
        assert_eq!(pmay_entries[0].missing_field, Some("area_type"));
    }

    #[test]
    fn pmmvy_requires_female_gender_not_just_the_pregnancy_flag() {
        let mut p = base_profile();
        p.gender = "female".into();
        p.is_pregnant_or_lactating_first_child = true;
        assert!(has(&matches(&p), "pmmvy"));

        p.gender = "male".into();
        assert!(!has(&matches(&p), "pmmvy"), "gender gates this rule, not just the flag");
    }

    #[test]
    fn pm_sym_requires_unorganised_occupation_18_to_40_age_band_and_income_under_1_8l() {
        let mut p = base_profile();
        p.occupation = "laborer".into();
        p.age = 40;
        p.annual_income = 179_999;
        assert!(has(&matches(&p), "pm-sym"));

        p.age = 41; // rule is `<= 40`
        assert!(!has(&matches(&p), "pm-sym"));
    }

    #[test]
    fn multiple_independent_matches_dont_interfere_with_each_other() {
        let mut p = base_profile();
        p.occupation = "farmer".into();
        p.land_holding_acres = Some(1.0);
        p.has_bank_account = false;
        p.annual_income = 50_000;
        let m = matches(&p);
        assert!(has(&m, "pm-kisan"));
        assert!(has(&m, "pmjdy"));
        assert!(has(&m, "ayushman-bharat"));
    }

    #[test]
    fn every_catalog_entry_has_a_rule_arm() {
        // evaluate_rule panics on an unknown id with no criteria -- this test forces the
        // panic to surface at test time (for every entry, not just
        // whichever ones happen to match a specific test's profile)
        // instead of a live 500 the first time someone adds a scheme to
        // schemes.json without a matching Rust arm or declarative criteria.
        let p = base_profile();
        for f in TEST_FACTS.iter() {
            let _ = evaluate_rule(f, &p);
        }
    }

    #[test]
    fn declarative_criteria_matches_profile() {
        let facts = SchemeFacts {
            id: "test-scheme".into(),
            name: "Test Scheme".into(),
            authority: "Ministry of Test".into(),
            benefit: "₹10,000 grant".into(),
            documents: vec!["Aadhaar".into()],
            official_note: "Note".into(),
            source_url: None,
            category_domain: Some("employment".into()),
            criteria: Some(SchemeCriteria {
                min_age: Some(18),
                max_age: Some(35),
                max_annual_income: Some(200_000),
                occupations: Some(vec!["unemployed".into()]),
                categories: Some(vec!["sc".into(), "st".into(), "obc".into()]),
                gender: None,
                requires_disability: None,
                min_disability_percentage: None,
                requires_land: None,
                is_widow: None,
                is_student: None,
                requires_no_bank_account: None,
                requires_kutcha_house: None,
                is_pregnant_or_lactating: None,
                max_daughter_age: None,
                area_type: None,
                states: None,
                match_reason: Some("You match the youth skill training criteria.".into()),
                ..Default::default()
            }),
            review_date: "2026-09-12".into(),
            exclusions: vec![],
        };

        let mut p = base_profile();
        p.age = 22;
        p.occupation = "unemployed".into();
        p.category = "obc".into();
        p.annual_income = 150_000;

        let outcome = evaluate_rule(&facts, &p);
        assert_eq!(outcome.status, RawStatus::Match);
        assert_eq!(outcome.reason, "You match the youth skill training criteria.");

        // Boundary: age exceeds limit
        p.age = 36;
        let outcome2 = evaluate_rule(&facts, &p);
        assert_eq!(outcome2.status, RawStatus::NoMatch);
    }
}

