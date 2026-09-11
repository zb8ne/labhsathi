import { useLanguage } from "../lib/i18n/LanguageContext";

// The topic name and JSON shape are literal Kafka schema identifiers, not
// UI prose -- they stay in English in both languages, same as a code
// sample would. Only the "image: never a field" commentary is translated.
export function SchemaStrip() {
  const { t } = useLanguage();
  return (
    <div className="mx-6 mt-14 flex flex-col items-start gap-3 rounded-xl border border-dark-border-soft bg-[#141110] px-6 py-5 sm:mx-14 sm:flex-row sm:items-center sm:gap-4">
      <span className="whitespace-nowrap text-[11px] font-bold uppercase tracking-wide text-ink-secondary">
        {t.privacy.schemaTopic}
      </span>
      <div className="hidden h-4 w-px bg-dark-border-soft sm:block" />
      <span className="font-mono text-[12.5px] text-[#d6d3d1]">
        {"{ job_id, "}
        <span className="text-teal-light">fields: {"{ age, income, state, category }"}</span>
        {", ts }"}
      </span>
      <div className="flex items-center gap-1.5 rounded-full border border-error-border bg-error-bg px-2.5 py-1.5 sm:ml-auto">
        <span className="font-mono text-[11px] text-error-light">{t.privacy.schemaNeverField}</span>
      </div>
    </div>
  );
}
