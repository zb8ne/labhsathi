import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";
import { PipelineArrow, PipelineStep } from "../components/PipelineStep";
import { SchemaStrip } from "../components/SchemaStrip";
import { useLanguage } from "../lib/i18n/LanguageContext";

export function PrivacyScreen() {
  const { t } = useLanguage();
  return (
    <div className="flex min-h-screen flex-col bg-cream text-ink dark:bg-dark-bg dark:text-dark-text">
      <Nav />

      <main className="flex-1 px-6 pb-8 pt-4 text-center sm:px-14">
        <div className="mb-3.5 text-xs font-semibold uppercase tracking-[0.08em] text-terracotta dark:text-amber-tan">
          {t.privacy.kicker}
        </div>
        <h1 className="mx-auto mb-3 max-w-3xl font-serif text-[28px] font-semibold sm:text-[38px]">
          {t.privacy.headline}
        </h1>
        <p className="mx-auto mb-12 max-w-[56ch] text-[15px] text-ink-secondary dark:text-dark-secondary sm:mb-14">{t.privacy.subhead}</p>

        <div className="mx-auto flex max-w-5xl flex-col items-center gap-8 sm:flex-row sm:items-stretch sm:justify-center sm:gap-0">
          <PipelineStep number={1} title={t.privacy.step1Title} iconColorClass="text-amber-orange dark:text-amber" description={t.privacy.step1Body}>
            <rect x="3" y="6" width="18" height="14" rx="2" />
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
            <circle cx="12" cy="13" r="3.2" />
          </PipelineStep>
          <PipelineArrow />
          <PipelineStep number={2} title={t.privacy.step2Title} iconColorClass="text-amber-orange" description={t.privacy.step2Body}>
            <circle cx="12" cy="12" r="3" />
            <path d="M12 2v3M12 19v3M4.2 4.2l2.1 2.1M17.7 17.7l2.1 2.1M2 12h3M19 12h3M4.2 19.8l2.1-2.1M17.7 6.3l2.1-2.1" />
          </PipelineStep>
          <PipelineArrow />
          <PipelineStep number={3} title={t.privacy.step3Title} iconColorClass="text-error-text" description={t.privacy.step3Body}>
            <path d="M4 7h16M9 7V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2m-9 0 1 12a2 2 0 0 0 2 2h4a2 2 0 0 0 2-2l1-12" />
          </PipelineStep>
          <PipelineArrow />
          <PipelineStep number={4} title={t.privacy.step4Title} iconColorClass="text-teal dark:text-teal-light" tone="teal" description={t.privacy.step4Body}>
            <path d="M20 6 9 17l-5-5" />
          </PipelineStep>
        </div>
      </main>

      <SchemaStrip />
      <div className="h-14 sm:h-0" />
      <GradientBar />
    </div>
  );
}
