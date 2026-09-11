import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { useDocumentJob } from "../hooks/useDocumentJob";
import { useLanguage } from "../lib/i18n/LanguageContext";
import type { translations } from "../lib/i18n/translations";
import type { ExtractedFields } from "../lib/types";

interface ScanPanelProps {
  onExtracted: (fields: ExtractedFields) => void;
}

export function ScanPanel({ onExtracted }: ScanPanelProps) {
  const { t } = useLanguage();
  const { state, scan } = useDocumentJob();
  const inputRef = useRef<HTMLInputElement>(null);
  // Guards against firing onExtracted again on every unrelated re-render
  // while phase stays "done" -- only fire once per distinct fields object
  // (a new scan() always produces a new object reference).
  const notifiedFields = useRef<ExtractedFields | null>(null);

  useEffect(() => {
    if (state.phase === "done" && state.fields !== notifiedFields.current) {
      notifiedFields.current = state.fields;
      onExtracted(state.fields);
    }
  }, [state, onExtracted]);

  const handleFile = (file: File | undefined) => {
    if (!file) return;
    void scan(file);
    // Clear the input's value so selecting the *same* file again still
    // fires a change event -- browsers don't re-fire onChange for an
    // unchanged value, so without this a retry-with-the-same-file would
    // silently do nothing.
    if (inputRef.current) inputRef.current.value = "";
  };

  return (
    <div className="mb-7 rounded-2xl border border-dashed border-amber-tan bg-[#fdf4ec] px-5 py-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <ScanIcon />
          <div>
            <div className="text-sm font-semibold text-ink">{t.scan.title}</div>
            <StatusLine state={state} t={t} />
          </div>
        </div>
        {/* GitHub issue #3: "done" wasn't in this set, so after a
            successful scan the button vanished for good -- no way to
            rescan a different document or retry after a bad read. */}
        {state.phase === "idle" || state.phase === "degraded" || state.phase === "done" ? (
          <button
            type="button"
            onClick={() => inputRef.current?.click()}
            className="whitespace-nowrap rounded-[10px] bg-terracotta px-4 py-2.5 text-sm font-semibold text-white hover:bg-terracotta-hover"
          >
            {state.phase === "done" ? t.scan.buttonScanAnother : t.scan.buttonScan}
          </button>
        ) : (
          <div className="whitespace-nowrap rounded-[10px] bg-terracotta/40 px-4 py-2.5 text-sm font-semibold text-white">
            {t.scan.buttonWorking}
          </div>
        )}
        <input
          ref={inputRef}
          type="file"
          accept="image/*"
          className="hidden"
          onChange={(e) => handleFile(e.target.files?.[0])}
        />
      </div>
    </div>
  );
}

function StatusLine({
  state,
  t,
}: {
  state: ReturnType<typeof useDocumentJob>["state"];
  t: (typeof translations)["en"];
}) {
  switch (state.phase) {
    case "idle":
      return (
        <div className="text-xs text-[#92400e]">
          {t.scan.hintIdle} &mdash; <Link to="/privacy" className="underline">{t.scan.hintIdleLink}</Link>
        </div>
      );
    case "uploading":
      return <div className="text-xs text-[#92400e]">{t.scan.hintUploading}</div>;
    case "queued":
      return <div className="text-xs text-[#92400e]">{t.scan.hintQueued}</div>;
    case "processing":
      return <div className="text-xs text-[#92400e]">{t.scan.hintProcessing}</div>;
    case "done":
      return <div className="text-xs text-success-text">{t.scan.hintDone}</div>;
    case "degraded":
      return <div className="text-xs text-error-text">{t.scan.hintDegraded}</div>;
  }
}

function ScanIcon() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="#9a3412" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="6" width="18" height="14" rx="2" />
      <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      <circle cx="12" cy="13" r="3.2" />
    </svg>
  );
}
