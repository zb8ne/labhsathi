import { Link } from "react-router-dom";
import { GradientBar } from "../components/GradientBar";
import { Nav } from "../components/Nav";

// This page is intentionally English-only for now, same precedent as the
// Kafka schema strip on the Privacy screen (SchemaStrip.tsx): it's closer
// to technical documentation than product copy, and a rushed Hindi
// translation of architecture prose risks being wrong rather than useful.
// Worth a real translation pass later, not a blocker for shipping this.
export function HowItWorksScreen() {
  return (
    <div className="flex min-h-screen flex-col bg-cream text-ink dark:bg-dark-bg dark:text-dark-text">
      <Nav />

      <main className="mx-auto w-full max-w-[780px] flex-1 px-6 pb-20 pt-14 sm:px-0">
        <div className="mb-3.5 text-xs font-semibold uppercase tracking-[0.08em] text-terracotta dark:text-amber-tan">
          The full walkthrough
        </div>
        <h1 className="mb-4 font-serif text-[32px] font-semibold leading-tight sm:text-[44px]">
          How LabhSathi actually works.
        </h1>
        <p className="mb-16 max-w-[62ch] text-[16px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
          Two answers to "how does it work," at two depths. First the five-minute version, for anyone
          using it. Then the architecture underneath, for anyone who wants to see why the privacy and
          reliability claims aren't just marketing.
        </p>

        {/* ---------------------------------------------------------- */}
        <SectionKicker>the five-minute version</SectionKicker>
        <h2 className="mb-10 font-serif text-2xl font-semibold sm:text-[28px]">From document to checklist.</h2>

        <JourneyStep
          number={1}
          title="Scan a document to auto-fill six fields, or just type them"
          body="Upload an income certificate, a ration card, an ID — anything with your age, income, occupation, state, category and land holding on it. LabhSathi reads those six fields once, fills them in, and forgets the image existed. The rest of the form (family size and a few household questions a document can't answer) still needs a direct answer either way. No account, no login, nothing saved."
          image="/how-it-works-assets/step-1-form.jpg"
          imageAlt="LabhSathi's main form, pre-filled with a sample household's details"
        />

        <JourneyStep
          number={2}
          title="Every scheme worth checking, in under two seconds"
          body="LabhSathi checks your answers against a real, cited catalog of central government schemes — pensions, health cover, housing, agriculture, financial inclusion — and shows exactly which ones are worth pursuing."
          image="/how-it-works-assets/step-2-results.jpg"
          imageAlt="Results screen showing 28 matched schemes across multiple categories"
        />

        <JourneyStep
          number={3}
          title="Never just a match — always the reason why"
          body="Every card tells you the specific fact about your situation that made it match, so it reads like an explanation, not a lottery ticket. 'Worth checking' is the honest framing: a real signal, not a final determination."
          image="/how-it-works-assets/step-3-explained.png"
          imageAlt="Ayushman Bharat PM-JAY card showing the reason it matched and documents needed"
        />

        <JourneyStep
          number={4}
          title="A checklist you can actually bring with you"
          body="Every matched scheme lists the exact documents it needs and links to the real official source to verify before you rely on it. Print the whole assessment as a single page for the counter at your nearest Common Service Centre."
        />

        {/* ---------------------------------------------------------- */}
        <SectionKicker>under the hood</SectionKicker>
        <h2 className="mb-5 font-serif text-2xl font-semibold sm:text-[28px]">
          Why a document upload doesn't just... call an API.
        </h2>
        <p className="mb-10 max-w-[62ch] text-[15px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
          The earlier version of this did exactly that: one request in, one blocking call to a vision
          model, one response out. Simple, and quietly wrong in three ways — it ties up a request
          thread for the slowest thing the system does, it can't scale document extraction without
          scaling the whole API alongside it, and nothing except code discipline stops a future "just
          cache it for a second" shortcut from making an image outlive the request it arrived in. So
          the document path is its own event-driven pipeline instead:
        </p>

        <h3 className="mb-3 font-serif text-lg font-semibold">The document path, close up</h3>
        <ArchitectureDiagram />

        <h3 className="mb-3 mt-14 font-serif text-lg font-semibold">
          The actual deployment, right now — not a mockup
        </h3>
        <p className="mb-6 max-w-[62ch] text-[15px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
          This is Railway's live service graph for production, pulled directly from the running
          deployment. Seven services, all reporting <span className="font-semibold text-success-accent dark:text-success-accent">Online</span> at
          the same time — the same seven this page has been describing, wired together exactly like this.
        </p>
        <img
          src="/how-it-works-assets/architecture-topology.png"
          alt="Railway's live production service graph: Postgres, catalog-service, redis, kafka, ocr-worker, api-gateway, and frontend, all Online, with their real dependency arrows"
          className="w-full rounded-2xl border border-border dark:border-dark-border"
          loading="lazy"
        />

        <div className="mt-8 grid gap-x-8 gap-y-6 sm:grid-cols-2">
          <ServiceNote name="frontend" tag="labhsathi.info">
            The React app itself. Talks to api-gateway for every scheme-related call, plus Google Fonts
            for the two typefaces on this page — no other third party sees a request from this app.
          </ServiceNote>
          <ServiceNote name="api-gateway" tag="Rust / Axum">
            The only entry point. Accepts uploads, serves{" "}
            <span className="font-mono text-[13px]">/api/match</span>, writes to Redis, publishes to
            Kafka. Never calls the vision API itself.
          </ServiceNote>
          <ServiceNote name="kafka" tag="event bus">
            Connects api-gateway and ocr-worker without either one calling the other directly. Two
            topics: <span className="font-mono text-[13px]">document.jobs.submitted</span> and{" "}
            <span className="font-mono text-[13px]">document.jobs.completed</span>.
          </ServiceNote>
          <ServiceNote name="ocr-worker" tag="Rust, Kafka consumer">
            Stateless and horizontally scalable — this is the one thing that actually scales under
            load. Reads an image exactly once, calls the vision API, forgets the image existed.
          </ServiceNote>
          <ServiceNote name="redis" tag="60s handoff cache">
            Where the raw image sits between api-gateway writing it and ocr-worker's one-time read.
            Persistence is off. Nothing here is meant to survive a minute.
          </ServiceNote>
          <ServiceNote name="catalog-service" tag="Rust / Axum">
            Owns the scheme data, exposes it over its own internal{" "}
            <span className="font-mono text-[13px]">GET /schemes</span>. Adding a scheme is a database
            row here, not a redeploy of the matching engine.
          </ServiceNote>
          <ServiceNote name="Postgres" tag="one JSONB row per scheme">
            What catalog-service actually persists to. Every entry carries its own{" "}
            <span className="font-mono text-[13px]">source_url</span> and last-checked date — the
            catalog cites itself.
          </ServiceNote>
        </div>

        <div className="mt-10 grid gap-8 sm:grid-cols-2">
          <ArchNote title="Redis is a handoff, not a datastore">
            The raw image and the job's status live under separate keys with separate TTLs — 60
            seconds for the image, 5 minutes for the status. The image key is read with{" "}
            <code className="font-mono text-[13px] text-terracotta dark:text-amber-tan">GETDEL</code>,
            which deletes it in the same atomic step that reads it, so there is no window where it sits
            around unread. Even in local dev, Redis runs with persistence off.
          </ArchNote>
          <ArchNote title="Privacy enforced by the type system, not a promise">
            <span className="font-mono text-[13px]">DocumentJobSubmitted</span> and{" "}
            <span className="font-mono text-[13px]">DocumentJobCompleted</span> — the two Kafka event
            schemas — have no field capable of holding image bytes anywhere in their definition. Not a
            rule enforced by a code reviewer; a shape the compiler enforces. See{" "}
            <Link to="/privacy" className="underline decoration-amber-tan decoration-2 underline-offset-4">
              the Privacy page
            </Link>{" "}
            for the honest limit on that claim.
          </ArchNote>
          <ArchNote title="A crash never loses or duplicates a job">
            ocr-worker only commits a Kafka offset after it has published a terminal event — success
            or failure — for that message. Crash before that, and the job redelivers to another
            worker. If that redelivery finds the image already consumed, that's a named, expected
            failure case, not corruption.
          </ArchNote>
          <ArchNote title="Matching never depends on the vision pipeline">
            <span className="font-mono text-[13px]">/api/match</span> is fast, in-memory Rust with one
            cached network hop to catalog-service (30-second TTL, stale-fallback on error) — it doesn't
            know or care whether a profile came from a scan or from typing. If ocr-worker is completely
            down, the manual form still works, because the two were never the same code path to begin
            with.
          </ArchNote>
        </div>

        <div className="mt-14 rounded-2xl border border-border bg-input-bg px-7 py-6 dark:border-dark-border dark:bg-dark-card">
          <h3 className="mb-2 font-serif text-lg font-semibold">The one exception: not everything is a database row</h3>
          <p className="text-[14.5px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
            Most schemes are matched declaratively straight out of Postgres — age/income bounds,
            occupation, category, and similar, no Rust change needed to add one. A handful need a real
            follow-up question instead of a flat match/no-match (PM-KISAN, the NSAP pensions, PMAY's
            urban/rural split), and those still carry a small hand-written rule in{" "}
            <span className="font-mono text-[13px]">labhsathi-core</span>, the shared library both
            api-gateway and catalog-service depend on.
          </p>
        </div>

        {/* ---------------------------------------------------------- */}
        <SectionKicker>what this is, and isn't</SectionKicker>
        <p className="mb-2 max-w-[62ch] text-[15px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
          The scheme catalog is curated, central-government-only, and not exhaustive. Eligibility rules
          are simplified approximations of real criteria — close enough to tell you what's worth
          checking, not close enough to skip verifying with the official source before you act on it.
          Every match says exactly that: "worth checking," never a final determination.
        </p>
        <p className="max-w-[62ch] text-[15px] leading-relaxed text-ink-secondary dark:text-dark-secondary">
          Read the full reasoning in{" "}
          <a
            href="https://github.com/zb8ne/labhsathi/blob/main/docs/adr/0001-event-driven-document-pipeline.md"
            target="_blank"
            rel="noreferrer"
            className="underline decoration-amber-tan decoration-2 underline-offset-4"
          >
            ADR 0001
          </a>{" "}
          or the project's{" "}
          <a
            href="https://github.com/zb8ne/labhsathi"
            target="_blank"
            rel="noreferrer"
            className="underline decoration-amber-tan decoration-2 underline-offset-4"
          >
            README
          </a>
          .
        </p>
      </main>

      <GradientBar />
    </div>
  );
}

function SectionKicker({ children }: { children: React.ReactNode }) {
  return (
    <div className="mb-3 text-xs font-semibold uppercase tracking-[0.08em] text-ink-tertiary dark:text-dark-secondary">
      {children}
    </div>
  );
}

function JourneyStep({
  number,
  title,
  body,
  image,
  imageAlt,
}: {
  number: number;
  title: string;
  body: string;
  image?: string;
  imageAlt?: string;
}) {
  return (
    <div className="mb-14 flex flex-col gap-5 sm:flex-row sm:gap-8">
      <div className="flex-shrink-0">
        <div className="flex h-9 w-9 items-center justify-center rounded-full bg-ink text-sm font-bold text-cream dark:bg-dark-text dark:text-dark-bg">
          {number}
        </div>
      </div>
      <div className="flex-1">
        <h3 className="mb-2 font-serif text-xl font-semibold">{title}</h3>
        <p className="mb-4 max-w-[58ch] text-[15px] leading-relaxed text-ink-secondary dark:text-dark-secondary">{body}</p>
        {image && (
          <img
            src={image}
            alt={imageAlt}
            className="w-full rounded-xl border border-border shadow-sm dark:border-dark-border"
            loading="lazy"
          />
        )}
      </div>
    </div>
  );
}

function ServiceNote({ name, tag, children }: { name: string; tag: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-3">
      <div className="mt-1.5 h-1.5 w-1.5 flex-shrink-0 rounded-full bg-success-accent" />
      <div>
        <div className="mb-1 flex items-baseline gap-2">
          <span className="font-mono text-[13.5px] font-semibold">{name}</span>
          <span className="text-[11px] text-ink-tertiary dark:text-dark-secondary">{tag}</span>
        </div>
        <p className="text-[13.5px] leading-relaxed text-ink-tertiary dark:text-dark-secondary">{children}</p>
      </div>
    </div>
  );
}

function ArchNote({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <div className="mb-1.5 text-[14px] font-semibold">{title}</div>
      <p className="text-[13.5px] leading-relaxed text-ink-tertiary dark:text-dark-secondary">{children}</p>
    </div>
  );
}

function ArchBox({ label, sub, accent }: { label: string; sub: string; accent?: "kafka" | "service" | "cache" }) {
  const tone =
    accent === "kafka"
      ? "border-amber-tan/50 bg-badge-bg/60 dark:border-amber-tan/25 dark:bg-amber-tan/5"
      : accent === "cache"
        ? "border-teal-light/40 bg-teal-light/10 dark:border-[#2d5147] dark:bg-[#1a2e29]"
        : "border-border bg-input-bg dark:border-dark-border dark:bg-dark-card";
  return (
    <div className={`flex w-[132px] flex-shrink-0 flex-col items-center rounded-xl border px-3 py-4 text-center ${tone}`}>
      <div className="mb-0.5 text-[12.5px] font-semibold leading-tight">{label}</div>
      <div className="text-[10.5px] leading-tight text-ink-tertiary dark:text-dark-secondary">{sub}</div>
    </div>
  );
}

function DiagramArrow({ down }: { down?: boolean }) {
  return (
    <div className={`flex flex-shrink-0 items-center justify-center text-ink-tertiary dark:text-dark-secondary ${down ? "h-8 w-[132px] rotate-90" : "w-8"}`}>
      <svg width="24" height="12" viewBox="0 0 24 12" fill="none">
        <path d="M0 6h18M13 1l5 5-5 5" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" />
      </svg>
    </div>
  );
}

function ArchitectureDiagram() {
  return (
    <div className="overflow-x-auto rounded-2xl border border-border bg-white/60 p-6 dark:border-dark-border dark:bg-black/10">
      <div className="flex min-w-[820px] items-center justify-center gap-1">
        <ArchBox label="Upload" sub="api-gateway" />
        <DiagramArrow />
        <ArchBox label="job.submitted" sub="Kafka topic" accent="kafka" />
        <DiagramArrow />
        <ArchBox label="ocr-worker" sub="reads once, forgets" />
        <DiagramArrow />
        <ArchBox label="job.completed" sub="Kafka topic" accent="kafka" />
        <DiagramArrow />
        <ArchBox label="Fields only" sub="age, income, state..." accent="cache" />
      </div>
      <div className="mt-3 flex min-w-[820px] justify-center">
        <div className="text-[11px] text-ink-tertiary dark:text-dark-secondary">
          Redis sits between <span className="font-mono">api-gateway</span> and{" "}
          <span className="font-mono">ocr-worker</span> as a 60-second handoff cache — never a datastore.
        </div>
      </div>
    </div>
  );
}
