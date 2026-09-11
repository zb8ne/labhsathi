import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";
import { ProfileForm } from "../components/ProfileForm";
import { ScanPanel } from "../components/ScanPanel";
import { matchSchemes } from "../lib/api";
import type { ExtractedFields, UserProfile } from "../lib/types";

const STEPS = [
  { title: "Tell us your situation", body: "Income, occupation, family, category — takes under a minute." },
  { title: "We match you against real schemes", body: "PM-KISAN, Ayushman Bharat, pensions, scholarships and more." },
  { title: "Get your document checklist", body: "Know exactly what to bring, before you go anywhere." },
];

export function MainScreen() {
  const [extractedFields, setExtractedFields] = useState<ExtractedFields | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  async function handleSubmit(profile: UserProfile) {
    setSubmitting(true);
    setError(null);
    try {
      const result = await matchSchemes(profile);
      navigate("/results", { state: result });
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not match schemes. Please try again.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="flex min-h-screen flex-col bg-cream">
      <Nav variant="light" />

      <main className="flex flex-1 flex-col gap-12 px-6 py-10 sm:flex-row sm:px-14 sm:py-14">
        <div className="sm:w-[420px] sm:flex-shrink-0">
          <div className="mb-6 inline-block rounded-full border border-badge-border bg-badge-bg px-3 py-1.5 text-xs font-semibold tracking-wide text-[#92400e]">
            JAN JEEVAN · WELFARE SCHEME DISCOVERY
          </div>
          <h1 className="mb-5 font-serif text-4xl font-semibold leading-[1.05] tracking-tight sm:text-[52px]">
            Know what
            <br />
            you&rsquo;re <span className="italic text-terracotta">entitled to.</span>
          </h1>
          <p className="mb-8 max-w-[38ch] text-[17px] leading-relaxed text-ink-muted">
            Answer a few questions or scan a document. We match you against real central government
            schemes — and your document is never stored, not even for a second longer than it takes to
            read it.
          </p>

          <div className="flex flex-col gap-4 border-t border-border pt-7">
            {STEPS.map((step, i) => (
              <div key={step.title} className="flex items-start gap-3.5">
                <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-ink text-[13px] font-bold text-cream">
                  {i + 1}
                </div>
                <div>
                  <div className="text-sm font-semibold">{step.title}</div>
                  <div className="text-[13px] text-ink-tertiary">{step.body}</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        <div className="flex-1">
          <ScanPanel onExtracted={setExtractedFields} />
          <ProfileForm extractedFields={extractedFields} onSubmit={handleSubmit} submitting={submitting} />
          {error && <p className="mt-4 text-sm text-error-text">{error}</p>}
        </div>
      </main>

      <GradientBar />
    </div>
  );
}
