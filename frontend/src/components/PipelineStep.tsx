interface PipelineStepProps {
  number: number;
  title: string;
  description: string;
  iconColor: string;
  iconBg?: string;
  iconBorder?: string;
  children: React.ReactNode; // the SVG icon path(s)
}

export function PipelineStep({
  number,
  title,
  description,
  iconColor,
  iconBg = "#292524",
  iconBorder = "#44403c",
  children,
}: PipelineStepProps) {
  return (
    <div className="flex w-full flex-col items-center text-center sm:w-[260px]">
      <div
        className="mb-5 flex h-[88px] w-[88px] items-center justify-center rounded-[20px] border"
        style={{ background: iconBg, borderColor: iconBorder }}
      >
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke={iconColor} strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
          {children}
        </svg>
      </div>
      <div className="mb-1.5 text-[15px] font-semibold">
        {number}. {title}
      </div>
      <div className="text-[13px] leading-relaxed text-dark-secondary">{description}</div>
    </div>
  );
}

export function PipelineArrow() {
  return (
    <div className="hidden w-[60px] items-center justify-center pt-11 sm:flex">
      <svg width="40" height="16" viewBox="0 0 40 16" fill="none">
        <path d="M0 8h34M28 2l6 6-6 6" stroke="#57534e" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}
