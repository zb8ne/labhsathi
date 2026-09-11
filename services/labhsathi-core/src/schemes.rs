use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Clone)]
pub struct SchemeMatch {
    pub id: &'static str,
    pub name: &'static str,
    pub authority: &'static str,
    pub benefit: &'static str,
    pub reason: String,
    pub documents: Vec<&'static str>,
    pub official_note: &'static str,
}

type RuleFn = fn(&UserProfile) -> Option<String>;

struct Scheme {
    id: &'static str,
    name: &'static str,
    authority: &'static str,
    benefit: &'static str,
    documents: &'static [&'static str],
    official_note: &'static str,
    rule: RuleFn,
}

// NOTE ON ACCURACY: Eligibility rules below are simplified approximations of
// real central-government scheme criteria for prototype/demo purposes. Exact
// eligibility always depends on the latest official notification — the app
// surfaces this schemes as *candidates worth checking*, not a final
// determination. See README for sources used to build this table.
const SCHEMES: &[Scheme] = &[
    Scheme {
        id: "pm-kisan",
        name: "PM-KISAN (Pradhan Mantri Kisan Samman Nidhi)",
        authority: "Ministry of Agriculture & Farmers Welfare",
        benefit: "₹6,000/year direct income support in 3 installments",
        documents: &["Aadhaar card", "Land ownership records (khatauni/khasra)", "Bank passbook", "Passport-size photo"],
        official_note: "Excludes institutional land holders and certain higher-income categories (e.g. income-tax payers, government employees) per official notification.",
        rule: |p| {
            if p.occupation == "farmer" && p.land_holding_acres.unwrap_or(0.0) > 0.0 {
                Some("You reported farming as your occupation with land holdings — PM-KISAN provides direct income support to landholding farmer families.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "ayushman-bharat",
        name: "Ayushman Bharat PM-JAY",
        authority: "National Health Authority",
        benefit: "₹5,00,000/family/year cashless health insurance",
        documents: &["Aadhaar card", "Ration card / SECC household ID", "Income certificate"],
        official_note: "Actual eligibility is based on SECC 2011 deprivation/occupational criteria, not income alone — verify your household on the official PM-JAY beneficiary portal.",
        rule: |p| {
            if p.annual_income < 250_000 {
                Some("Your household income falls in the low-income band PM-JAY targets — worth checking your SECC beneficiary status.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "pmjdy",
        name: "Pradhan Mantri Jan Dhan Yojana",
        authority: "Department of Financial Services",
        benefit: "Zero-balance bank account, RuPay debit card, accident & life insurance cover",
        documents: &["Aadhaar card", "Address proof (if no Aadhaar)"],
        official_note: "Open to any Indian resident; no income or occupation restriction.",
        rule: |p| {
            if !p.has_bank_account {
                Some("You reported not having a bank account — PM Jan Dhan Yojana gives you one with zero balance requirement plus insurance cover.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "nsap-ignoaps",
        name: "National Old Age Pension (IGNOAPS)",
        authority: "Ministry of Rural Development (NSAP)",
        benefit: "Monthly pension (₹200–1000+, state top-ups vary)",
        documents: &["Aadhaar card", "Age proof", "BPL / income certificate", "Bank passbook"],
        official_note: "State governments top up the central amount; exact amount varies by state.",
        rule: |p| {
            if p.age >= 60 && p.annual_income < 100_000 {
                Some("You're 60+ with household income below the BPL-linked threshold this scheme targets.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "nsap-ignwps",
        name: "National Widow Pension (IGNWPS)",
        authority: "Ministry of Rural Development (NSAP)",
        benefit: "Monthly pension (₹300+, state top-ups vary)",
        documents: &["Aadhaar card", "Age proof", "Husband's death certificate", "BPL / income certificate", "Bank passbook"],
        official_note: "Covers widows aged 40-79 in most states; some states transition beneficiaries to IGNOAPS at 60 instead -- check your state's rule.",
        rule: |p| {
            if p.is_widow && p.age >= 40 && p.age <= 79 && p.annual_income < 100_000 {
                Some("You reported being a widow in the 40-79 age band with income below the BPL-linked threshold this scheme targets.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "nsap-igndps",
        name: "National Disability Pension (IGNDPS)",
        authority: "Ministry of Rural Development (NSAP)",
        benefit: "Monthly disability pension",
        documents: &["Aadhaar card", "Disability certificate (UDID, ≥80%)", "Income certificate"],
        official_note: "Requires a certified disability of 80% or more from a competent medical authority.",
        rule: |p| {
            if p.has_disability
                && p.disability_percentage.unwrap_or(0) >= 80
                && p.age >= 18
                && p.age <= 79
                && p.annual_income < 100_000
            {
                Some("Your reported disability level (80%+), age band, and income match IGNDPS criteria.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "nsp-scholarship",
        name: "National Scholarship Portal — Pre/Post-Matric Scholarship",
        authority: "Ministry of Social Justice & Empowerment / Ministry of Minority Affairs",
        benefit: "Tuition fee reimbursement + maintenance allowance for students",
        documents: &["Aadhaar card", "Caste/community certificate", "Income certificate", "Previous year mark sheet", "Bank passbook"],
        official_note: "Separate schemes and income caps apply per category (SC/ST/OBC/Minority) — check the specific scheme's cutoff on scholarships.gov.in.",
        rule: |p| {
            if p.is_student
                && p.category != "general"
                && p.annual_income < 250_000
            {
                Some(format!(
                    "You're a student from the {} category with household income under ₹2.5L — several NSP scholarships target exactly this.",
                    p.category.to_uppercase()
                ))
            } else {
                None
            }
        },
    },
    Scheme {
        id: "sukanya-samriddhi",
        name: "Sukanya Samriddhi Yojana",
        authority: "Ministry of Finance",
        benefit: "High-interest savings account for a girl child's education/marriage",
        documents: &["Girl child's birth certificate", "Guardian's Aadhaar & PAN", "Address proof"],
        official_note: "Account must be opened before the girl turns 10.",
        rule: |p| {
            if let Some(age) = p.girl_child_age {
                if age < 10 {
                    return Some("You have a daughter under 10 — this scheme locks in a government-backed high interest rate for her future.".into());
                }
            }
            None
        },
    },
    Scheme {
        id: "pmay",
        name: "Pradhan Mantri Awas Yojana",
        authority: "Ministry of Housing & Urban Affairs / Rural Development",
        benefit: "Financial assistance / interest subsidy to build or buy a pucca house",
        documents: &["Aadhaar card", "Income certificate", "Land documents (if owned)", "Bank passbook"],
        official_note: "Urban (PMAY-U) and rural (PMAY-G) versions have different income slabs and application processes.",
        rule: |p| {
            if p.has_kutcha_house && p.annual_income < 300_000 {
                Some("You reported living in a kutcha/temporary house with income under ₹3L — PMAY funds pucca house construction for exactly this profile.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "pmmvy",
        name: "Pradhan Mantri Matru Vandana Yojana",
        authority: "Ministry of Women & Child Development",
        benefit: "₹5,000 cash benefit for pregnancy/lactation (first living child)",
        documents: &["Aadhaar card", "MCP card (Mother and Child Protection card)", "Bank passbook"],
        official_note: "Applies to the first living child; benefit is paid in installments tied to health checkups.",
        rule: |p| {
            if p.gender == "female" && p.is_pregnant_or_lactating_first_child {
                Some("You indicated a first pregnancy/lactation — PMMVY provides direct cash support tied to your checkups.".into())
            } else {
                None
            }
        },
    },
    Scheme {
        id: "pm-sym",
        name: "Pradhan Mantri Shram Yogi Maan-dhan",
        authority: "Ministry of Labour & Employment",
        benefit: "Monthly pension of ₹3,000 after age 60 (unorganised sector)",
        documents: &["Aadhaar card", "Bank passbook (with IFSC)", "Mobile number"],
        official_note: "For unorganised-sector workers aged 18–40 earning up to ₹15,000/month, not covered by EPFO/ESIC/NPS.",
        rule: |p| {
            let unorganised = matches!(p.occupation.as_str(), "laborer" | "self_employed" | "homemaker");
            if unorganised && p.age >= 18 && p.age <= 40 && p.annual_income < 180_000 {
                Some("You're in the 18–40 age band, working in the unorganised sector, within the income cap — this builds you a pension for later.".into())
            } else {
                None
            }
        },
    },
];

pub fn match_schemes(profile: &UserProfile) -> Vec<SchemeMatch> {
    SCHEMES
        .iter()
        .filter_map(|s| {
            (s.rule)(profile).map(|reason| SchemeMatch {
                id: s.id,
                name: s.name,
                authority: s.authority,
                benefit: s.benefit,
                reason,
                documents: s.documents.to_vec(),
                official_note: s.official_note,
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
        }
    }

    fn matched_ids(p: &UserProfile) -> Vec<&'static str> {
        match_schemes(p).into_iter().map(|m| m.id).collect()
    }

    #[test]
    fn base_profile_matches_nothing() {
        assert_eq!(matched_ids(&base_profile()), Vec::<&str>::new());
    }

    #[test]
    fn pm_kisan_requires_farming_occupation_and_land_holding() {
        let mut p = base_profile();
        p.occupation = "farmer".into();
        p.land_holding_acres = Some(2.0);
        assert!(matched_ids(&p).contains(&"pm-kisan"));

        // boundary: the rule is `> 0.0`, not `>= 0.0`
        p.land_holding_acres = Some(0.0);
        assert!(!matched_ids(&p).contains(&"pm-kisan"));
    }

    #[test]
    fn ayushman_bharat_income_boundary_is_exclusive() {
        let mut p = base_profile();
        p.annual_income = 249_999;
        assert!(matched_ids(&p).contains(&"ayushman-bharat"));

        p.annual_income = 250_000; // rule is `< 250_000`
        assert!(!matched_ids(&p).contains(&"ayushman-bharat"));
    }

    #[test]
    fn pmjdy_matches_only_without_a_bank_account() {
        let mut p = base_profile();
        p.has_bank_account = false;
        assert!(matched_ids(&p).contains(&"pmjdy"));

        p.has_bank_account = true;
        assert!(!matched_ids(&p).contains(&"pmjdy"));
    }

    #[test]
    fn ignoaps_age_boundary_is_inclusive_at_60() {
        let mut p = base_profile();
        p.age = 60;
        p.annual_income = 99_999;
        assert!(matched_ids(&p).contains(&"nsap-ignoaps"));

        p.age = 59;
        assert!(!matched_ids(&p).contains(&"nsap-ignoaps"));
    }

    #[test]
    fn ignwps_requires_widow_and_the_40_to_79_age_band() {
        let mut p = base_profile();
        p.is_widow = true;
        p.annual_income = 99_999;
        p.age = 40;
        assert!(matched_ids(&p).contains(&"nsap-ignwps"));
        p.age = 79;
        assert!(matched_ids(&p).contains(&"nsap-ignwps"));

        p.age = 39;
        assert!(!matched_ids(&p).contains(&"nsap-ignwps"), "below the band");
        p.age = 80;
        assert!(!matched_ids(&p).contains(&"nsap-ignwps"), "above the band");

        p.age = 50;
        p.is_widow = false;
        assert!(!matched_ids(&p).contains(&"nsap-ignwps"), "not a widow");
    }

    #[test]
    fn igndps_requires_80_percent_disability_in_the_18_to_79_age_band() {
        let mut p = base_profile();
        p.has_disability = true;
        p.disability_percentage = Some(80);
        p.age = 40;
        p.annual_income = 99_999;
        assert!(matched_ids(&p).contains(&"nsap-igndps"));

        // boundary: the rule is `>= 80`
        p.disability_percentage = Some(79);
        assert!(!matched_ids(&p).contains(&"nsap-igndps"));
    }

    #[test]
    fn nsp_scholarship_requires_non_general_category_and_income_under_2_5l() {
        let mut p = base_profile();
        p.is_student = true;
        p.category = "obc".into();
        p.annual_income = 249_999;
        assert!(matched_ids(&p).contains(&"nsp-scholarship"));

        p.category = "general".into();
        assert!(!matched_ids(&p).contains(&"nsp-scholarship"), "category must not be general");
    }

    #[test]
    fn sukanya_samriddhi_daughter_age_boundary_is_exclusive_at_10() {
        let mut p = base_profile();
        p.girl_child_age = Some(9);
        assert!(matched_ids(&p).contains(&"sukanya-samriddhi"));

        p.girl_child_age = Some(10); // rule is `< 10`
        assert!(!matched_ids(&p).contains(&"sukanya-samriddhi"));
    }

    #[test]
    fn pmay_requires_kutcha_house_and_income_under_3l() {
        let mut p = base_profile();
        p.has_kutcha_house = true;
        p.annual_income = 299_999;
        assert!(matched_ids(&p).contains(&"pmay"));

        p.annual_income = 300_000; // rule is `< 300_000`
        assert!(!matched_ids(&p).contains(&"pmay"));
    }

    #[test]
    fn pmmvy_requires_female_gender_not_just_the_pregnancy_flag() {
        let mut p = base_profile();
        p.gender = "female".into();
        p.is_pregnant_or_lactating_first_child = true;
        assert!(matched_ids(&p).contains(&"pmmvy"));

        p.gender = "male".into();
        assert!(!matched_ids(&p).contains(&"pmmvy"), "gender gates this rule, not just the flag");
    }

    #[test]
    fn pm_sym_requires_unorganised_occupation_18_to_40_age_band_and_income_under_1_8l() {
        let mut p = base_profile();
        p.occupation = "laborer".into();
        p.age = 40;
        p.annual_income = 179_999;
        assert!(matched_ids(&p).contains(&"pm-sym"));

        p.age = 41; // rule is `<= 40`
        assert!(!matched_ids(&p).contains(&"pm-sym"));
    }

    #[test]
    fn multiple_independent_matches_dont_interfere_with_each_other() {
        let mut p = base_profile();
        p.occupation = "farmer".into();
        p.land_holding_acres = Some(1.0);
        p.has_bank_account = false;
        p.annual_income = 50_000;
        let ids = matched_ids(&p);
        assert!(ids.contains(&"pm-kisan"));
        assert!(ids.contains(&"pmjdy"));
        assert!(ids.contains(&"ayushman-bharat"));
    }
}
