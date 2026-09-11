import { useEffect, useRef, useState } from "react";
import type { ExtractedFields, UserProfile } from "../lib/types";
import { useLanguage } from "../lib/i18n/LanguageContext";

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
};

/** An explicit, obvious shortcut for trying the product without typing --
 * a landholding farmer, deliberately (not just "farmer"), since PM-KISAN
 * specifically requires land holding, not occupation alone (a correction
 * from this session's product review). Matches several real schemes so
 * the results screen has something to show. */
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
};

function loadPersistedProfile(): UserProfile {
  try {
    const raw = sessionStorage.getItem(STORAGE_KEY);
    if (raw) return { ...DEFAULT_PROFILE, ...JSON.parse(raw) };
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
  onSubmit: (profile: UserProfile) => void;
  submitting: boolean;
}

export function ProfileForm({ extractedFields, sampleTrigger, onSubmit, submitting }: ProfileFormProps) {
  const { t } = useLanguage();
  const [profile, setProfile] = useState<UserProfile>(loadPersistedProfile);
  // Fields the user has actually typed into -- auto-fill only ever writes
  // into fields NOT in this set, so a scan can never clobber what someone
  // already entered by hand, no matter the order the two happen in.
  const touched = useRef<Set<keyof UserProfile>>(new Set());
  const lastSampleTrigger = useRef(sampleTrigger);

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
    setProfile((prev) => {
      const next = { ...prev };
      if (!touched.current.has("age") && extractedFields.age != null) next.age = extractedFields.age;
      if (!touched.current.has("annual_income") && extractedFields.annual_income != null)
        next.annual_income = extractedFields.annual_income;
      if (!touched.current.has("state") && extractedFields.state) next.state = extractedFields.state;
      if (!touched.current.has("category") && extractedFields.category)
        next.category = extractedFields.category;
      if (!touched.current.has("occupation") && extractedFields.occupation)
        next.occupation = extractedFields.occupation;
      if (!touched.current.has("land_holding_acres") && extractedFields.land_holding_acres != null)
        next.land_holding_acres = extractedFields.land_holding_acres;
      return next;
    });
  }, [extractedFields]);

  // An explicit user action, not a background merge -- replaces the whole
  // profile and clears "touched" so the sample isn't silently blocked by
  // a stray earlier edit, unlike the scan auto-fill above.
  useEffect(() => {
    if (sampleTrigger === lastSampleTrigger.current) return;
    lastSampleTrigger.current = sampleTrigger;
    touched.current.clear();
    setProfile(SAMPLE_PROFILE);
  }, [sampleTrigger]);

  function set<K extends keyof UserProfile>(key: K, value: UserProfile[K]) {
    touched.current.add(key);
    setProfile((prev) => ({ ...prev, [key]: value }));
  }

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSubmit(profile);
      }}
      className="rounded-[20px] border border-border bg-white p-7 shadow-[0_1px_2px_rgba(28,25,23,0.04),0_12px_32px_-12px_rgba(28,25,23,0.10)]"
    >
      <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
        <Field label={t.form.age}>
          <input
            type="number"
            min={0}
            max={120}
            required
            value={profile.age || ""}
            onChange={(e) => set("age", Number(e.target.value))}
            className={inputClass}
          />
        </Field>
        <Field label={t.form.income}>
          <input
            type="number"
            min={0}
            required
            value={profile.annual_income || ""}
            onChange={(e) => set("annual_income", Number(e.target.value))}
            className={inputClass}
          />
        </Field>
        <Field label={t.form.occupation}>
          <select
            value={profile.occupation}
            onChange={(e) => set("occupation", e.target.value)}
            className={inputClass}
          >
            {Object.entries(t.form.occupationOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.state}>
          <input
            type="text"
            required
            placeholder={t.form.statePlaceholder}
            value={profile.state}
            onChange={(e) => set("state", e.target.value)}
            className={inputClass}
          />
        </Field>
        <Field label={t.form.category}>
          <select
            value={profile.category}
            onChange={(e) => set("category", e.target.value)}
            className={inputClass}
          >
            {Object.entries(t.form.categoryOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.familySize}>
          <input
            type="number"
            min={1}
            required
            value={profile.family_size}
            onChange={(e) => set("family_size", Number(e.target.value))}
            className={inputClass}
          />
        </Field>
        <Field label={t.form.gender}>
          <select
            value={profile.gender}
            onChange={(e) => set("gender", e.target.value)}
            className={inputClass}
          >
            {Object.entries(t.form.genderOptions).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label={t.form.landHolding}>
          <input
            type="number"
            min={0}
            step={0.1}
            value={profile.land_holding_acres ?? ""}
            onChange={(e) => set("land_holding_acres", e.target.value === "" ? null : Number(e.target.value))}
            className={inputClass}
          />
        </Field>
        <Field label={t.form.daughterAge}>
          <input
            type="number"
            min={0}
            max={25}
            value={profile.girl_child_age ?? ""}
            onChange={(e) => set("girl_child_age", e.target.value === "" ? null : Number(e.target.value))}
            className={inputClass}
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
          <Field label={t.form.disabilityPercentage}>
            <input
              type="number"
              min={0}
              max={100}
              value={profile.disability_percentage ?? ""}
              onChange={(e) =>
                set("disability_percentage", e.target.value === "" ? null : Number(e.target.value))
              }
              className={inputClass}
            />
          </Field>
        </div>
      )}

      <button
        type="submit"
        disabled={submitting}
        className="mt-7 flex w-full items-center justify-center gap-2 rounded-xl bg-ink px-4 py-4 text-[15px] font-semibold text-cream hover:bg-ink/90 disabled:opacity-60"
      >
        {submitting ? t.form.submitting : t.form.submit}
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#faf7f2" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <path d="M5 12h14M13 5l7 7-7 7" />
        </svg>
      </button>
    </form>
  );
}

const inputClass =
  "rounded-[10px] border border-border bg-input-bg px-3.5 py-3 text-sm text-ink focus:outline-none focus:ring-2 focus:ring-terracotta/40";

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-2">
      <span className="text-xs font-semibold uppercase tracking-wide text-ink-secondary">{label}</span>
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
    <label className="flex items-center gap-2 rounded-full border border-border bg-input-bg px-3.5 py-2 text-[13px] text-ink-muted">
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
