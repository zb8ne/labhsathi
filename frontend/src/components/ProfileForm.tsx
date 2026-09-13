import { useEffect, useRef, useState } from "react";
import type { ExtractedFields, UserProfile } from "../lib/types";
import { useLanguage } from "../lib/i18n/LanguageContext";
import { normalizeCategory, normalizeOccupation } from "../lib/normalizeExtracted";
import { INDIAN_STATES, normalizeState } from "../lib/indianStates";

const STORAGE_KEY = "labhsathi:profile";

const DEFAULT_PROFILE: UserProfile = {
  age: 0,
  annual_income: 0,
  occupation: "farmer",
  state: "",
  gender: "female",
  has_disability: false,
  disability_percentage: null,
  land_holding_acres: null,
  family_size: 1,
  is_widow: false,
  category: "general",
  is_student: false,
  // Defaults to true so the "No bank account" pill starts unchecked --
  // GitHub issue #8: defaulting to false pre-asserted "you have no bank
  // account" on behalf of every visitor before they'd said anything.
  has_bank_account: true,
  has_kutcha_house: false,
  is_pregnant_or_lactating_first_child: false,
  girl_child_age: null,
  area_type: null,
};

/** An explicit, obvious shortcut for trying the product without typing --
 * a landholding farmer, deliberately (not just "farmer"), since PM-KISAN
 * specifically requires land holding, not occupation alone (a correction
 * from this session's product review). area_type is set (not left null)
 * so the sample lands on a clean PMAY-Gramin match instead of surfacing
 * a needs-info prompt in what's meant to be a fully-resolved demo. */
const SAMPLE_PROFILE: UserProfile = {
  age: 45,
  annual_income: 80_000,
  occupation: "farmer",
  state: "Goa",
  gender: "male",
  has_disability: false,
  disability_percentage: null,
  land_holding_acres: 2.0,
  family_size: 4,
  is_widow: false,
  category: "obc",
  is_student: false,
  has_bank_account: false,
  has_kutcha_house: true,
  is_pregnant_or_lactating_first_child: false,
  girl_child_age: null,
  area_type: "rural",
};

function loadPersistedProfile(): UserProfile {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw) {
      const saved: UserProfile = { ...DEFAULT_PROFILE, ...JSON.parse(raw) };
      // A session saved while State was still free text can hold anything.
      return { ...saved, state: normalizeState(saved.state) ?? "" };
    }
  } catch {
    // corrupt/blocked storage -- fall through to defaults rather than crash
  }
  return DEFAULT_PROFILE;
}

interface ProfileFormProps {
  extractedFields: ExtractedFields | null;
  /** Incremented by MainScreen each time "try a sample household" is
   * clicked -- watched by reference-change, not truthiness, so the same
   * sample can be requested more than once in one session. */
  sampleTrigger: number;
  /** A UserProfile field key to scroll to and focus on mount -- set when
   * arriving here via a needs_info scheme's "Answer this" link on the
   * results screen. */
  focusField?: string | null;
  onSubmit: (profile: UserProfile) => void;
  submitting: boolean;
}

export function ProfileForm({ extractedFields, sampleTrigger, focusField, onSubmit, submitting }: ProfileFormProps) {
  const { t, lang } = useLanguage();
  const [profile, setProfile] = useState<UserProfile>(loadPersistedProfile);
  // Fields the user has actually typed into -- auto-fill only ever writes
  // into fields NOT in this set, so a scan can never clobber what someone
  // already entered by hand, no matter the order the two happen in.
  const touched = useRef<Set<keyof UserProfile>>(new Set());
  const lastSampleTrigger = useRef(sampleTrigger);
  // Fields whose *current* value came from a scan, not typing -- purely
  // for the "from scan" visual marker. A field drops out of this set the
  // moment the user edits it (see `set` below), independent of `touched`
  // (which never un-marks, since its whole job is permanent protection).
  const [scannedFields, setScannedFields] = useState<Set<keyof UserProfile>>(new Set());
  const [highlightField, setHighlightField] = useState<string | null>(focusField ?? null);

  // State is typed with suggestions, but only a recognized state ever
  // reaches profile.state: stateText is what's in the box, profile.state is
  // the canonical name (or "" while the text doesn't match one).
  const stateInput = useRef<HTMLInputElement>(null);
  const [stateText, setStateText] = useState(() => stateLabel(profile.state, lang));

  // A scan, the sample household, or a language switch changed the state:
  // show its label, unless the person is mid-typing in the box.
  useEffect(() => {
    if (document.activeElement === stateInput.current) return;
    if (profile.state) setStateText(stateLabel(profile.state, lang));
  }, [profile.state, lang]);

  // Unrecognized text blocks submit with a message instead of being sent.
  useEffect(() => {
    stateInput.current?.setCustomValidity(stateText.trim() && !profile.state ? t.form.stateInvalid : "");
  }, [stateText, profile.state, t]);

  // Persisted to sessionStorage (not lifted to MainScreen/localStorage) --
  // fixes a real bug from this session's product review: "Editing
  // answers →" navigates away from "/" and back, which unmounts and
  // remounts this component, silently discarding everything typed.
  // sessionStorage survives that remount without needing a bigger state
  // lift, and clears itself when the tab closes rather than persisting
  // someone else's income figures indefinitely on a shared machine.
  useEffect(() => {
    try {
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(profile));
    } catch {
      // best-effort -- a failed write just means this session's answers
      // don't survive a back-navigation, not worth surfacing to the user
    }
  }, [profile]);

  useEffect(() => {
    if (!extractedFields) return;
    const fromScan: (keyof UserProfile)[] = [];
    setProfile((prev) => {
      const next = { ...prev };
      if (!touched.current.has("age") && extractedFields.age != null) {
        next.age = extractedFields.age;
        fromScan.push("age");
      }
      if (!touched.current.has("annual_income") && extractedFields.annual_income != null) {
        next.annual_income = extractedFields.annual_income;
        fromScan.push("annual_income");
      }
      if (!touched.current.has("state") && extractedFields.state) {
        // Same closed-set rule as category/occupation below.
        const normalized = normalizeState(extractedFields.state);
        if (normalized) {
          next.state = normalized;
          fromScan.push("state");
        }
      }
      if (!touched.current.has("category") && extractedFields.category) {
        // Model wording normalized into the closed dropdown set -- an
        // unrecognized value is left alone rather than injected as free
        // text into a <select> that can't represent it.
        const normalized = normalizeCategory(extractedFields.category);
        if (normalized) {
          next.category = normalized;
          fromScan.push("category");
        }
      }
      if (!touched.current.has("occupation") && extractedFields.occupation) {
        const normalized = normalizeOccupation(extractedFields.occupation);
        if (normalized) {
          next.occupation = normalized;
          fromScan.push("occupation");
        }
      }
      if (!touched.current.has("land_holding_acres") && extractedFields.land_holding_acres != null) {
        next.land_holding_acres = extractedFields.land_holding_acres;
        fromScan.push("land_holding_acres");
      }
      return next;
    });
    if (fromScan.length > 0) {
      setScannedFields((prev) => new Set([...prev, ...fromScan]));
    }
  }, [extractedFields]);

  // An explicit user action, not a background merge -- replaces the whole
  // profile and clears "touched" so the sample isn't silently blocked by
  // a stray earlier edit, unlike the scan auto-fill above.
  useEffect(() => {
    if (sampleTrigger === lastSampleTrigger.current) return;
    lastSampleTrigger.current = sampleTrigger;
    touched.current.clear();
    setScannedFields(new Set());
    setProfile(SAMPLE_PROFILE);
  }, [sampleTrigger]);

  useEffect(() => {
    if (!focusField) return;
    setHighlightField(focusField);
    const el = document.getElementById(`field-${focusField}`);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "center" });
      if (el instanceof HTMLElement) el.focus();
    }
  }, [focusField]);

  function set<K extends keyof UserProfile>(key: K, value: UserProfile[K]) {
    touched.current.add(key);
    setProfile((prev) => ({ ...prev, [key]: value }));
    setScannedFields((prev) => {
      if (!prev.has(key)) return prev;
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
    if (highlightField === key) setHighlightField(null);
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit(profile);
      }}
      className="rounded-[20px] border border-border bg-white p-7 shadow-[0_1px_2px_rgba(28,25,23,0.04),0_12px_32px_-12px_rgba(28,25,23,0.10)] dark:border-dark-border dark:bg-dark-card dark:shadow-none"
    >
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
        <Field label={t.form.age} id="age">
          <input
            id="field-age"
            type="number"
            min={0}
            max={120}
            required
            value={profile.age || ""}
            onChange={(e) => set("age", Number(e.target.value))}
            className={inputClass(scannedFields.has("age"), highlightField === "age")}
          />
        </Field>
        <Field label={t.form.income} id="annual_income">
          <input
            id="field-annual_income"
            type="number"
            min={0}
            required
            value={profile.annual_income || ""}
            onChange={(e) => set("annual_income", Number(e.target.value))}
            className={inputClass(scannedFields.has("annual_income"), highlightField === "annual_income")}
          />
        </Field>
        <Field label={t.form.occupation} id="occupation">
          <select
            id="field-occupation"
            value={profile.occupation}
            onChange={(e) => set("occupation", e.target.value)}
            className={inputClass(scannedFields.has("occupation"), highlightField === "occupation")}
          >
            {Object.entries(t.form.occupationOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.state} id="state">
          <input
            ref={stateInput}
            id="field-state"
            type="text"
            required
            list="state-options"
            autoComplete="off"
            placeholder={t.form.statePlaceholder}
            value={stateText}
            onChange={(e) => {
              setStateText(e.target.value);
              set("state", normalizeState(e.target.value) ?? "");
            }}
            onBlur={() => {
              if (profile.state) setStateText(stateLabel(profile.state, lang));
            }}
            className={inputClass(scannedFields.has("state"), highlightField === "state")}
          />
          <datalist id="state-options">
            {INDIAN_STATES.map((s) => (
              <option key={s.value} value={lang === "hi" ? s.hi : s.value} />
            ))}
          </datalist>
        </Field>
        <Field label={t.form.category} id="category">
          <select
            id="field-category"
            value={profile.category}
            onChange={(e) => set("category", e.target.value)}
            className={inputClass(scannedFields.has("category"), highlightField === "category")}
          >
            {Object.entries(t.form.categoryOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.familySize} id="family_size">
          <input
            id="field-family_size"
            type="number"
            min={1}
            required
            value={profile.family_size}
            onChange={(e) => set("family_size", Number(e.target.value))}
            className={inputClass(false, highlightField === "family_size")}
          />
        </Field>
        <Field label={t.form.gender} id="gender">
          <select
            id="field-gender"
            value={profile.gender}
            onChange={(e) => set("gender", e.target.value)}
            className={inputClass(false, highlightField === "gender")}
          >
            {Object.entries(t.form.genderOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.landHolding} id="land_holding_acres">
          <input
            id="field-land_holding_acres"
            type="number"
            min={0}
            step={0.1}
            value={profile.land_holding_acres ?? ""}
            onChange={(e) => set("land_holding_acres", e.target.value === "" ? null : Number(e.target.value))}
            className={inputClass(scannedFields.has("land_holding_acres"), highlightField === "land_holding_acres")}
          />
        </Field>
        <Field label={t.form.daughterAge} id="girl_child_age">
          <input
            id="field-girl_child_age"
            type="number"
            min={0}
            max={25}
            value={profile.girl_child_age ?? ""}
            onChange={(e) => set("girl_child_age", e.target.value === "" ? null : Number(e.target.value))}
            className={inputClass(false, highlightField === "girl_child_age")}
          />
        </Field>
      </div>

      <div className="mt-5 flex flex-wrap gap-2.5">
        <Pill
          label={t.form.pillDisability}
          checked={profile.has_disability}
          onChange={(v) => set("has_disability", v)}
        />
        <Pill
          label={t.form.pillKutcha}
          checked={profile.has_kutcha_house}
          onChange={(v) => set("has_kutcha_house", v)}
        />
        <Pill label={t.form.pillStudent} checked={profile.is_student} onChange={(v) => set("is_student", v)} />
        <Pill
          label={t.form.pillNoBankAccount}
          checked={!profile.has_bank_account}
          onChange={(v) => set("has_bank_account", !v)}
        />
        <Pill label={t.form.pillWidow} checked={profile.is_widow} onChange={(v) => set("is_widow", v)} />
        {profile.gender === "female" && (
          <Pill
            label={t.form.pillPregnant}
            checked={profile.is_pregnant_or_lactating_first_child}
            onChange={(v) => set("is_pregnant_or_lactating_first_child", v)}
          />
        )}
      </div>

      {profile.has_disability && (
        <div className="mt-4 max-w-xs">
          <Field label={t.form.disabilityPercentage} id="disability_percentage">
            <input
              id="field-disability_percentage"
              type="number"
              min={0}
              max={100}
              value={profile.disability_percentage ?? ""}
              onChange={(e) =>
                set("disability_percentage", e.target.value === "" ? null : Number(e.target.value))
              }
              className={inputClass(false, highlightField === "disability_percentage")}
            />
          </Field>
        </div>
      )}

      {/* Only asked when it's actually load-bearing (housing schemes) --
          this is the one piece of missing information that can turn a
          needs_info PMAY card into a real match or non-match. */}
      {profile.has_kutcha_house && (
        <div className="mt-4 max-w-xs">
          <Field label={t.form.areaType} id="area_type">
            <select
              id="field-area_type"
              value={profile.area_type ?? ""}
              onChange={(e) => set("area_type", e.target.value === "" ? null : e.target.value)}
              className={inputClass(false, highlightField === "area_type")}
            >
              <option value="">{t.form.areaTypeUnset}</option>
              {Object.entries(t.form.areaTypeOptions).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
        </div>
      )}

      <button
        type="submit"
        disabled={submitting}
        className="mt-7 flex w-full items-center justify-center gap-2 rounded-xl bg-ink px-4 py-4 text-[15px] font-semibold text-cream hover:bg-ink/90 disabled:opacity-60 dark:bg-dark-text dark:text-dark-bg dark:hover:bg-dark-text/90"
      >
        {submitting ? t.form.submitting : t.form.submit}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M5 12h14M13 5l7 7-7 7" />
        </svg>
      </button>
    </form>
  );
}

function stateLabel(value: string, lang: string): string {
  const s = INDIAN_STATES.find((x) => x.value === value);
  if (!s) return "";
  return lang === "hi" ? s.hi : s.value;
}

function inputClass(fromScan: boolean, highlighted: boolean): string {
  const base =
    "rounded-[10px] border bg-input-bg px-3.5 py-3 text-sm text-ink focus:outline-none focus:ring-2 focus:ring-terracotta/40 dark:bg-dark-bg dark:text-dark-text";
  if (highlighted) return `${base} border-terracotta ring-2 ring-terracotta/50`;
  if (fromScan) return `${base} border-teal bg-teal-light/10 dark:bg-teal-light/[0.08]`;
  return `${base} border-border dark:border-dark-border`;
}

function Field({ label, id, children }: { label: string; id: string; children: React.ReactNode }) {
  return (
    <label htmlFor={`field-${id}`} className="flex flex-col gap-2">
      <span className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wide text-ink-secondary dark:text-dark-secondary">
        {label}
      </span>
      {children}
    </label>
  );
}

function Pill({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-2 rounded-full border border-border bg-input-bg px-3.5 py-2 text-[13px] text-ink-muted dark:border-dark-border dark:bg-dark-bg dark:text-dark-secondary">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
        className="accent-terracotta"
      />
      {label}
    </label>
  );
}
