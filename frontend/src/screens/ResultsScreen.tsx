import { useEffect, useState, useMemo } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";
import { SchemeCard } from "../components/SchemeCard";
import { useLanguage, resultsHeading, needsInfoNote } from "../lib/i18n/LanguageContext";
import type { MatchResponse } from "../lib/types";
import { CATEGORY_LABELS } from "../lib/types";

export function ResultsScreen() {
  const { t } = useLanguage();
  const location = useLocation();
  const navigate = useNavigate();
  const result = location.state as MatchResponse | undefined;
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  // Direct navigation with no match result (refresh, deep link, back button
  // after clearing state) -- send back to the form rather than rendering a
  // broken results page. Navigating during render is unsafe in React, so
  // this happens in an effect and the render below bails out with null in
  // the meantime.
  useEffect(() => {
    if (!result) navigate("/", { replace: true });
  }, [result, navigate]);

  const categories = useMemo(() => {
    if (!result) return [];
    const counts = new Map<string, number>();
    for (const s of result.schemes) {
      if (s.category_domain) {
        counts.set(s.category_domain, (counts.get(s.category_domain) ?? 0) + 1);
      }
    }
    return Array.from(counts.entries()).map(([cat, count]) => ({
      key: cat,
      label: CATEGORY_LABELS[cat] ?? cat,
      count,
    }));
  }, [result]);

  if (!result) return null;

  // matched_count from the API is the *total* rows returned, which now
  // includes needs_info entries -- counting only real matches for the
  // headline keeps "eligible for N schemes" honest (a needs_info card
  // isn't a known eligibility yet, just an open question).
  const matchCount = result.schemes.filter((s) => s.status === "match").length;
  const needsInfoCount = result.schemes.filter((s) => s.status === "needs_info").length;

  const displayedSchemes =
    selectedCategory === "all"
      ? result.schemes
      : result.schemes.filter((s) => s.category_domain === selectedCategory);

  return (
    <div className="flex min-h-screen flex-col bg-cream text-ink dark:bg-dark-bg dark:text-dark-text">
      <Nav
        right={
          <Link to="/" className="whitespace-nowrap text-[13px] text-ink-tertiary dark:text-dark-secondary print:hidden">
            {t.nav.editingAnswers}
          </Link>
        }
      />

      <main className="flex-1 px-6 pb-14 pt-10 sm:px-14 sm:pt-12">
        <div className="mb-8 flex flex-col items-start gap-4 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h1 className="mb-2 font-serif text-3xl font-semibold sm:text-[34px]">
              {resultsHeading(t, matchCount)}
            </h1>
            {needsInfoCount > 0 && (
              <p className="mb-1 text-[13.5px] font-medium text-teal dark:text-teal-light">{needsInfoNote(t, needsInfoCount)}</p>
            )}
            <p className="text-[15px] text-ink-secondary dark:text-dark-secondary">{t.results.subhead}</p>
          </div>
          {result.schemes.length > 0 && (
            <button type="button" onClick={printInLightMode} className="flex shrink-0 items-center gap-2 whitespace-nowrap rounded-[10px] border border-border bg-white px-4 py-2.5 text-sm font-semibold text-ink hover:bg-input-bg print:hidden dark:border-dark-border dark:bg-dark-card dark:text-dark-text dark:hover:bg-dark-border">
              <PrintIcon />
              {t.results.printButton}
            </button>
          )}
        </div>

        {categories.length > 1 && (
          <div className="mb-6 flex flex-wrap items-center gap-2 print:hidden">
            <button
              type="button"
              onClick={() => setSelectedCategory("all")}
              className={`rounded-full px-3.5 py-1.5 text-xs font-semibold transition-colors ${
                selectedCategory === "all"
                  ? "bg-ink text-cream dark:bg-dark-text dark:text-dark-bg"
                  : "border border-border bg-white text-ink-secondary hover:bg-input-bg dark:border-dark-border dark:bg-dark-card dark:text-dark-secondary"
              }`}
            >
              All ({result.schemes.length})
            </button>
            {categories.map((c) => (
              <button
                key={c.key}
                type="button"
                onClick={() => setSelectedCategory(c.key)}
                className={`rounded-full px-3.5 py-1.5 text-xs font-semibold transition-colors ${
                  selectedCategory === c.key
                    ? "bg-ink text-cream dark:bg-dark-text dark:text-dark-bg"
                    : "border border-border bg-white text-ink-secondary hover:bg-input-bg dark:border-dark-border dark:bg-dark-card dark:text-dark-secondary"
                }`}
              >
                {c.label} ({c.count})
              </button>
            ))}
          </div>
        )}

        {displayedSchemes.length === 0 ? (
          <p className="text-ink-secondary dark:text-dark-secondary">{t.results.noMatches}</p>
        ) : (
          <>
            <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 print:grid-cols-1">
              {displayedSchemes.map((scheme) => (
                <SchemeCard key={scheme.id} scheme={scheme} />
              ))}
            </div>
            <p className="mt-6 text-xs text-ink-tertiary dark:text-dark-secondary print:hidden">{t.results.printNote}</p>
          </>
        )}
      </main>

      <div className="print:hidden">
        <GradientBar />
      </div>
    </div>
  );
}

// Paper is always light regardless of on-screen theme -- printing a dark
// background wastes ink and most printers/PDF exports don't render it
// well anyway. Relying on Tailwind's print:/dark: cascade order to fight
// this out per-element is fragile; instead the .dark class is removed
// from <html> for the actual print, then restored once the print dialog
// closes (afterprint fires whether the user prints or cancels).
function printInLightMode() {
  const root = document.documentElement;
  const wasDark = root.classList.contains("dark");
  if (!wasDark) {
    window.print();
    return;
  }
  root.classList.remove("dark");
  const restore = () => {
    root.classList.add("dark");
    window.removeEventListener("afterprint", restore);
  };
  window.addEventListener("afterprint", restore);
  window.print();
}

function PrintIcon() {
  return (
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <path d="M6 9V2h12v7" />
      <path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2" />
      <path d="M6 14h12v8H6z" />
    </svg>
  );
}
