import type { SchemeMatch } from "../lib/types";

export function SchemeCard({ scheme }: { scheme: SchemeMatch }) {
  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-border bg-white p-6">
      {/* GitHub issue #3: a long benefit string (e.g. "₹5,00,000/family/year
          cashless health insurance") with a nowrap badge in a row that
          never wraps forced horizontal overflow on phone widths. Stacks
          below sm; the badge itself is also allowed to wrap once stacked. */}
      <div className="flex flex-col items-start gap-2 sm:flex-row sm:items-start sm:justify-between sm:gap-3">
        <div>
          <div className="mb-1.5 text-[11px] font-bold uppercase tracking-wide text-terracotta">
            {scheme.authority}
          </div>
          <div className="font-serif text-lg font-semibold">{scheme.name}</div>
        </div>
        <div className="rounded-full border border-success-border bg-success-bg px-3 py-1.5 text-xs font-semibold text-success-text sm:whitespace-nowrap">
          {scheme.benefit}
        </div>
      </div>

      <div className="rounded-[10px] border-l-[3px] border-amber-tan bg-input-bg px-4 py-3.5 text-[13.5px] leading-relaxed text-ink-muted">
        {scheme.reason}
      </div>

      <div>
        <div className="mb-2 text-[11px] font-bold uppercase tracking-wide text-ink-tertiary">
          Documents needed
        </div>
        <div className="flex flex-wrap gap-1.5">
          {scheme.documents.map((doc) => (
            <span key={doc} className="rounded-full bg-[#f5f5f4] px-2.5 py-1.5 text-xs text-ink-muted">
              {doc}
            </span>
          ))}
        </div>
      </div>

      <p className="text-xs text-ink-tertiary">{scheme.official_note}</p>
    </div>
  );
}
