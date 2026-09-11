import { useState } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";
import { ProfileForm } from "../components/ProfileForm";
import { ScanPanel } from "../components/ScanPanel";
import { matchSchemes } from "../lib/api";
import { useLanguage } from "../lib/i18n/LanguageContext";
import type { ExtractedFields, UserProfile } from "../lib/types";

export function MainScreen() {
  const { t } = useLanguage();
  const [extractedFields, setExtractedFields] = useState<ExtractedFields | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Incremented (never reset to 0) so ProfileForm can watch for *changes*
  // rather than truthiness -- the sample can be requested more than once.
  const [sampleTrigger, setSampleTrigger] = useState(0);
  const navigate = useNavigate();
  // Set when arriving here via a needs_info scheme's "Answer this" link on
  // the results screen -- see ResultsScreen's navigate() call.
  const location = useLocation();
  const focusField = (location.state as { focusField?: string } | null)?.focusField ?? null;

  const steps = [
    { title: t.main.step1Title, body: t.main.step1Body },
    { title: t.main.step2Title, body: t.main.step2Body },
    { title: t.main.step3Title, body: t.main.step3Body },
  ];

  async function handleSubmit(profile: UserProfile) {
    setSubmitting(true);
    setError(null);
    try {
      const result = await matchSchemes(profile);
      navigate("/results", { state: result });
    } catch (e) {
      setError(e instanceof Error ? e.message : t.main.matchError);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-cream text-ink dark:bg-dark-bg dark:text-dark-text">
      <Nav />

      <main className="flex flex-1 flex-col gap-12 px-6 py-10 sm:flex-row sm:px-14 sm:py-14">
        <div className="sm:w-[420px] sm:flex-shrink-0">
          <div className="mb-6 inline-block rounded-full border border-badge-border bg-badge-bg px-3 py-1.5 text-xs font-semibold tracking-wide text-[#92400e] dark:border-amber-tan/30 dark:bg-amber-tan/10 dark:text-amber-tan">
            {t.main.tagline}
          </div>
          <h1 className="mb-5 font-serif text-4xl font-semibold leading-[1.05] tracking-tight sm:text-[52px]">
            {t.main.headlineLine1}
            <br />
            <span className="italic text-terracotta dark:text-amber-orange">{t.main.headlineEmphasis}</span>
          </h1>
          <p className="mb-4 max-w-[38ch] text-[17px] leading-relaxed text-ink-muted dark:text-dark-secondary">{t.main.subhead}</p>
          <button
            type="button"
            onClick={() => setSampleTrigger((n) => n + 1)}
            className="mb-8 text-sm font-semibold text-terracotta underline decoration-amber-tan decoration-2 underline-offset-4 hover:text-terracotta-hover dark:text-amber-orange dark:hover:text-amber-tan"
          >
            {t.main.sampleShortcut}
          </button>

          <div className="flex flex-col gap-4 border-t border-border pt-7 dark:border-dark-border">
            {steps.map((step, i) => (
              <div key={step.title} className="flex items-start gap-3.5">
                <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-ink text-[13px] font-bold text-cream dark:bg-dark-text dark:text-dark-bg">
                  {i + 1}
                </div>
                <div>
                  <div className="text-sm font-semibold">{step.title}</div>
                  <div className="text-[13px] text-ink-tertiary dark:text-dark-secondary">{step.body}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="flex-1">
          <ScanPanel onExtracted={setExtractedFields} />
          <ProfileForm
            extractedFields={extractedFields}
            sampleTrigger={sampleTrigger}
            focusField={focusField}
            onSubmit={handleSubmit}
            submitting={submitting}
          />
          {error && <p className="mt-4 text-sm text-error-text">{error}</p>}
        </div>
      </main>

      <GradientBar />
    </div>
  );
}
