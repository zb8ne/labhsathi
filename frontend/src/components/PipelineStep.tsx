interface PipelineStepProps {
  number: number;
  title: string;
  description: string;
  /** Tailwind text-color classes for the icon stroke (uses currentColor) --
   * a class, not a raw hex, so a step can read well in both themes (e.g.
   * the teal checkmark is darker on a light background than on dark). */
  iconColorClass: string;
  /** "teal" for the final "fields only" checkmark step (its box picks up a
   * faint teal tint in both themes, matching the icon); everything else
   * uses a plain neutral box. */
  tone?: "neutral" | "teal";
  children: React.ReactNode; // the SVG icon path(s)
}

export function PipelineStep({ number, title, description, iconColorClass, tone = "neutral", children }: PipelineStepProps) {
  const boxClass =
    tone === "teal"
      ? "border-teal-light/40 bg-teal-light/10 dark:border-[#2d5147] dark:bg-[#1a2e29]"
      : "border-border bg-input-bg dark:border-dark-border dark:bg-dark-card";
  return (
    <div className="flex w-full flex-col items-center text-center sm:w-[260px]">
      <div className={`mb-5 flex h-[88px] w-[88px] items-center justify-center rounded-[20px] border ${boxClass}`}>
        <svg
          width="36"
          height="36"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
          className={iconColorClass}
        >
          {children}
        </svg>
      </div>
      <div className="mb-1.5 text-[15px] font-semibold">
        {number}. {title}
      </div>
      <div className="text-[13px] leading-relaxed text-ink-tertiary dark:text-dark-secondary">{description}</div>
    </div>
  );
}

export function PipelineArrow() {
  return (
    <div className="hidden w-[60px] items-center justify-center pt-11 sm:flex">
      <svg width="40" height="16" viewBox="0 0 40 16" fill="none">
        <path
          d="M0 8h34M28 2l6 6-6 6"
          stroke="currentColor"
          strokeWidth="1.6"
          strokeLinecap="round"
          strokeLinejoin="round"
          className="text-ink-tertiary dark:text-dark-secondary"
        />
      </svg>
    </div>
  );
}
