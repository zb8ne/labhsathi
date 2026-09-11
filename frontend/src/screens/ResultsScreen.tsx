import { useEffect } from "react";
import { Link, useLocation, useNavigate } from "react-router-dom";
import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";
import { SchemeCard } from "../components/SchemeCard";
import { useLanguage, resultsHeading } from "../lib/i18n/LanguageContext";
import type { MatchResponse } from "../lib/types";

export function ResultsScreen() {
  const { t } = useLanguage();
  const location = useLocation();
  const navigate = useNavigate();
  const result = location.state as MatchResponse | undefined;

  // Direct navigation with no match result (refresh, deep link, back button
  // after clearing state) -- send back to the form rather than rendering a
  // broken results page. Navigating during render is unsafe in React, so
  // this happens in an effect and the render below bails out with null in
  // the meantime.
  useEffect(() => {
    if (!result) navigate("/", { replace: true });
  }, [result, navigate]);

  if (!result) return null;

  return (
    <div className="flex min-h-screen flex-col bg-cream">
      <Nav
        variant="light"
        right={
          <Link to="/" className="whitespace-nowrap text-[13px] text-ink-tertiary">
            {t.nav.editingAnswers}
          </Link>
        }
      />

      <main className="flex-1 px-6 pb-14 pt-10 sm:px-14 sm:pt-12">
        <div className="mb-8">
          <h1 className="mb-2 font-serif text-3xl font-semibold sm:text-[34px]">
            {resultsHeading(t, result.matched_count)}
          </h1>
          <p className="text-[15px] text-ink-secondary">{t.results.subhead}</p>
        </div>

        {result.schemes.length === 0 ? (
          <p className="text-ink-secondary">{t.results.noMatches}</p>
        ) : (
          <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
            {result.schemes.map((scheme) => (
              <SchemeCard key={scheme.id} scheme={scheme} />
            ))}
          </div>
        )}
      </main>

      <GradientBar />
    </div>
  );
}
