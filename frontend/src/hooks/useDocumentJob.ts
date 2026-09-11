import { useCallback, useRef, useState } from "react";
import { getJobStatus, uploadDocument } from "../lib/api";
import type { ExtractedFields } from "../lib/types";

// Two independent triggers for "give up and let the user fill the form
// manually" -- a backend-reported Failed status is one, but it can't cover
// the case where the worker is entirely down and never dequeues the
// message at all. This client-side ceiling covers that case.
const CLIENT_TIMEOUT_MS = 28_000;
const POLL_INTERVAL_MS = 1_500;

export type DocumentJobState =
  | { phase: "idle" }
  | { phase: "uploading" }
  | { phase: "queued" }
  | { phase: "processing" }
  | { phase: "done"; fields: ExtractedFields }
  | { phase: "degraded"; reason: string };

export function useDocumentJob() {
  const [state, setState] = useState<DocumentJobState>({ phase: "idle" });
  const pollHandle = useRef<number | null>(null);
  const timeoutHandle = useRef<number | null>(null);

  const stopPolling = useCallback(() => {
    if (pollHandle.current !== null) {
      window.clearInterval(pollHandle.current);
      pollHandle.current = null;
    }
    if (timeoutHandle.current !== null) {
      window.clearTimeout(timeoutHandle.current);
      timeoutHandle.current = null;
    }
  }, []);

  const degrade = useCallback(
    (reason: string) => {
      stopPolling();
      // "auto-fill unavailable -- fill the form manually" is the entire
      // contract here. The manual form is always live underneath the scan
      // panel regardless of this state, so there is no dead end.
      setState({ phase: "degraded", reason });
    },
    [stopPolling],
  );

  const scan = useCallback(
    async (file: File) => {
      stopPolling();
      setState({ phase: "uploading" });

      let jobId: string;
      try {
        const res = await uploadDocument(file);
        jobId = res.job_id;
      } catch (e) {
        degrade(e instanceof Error ? e.message : "upload failed");
        return;
      }

      setState({ phase: "queued" });

      timeoutHandle.current = window.setTimeout(() => {
        degrade("Extraction is taking too long.");
      }, CLIENT_TIMEOUT_MS);

      pollHandle.current = window.setInterval(async () => {
        try {
          const record = await getJobStatus(jobId);
          if (record === null) return; // not written yet, or already aged out -- keep polling until the timeout

          if (record.status === "processing") {
            setState((prev) => (prev.phase === "done" ? prev : { phase: "processing" }));
            return;
          }
          if (record.status === "done" && record.fields) {
            stopPolling();
            setState({ phase: "done", fields: record.fields });
            return;
          }
          if (record.status === "failed") {
            degrade(record.error ?? "Could not read this document.");
            return;
          }
        } catch (e) {
          // Transient poll failure -- let the client-side timeout be the
          // backstop rather than degrading on a single flaky request.
          console.warn("status poll failed", e);
        }
      }, POLL_INTERVAL_MS);
    },
    [degrade, stopPolling],
  );

  const reset = useCallback(() => {
    stopPolling();
    setState({ phase: "idle" });
  }, [stopPolling]);

  return { state, scan, reset };
}
