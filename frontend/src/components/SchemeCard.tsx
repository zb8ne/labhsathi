import { useState } from "react";
import { useNavigate } from "react-router-dom";
import type { SchemeMatch } from "../lib/types";
import { CATEGORY_LABELS } from "../lib/types";
import { useLanguage } from "../lib/i18n/LanguageContext";
import { localizeScheme } from "../lib/i18n/schemeTranslations";

const CHECKLIST_KEY = "labhsathi:documentChecklist";

function loadChecklist(): Record<string, string[]> {
  try {
    const raw = sessionStorage.getItem(CHECKLIST_KEY);
    if (raw) return JSON.parse(raw);
  } catch {
    // corrupt/blocked storage -- start empty rather than crash
  }
  return {};
}

function saveChecklist(all: Record<string, string[]>) {
  try {
    sessionStorage.setItem(CHECKLIST_KEY, JSON.stringify(all));
  } catch {
    // best-effort only -- losing this on write failure just means the
    // have/still-need marks don't survive a reload, not worth surfacing
  }
}

/** Which documents the user has already marked "have it," per scheme --
 * session-scoped like the rest of this app's client state, never sent to
 * a server. */
function useDocumentChecklist(schemeId: string) {
  const [have, setHave] = useState<Set<string>>(() => new Set(loadChecklist()[schemeId] ?? []));

  function toggle(doc: string) {
    setHave((prev) => {
      const next = new Set(prev);
      if (next.has(doc)) next.delete(doc);
      else next.add(doc);
      const all = loadChecklist();
      all[schemeId] = [...next];
      saveChecklist(all);
      return next;
    });
  }

  return { have, toggle };
}

export function SchemeCard({ scheme }: { scheme: SchemeMatch }) {
  const { t, lang } = useLanguage();
  const navigate = useNavigate();
  const localized = localizeScheme(scheme, lang);
  const { have, toggle } = useDocumentChecklist(scheme.id);
  const needsInfo = scheme.status === "needs_info";

  return (
    <div
      className={`flex flex-col gap-4 rounded-2xl border bg-white p-6 break-inside-avoid dark:bg-dark-card print:border-ink-tertiary ${
        needsInfo ? "border-teal-light dark:border-teal/40" : "border-border dark:border-dark-border"
      }`}
    >
      {/* GitHub issue #3: a long benefit string (e.g. "₹5,00,000/family/year
          cashless health insurance") with a nowrap badge in a row that
          never wraps forced horizontal overflow on phone widths. Stacks
          below sm; the badge itself is also allowed to wrap once stacked. */}
      <div className="flex flex-col items-start gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-3">
        <div>
          <div className="mb-1.5 flex flex-wrap items-center gap-2">
            <span className="text-[11px] font-bold uppercase tracking-wide text-terracotta dark:text-amber-orange">
              {localized.authority}
            </span>
            {scheme.category_domain && (
              <span className="rounded-full border border-border bg-input-bg px-2 py-0.5 text-[10px] font-semibold text-ink-secondary dark:border-dark-border dark:bg-dark-bg dark:text-dark-secondary">
                {CATEGORY_LABELS[scheme.category_domain] ?? scheme.category_domain}
              </span>
            )}
          </div>
          <div className="font-serif text-lg font-semibold">{localized.name}</div>
        </div>
        <div className="rounded-full border border-success-border bg-success-bg px-3 py-1.5 text-xs font-semibold text-success-text dark:border-success-border/30 dark:bg-success-bg/10 dark:text-success-accent sm:whitespace-nowrap">
          {localized.benefit}
        </div>
      </div>

      {needsInfo ? (
        <div className="flex flex-col gap-2.5 rounded-[10px] border-l-[3px] border-teal bg-teal-light/10 px-4 py-3.5 dark:bg-teal-light/[0.06] print:border print:bg-transparent">
          <div className="flex items-start gap-3">
            <span className="mt-0.5 shrink-0 rounded-full border border-teal-light bg-white px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-teal dark:border-teal/40 dark:bg-dark-card dark:text-teal-light">
              {t.results.badgeNeedsInfo}
            </span>
            <span className="text-[13.5px] leading-relaxed text-ink-muted dark:text-dark-secondary">{localized.reason}</span>
          </div>
          {scheme.missing_field && (
            <button
              type="button"
              onClick={() => navigate("/", { state: { focusField: scheme.missing_field } })}
              className="self-start text-xs font-semibold text-teal hover:underline dark:text-teal-light print:hidden"
            >
              {t.results.answerThis}
            </button>
          )}
        </div>
      ) : (
        <div className="flex items-start gap-3 rounded-[10px] border-l-[3px] border-amber-tan bg-input-bg px-4 py-3.5 dark:bg-dark-bg print:border print:bg-transparent">
          {/* "Worth checking," not "strong match" -- these are simplified
              rules against a demo scheme set, and the wording should never
              overstate that (product review note this session). */}
          <span className="mt-0.5 shrink-0 rounded-full bg-badge-bg px-2 py-0.5 text-[10px] font-bold uppercase tracking-wide text-[#92400e] dark:bg-amber-tan/10 dark:text-amber-tan">
            {t.results.badgeWorthChecking}
          </span>
          <span className="text-[13.5px] leading-relaxed text-ink-muted dark:text-dark-secondary">{localized.reason}</span>
        </div>
      )}

      <div>
        <div className="mb-2 text-[11px] font-bold uppercase tracking-wide text-ink-tertiary dark:text-dark-secondary">
          {t.results.documentsNeeded}
        </div>
        <div className="flex flex-wrap gap-1.5">
          {localized.documents.map((doc) => {
            const has = have.has(doc);
            return (
              <button
                key={doc}
                type="button"
                onClick={() => toggle(doc)}
                aria-pressed={has}
                title={has ? t.results.documentHave : t.results.documentNeed}
                className={`flex items-center gap-1.5 rounded-full px-2.5 py-1.5 text-xs transition-colors print:border print:bg-transparent ${
                  has
                    ? "bg-success-bg text-success-text border border-success-border dark:bg-success-bg/10 dark:text-success-accent dark:border-success-border/30"
                    : "bg-[#f5f5f4] text-ink-muted border border-transparent dark:bg-dark-bg dark:text-dark-secondary"
                }`}
              >
                <CheckDot has={has} />
                {doc}
              </button>
            );
          })}
        </div>
      </div>

      <p className="text-xs text-ink-tertiary dark:text-dark-secondary">{localized.official_note}</p>

      {!needsInfo &&
        (localized.source_url ? (
          <a
            href={localized.source_url}
            target="_blank"
            rel="noopener noreferrer"
            className="-mt-1 text-xs font-semibold text-terracotta hover:underline dark:text-amber-orange"
          >
            {t.results.officialSource}
          </a>
        ) : (
          <p className="-mt-1 text-xs text-ink-tertiary dark:text-dark-secondary">{t.results.applyFallback}</p>
        ))}
    </div>
  );
}

function CheckDot({ has }: { has: boolean }) {
  if (!has) return <span className="h-3 w-3 shrink-0 rounded-full border border-ink-tertiary/40 dark:border-dark-secondary/50" />;
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className="shrink-0">
      <path d="M20 6 9 17l-5-5" />
    </svg>
  );
}
