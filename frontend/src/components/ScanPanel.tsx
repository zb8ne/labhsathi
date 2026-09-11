import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { useDocumentJob } from "../hooks/useDocumentJob";
import type { ExtractedFields } from "../lib/types";

interface ScanPanelProps {
  onExtracted: (fields: ExtractedFields) => void;
}

export function ScanPanel({ onExtracted }: ScanPanelProps) {
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
  };

  return (
    <div className="mb-7 rounded-2xl border border-dashed border-amber-tan bg-[#fdf4ec] px-5 py-4">
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <ScanIcon />
          <div>
            <div className="text-sm font-semibold text-ink">Scan a document to auto-fill</div>
            <StatusLine state={state} />
          </div>
        </div>
        {state.phase === "idle" || state.phase === "degraded" ? (
          <button
            type="button"
            onClick={() => inputRef.current?.click()}
            className="whitespace-nowrap rounded-[10px] bg-terracotta px-4 py-2.5 text-sm font-semibold text-white hover:bg-terracotta-hover"
          >
            Scan
          </button>
        ) : (
          <div className="whitespace-nowrap rounded-[10px] bg-terracotta/40 px-4 py-2.5 text-sm font-semibold text-white">
            Working&hellip;
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

function StatusLine({ state }: { state: ReturnType<typeof useDocumentJob>["state"] }) {
  switch (state.phase) {
    case "idle":
      return (
        <div className="text-xs text-[#92400e]">
          Processed once, then discarded &mdash; <Link to="/privacy" className="underline">see how &rarr;</Link>
        </div>
      );
    case "uploading":
      return <div className="text-xs text-[#92400e]">Uploading&hellip;</div>;
    case "queued":
      return <div className="text-xs text-[#92400e]">Job submitted, waiting for a worker&hellip;</div>;
    case "processing":
      return <div className="text-xs text-[#92400e]">Extracting fields&hellip;</div>;
    case "done":
      return <div className="text-xs text-success-text">Fields filled in below.</div>;
    case "degraded":
      return (
        <div className="text-xs text-error-text">
          Auto-fill unavailable &mdash; fill the form manually below.
        </div>
      );
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
