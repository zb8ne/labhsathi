import type { SchemeMatch } from "../lib/types";
import { useLanguage } from "../lib/i18n/LanguageContext";
import { localizeScheme } from "../lib/i18n/schemeTranslations";

export function SchemeCard({ scheme }: { scheme: SchemeMatch }) {
  const { t, lang } = useLanguage();
  const localized = localizeScheme(scheme, lang);

  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-border bg-white p-6">
      {/* GitHub issue #3: a long benefit string (e.g. "₹5,00,000/family/year
          cashless health insurance") with a nowrap badge in a row that
          never wraps forced horizontal overflow on phone widths. Stacks
          below sm; the badge itself is also allowed to wrap once stacked. */}
      <div className="flex flex-col items-start gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-3">
        <div>
          <div className="mb-1.5 text-[11px] font-bold uppercase tracking-wide text-terracotta">
            {localized.authority}
          </div>
          <div className="font-serif text-lg font-semibold">{localized.name}</div>
        </div>
        <div className="rounded-full border border-success-border bg-success-bg px-3 py-1.5 text-xs font-semibold text-success-text sm:whitespace-nowrap">
          {localized.benefit}
        </div>
      </div>

      <div className="flex items-start gap-3 rounded-[10px] border-l-[3px] border-amber-tan bg-input-bg px-4 py-3.5">
        {/* "Worth checking," not "strong match" -- these are simplified
            rules against a demo scheme set, and the wording should never
            overstate that (product review note this session). */}
        <span className="mt-0.5 shrink-0 rounded-full bg-badge-bg px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-[#92400e]">
          {t.results.badgeWorthChecking}
        </span>
        <span className="text-[13.5px] leading-relaxed text-ink-muted">{localized.reason}</span>
      </div>

      <div>
        <div className="mb-2 text-[11px] font-bold uppercase tracking-wide text-ink-tertiary">
          {t.results.documentsNeeded}
        </div>
        <div className="flex flex-wrap gap-1.5">
          {localized.documents.map((doc) => (
            <span key={doc} className="rounded-full bg-[#f5f5f4] px-2.5 py-1.5 text-xs text-ink-muted">
              {doc}
            </span>
          ))}
        </div>
      </div>

      <p className="text-xs text-ink-tertiary">{localized.official_note}</p>

      {/* ZB8-12: only rendered when the backend actually has a
          live-verified government URL for this scheme -- most don't yet
          (see schemes.rs), so this is deliberately absent on most cards
          rather than a placeholder link. */}
      {localized.source_url && (
        <a
          href={localized.source_url}
          target="_blank"
          rel="noopener noreferrer"
          className="-mt-1 text-xs font-semibold text-terracotta hover:underline"
        >
          {t.results.officialSource}
        </a>
      )}
    </div>
  );
}
