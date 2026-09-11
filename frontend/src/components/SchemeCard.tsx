import type { SchemeMatch } from "../lib/types";

export function SchemeCard({ scheme }: { scheme: SchemeMatch }) {
  return (
    <div className="flex flex-col gap-4 rounded-2xl border border-border bg-white p-6">
      <div className="flex items-start justify-between gap-3">
        <div>
          <div className="mb-1.5 text-[11px] font-bold uppercase tracking-wide text-terracotta">
            {scheme.authority}
          </div>
          <div className="font-serif text-lg font-semibold">{scheme.name}</div>
        </div>
        <div className="whitespace-nowrap rounded-full border border-success-border bg-success-bg px-3 py-1.5 text-xs font-semibold text-success-text">
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
