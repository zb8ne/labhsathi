import type { JobStatusRecord, MatchResponse, UserProfile } from "./types";

declare global {
  interface Window {
    __LABHSATHI_CONFIG__?: { apiBaseUrl: string };
  }
}

function apiBase(): string {
  return window.__LABHSATHI_CONFIG__?.apiBaseUrl ?? "/api";
}

export async function matchSchemes(profile: UserProfile): Promise<MatchResponse> {
  const res = await fetch(`${apiBase()}/match`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(profile),
  });
  if (!res.ok) {
    throw new Error(`match request failed: ${res.status}`);
  }
  return res.json();
}

export interface UploadResponse {
  job_id: string;
}

export async function uploadDocument(file: File): Promise<UploadResponse> {
  const form = new FormData();
  form.append("document", file);

  const res = await fetch(`${apiBase()}/documents`, { method: "POST", body: form });
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: `upload failed: ${res.status}` }));
    throw new Error(body.error ?? `upload failed: ${res.status}`);
  }
  return res.json();
}

export async function getJobStatus(jobId: string): Promise<JobStatusRecord | null> {
  const res = await fetch(`${apiBase()}/documents/${jobId}`);
  if (res.status === 404) return null;
  if (!res.ok) {
    throw new Error(`status request failed: ${res.status}`);
  }
  return res.json();
}
