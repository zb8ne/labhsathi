import { useEffect, useRef, useState } from "react";
import type { ExtractedFields, UserProfile } from "../lib/types";

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
  has_bank_account: false,
  has_kutcha_house: false,
  is_pregnant_or_lactating_first_child: false,
  girl_child_age: null,
};

interface ProfileFormProps {
  extractedFields: ExtractedFields | null;
  onSubmit: (profile: UserProfile) => void;
  submitting: boolean;
}

export function ProfileForm({ extractedFields, onSubmit, submitting }: ProfileFormProps) {
  const [profile, setProfile] = useState<UserProfile>(DEFAULT_PROFILE);
  // Fields the user has actually typed into -- auto-fill only ever writes
  // into fields NOT in this set, so a scan can never clobber what someone
  // already entered by hand, no matter the order the two happen in.
  const touched = useRef<Set<keyof UserProfile>>(new Set());

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
        <Field label="Age">
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
        <Field label="Annual household income (₹)">
          <input
            type="number"
            min={0}
            required
            value={profile.annual_income || ""}
            onChange={(e) => set("annual_income", Number(e.target.value))}
            className={inputClass}
          />
        </Field>
        <Field label="Occupation">
          <select
            value={profile.occupation}
            onChange={(e) => set("occupation", e.target.value)}
            className={inputClass}
          >
            <option value="farmer">Farmer</option>
            <option value="laborer">Laborer / unorganised worker</option>
            <option value="self_employed">Self-employed</option>
            <option value="salaried">Salaried</option>
            <option value="unemployed">Unemployed</option>
            <option value="student">Student</option>
            <option value="homemaker">Homemaker</option>
            <option value="retired">Retired</option>
          </select>
        </Field>
        <Field label="State">
          <input
            type="text"
            required
            placeholder="e.g. Goa"
            value={profile.state}
            onChange={(e) => set("state", e.target.value)}
            className={inputClass}
          />
        </Field>
        <Field label="Category">
          <select
            value={profile.category}
            onChange={(e) => set("category", e.target.value)}
            className={inputClass}
          >
            <option value="general">General</option>
            <option value="sc">SC</option>
            <option value="st">ST</option>
            <option value="obc">OBC</option>
            <option value="minority">Minority</option>
          </select>
        </Field>
        <Field label="Family size">
          <input
            type="number"
            min={1}
            required
            value={profile.family_size}
            onChange={(e) => set("family_size", Number(e.target.value))}
            className={inputClass}
          />
        </Field>
      </div>

      <div className="mt-5 flex flex-wrap gap-2.5">
        <Pill
          label="Has a certified disability"
          checked={profile.has_disability}
          onChange={(v) => set("has_disability", v)}
        />
        <Pill
          label="Lives in a kutcha house"
          checked={profile.has_kutcha_house}
          onChange={(v) => set("has_kutcha_house", v)}
        />
        <Pill label="Currently a student" checked={profile.is_student} onChange={(v) => set("is_student", v)} />
        <Pill
          label="No bank account"
          checked={!profile.has_bank_account}
          onChange={(v) => set("has_bank_account", !v)}
        />
        <Pill label="Widow" checked={profile.is_widow} onChange={(v) => set("is_widow", v)} />
      </div>

      <button
        type="submit"
        disabled={submitting}
        className="mt-7 flex w-full items-center justify-center gap-2 rounded-xl bg-ink px-4 py-4 text-[15px] font-semibold text-cream hover:bg-ink/90 disabled:opacity-60"
      >
        {submitting ? "Finding your schemes…" : "Find my schemes"}
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
