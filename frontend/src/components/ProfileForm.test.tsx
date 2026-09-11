import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ProfileForm } from "./ProfileForm";
import { LanguageProvider } from "../lib/i18n/LanguageContext";
import type { ExtractedFields } from "../lib/types";

// GitHub issue #4: the one behavior #8's whole "scan can't clobber manual
// entry" design depends on -- a field the user has actually typed into
// must never be overwritten by a later extraction result, regardless of
// which happened first. Untouched fields must still merge normally, or
// the auto-fill feature wouldn't do anything.

const EXTRACTED: ExtractedFields = {
  age: 42,
  annual_income: 55_000,
  state: "Kerala",
  category: "sc",
  land_holding_acres: 1.5,
  occupation: "Laborer",
};

function renderForm(props: Partial<Parameters<typeof ProfileForm>[0]> = {}) {
  return render(
    <LanguageProvider>
      <ProfileForm
        extractedFields={null}
        sampleTrigger={0}
        onSubmit={vi.fn()}
        submitting={false}
        {...props}
      />
    </LanguageProvider>,
  );
}

describe("ProfileForm auto-fill merge", () => {
  beforeEach(() => {
    // The form now persists to sessionStorage (fixes the "Editing answers"
    // back-navigation state loss) -- clear it so tests don't leak state
    // into each other via the initial-load path.
    sessionStorage.clear();
    // jsdom doesn't implement scrollIntoView -- the focusField behavior
    // calls it, so stub it out rather than let every test using that path
    // throw "not implemented".
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("fills an untouched field from a scan result", () => {
    const { rerender } = renderForm();

    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={EXTRACTED} sampleTrigger={0} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    expect(screen.getByLabelText("State")).toHaveValue("Kerala");
    expect(screen.getByLabelText("Age")).toHaveValue(42);
  });

  it("still lets an untouched field pick up the scan even after a different field was edited", () => {
    const { rerender } = renderForm();

    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "70" } });

    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={EXTRACTED} sampleTrigger={0} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    // Age was touched -- stays at the manual value.
    expect(screen.getByLabelText("Age")).toHaveValue(70);
    // State was never touched -- picks up the scan.
    expect(screen.getByLabelText("State")).toHaveValue("Kerala");
  });

  it("the core no-overwrite case: edit first, then a scan result arrives on the same instance", () => {
    const { rerender } = renderForm();

    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "65" } });
    expect(screen.getByLabelText("Age")).toHaveValue(65);

    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={EXTRACTED} sampleTrigger={0} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    expect(screen.getByLabelText("Age")).toHaveValue(65);
  });

  it("persists answers to sessionStorage so a back-navigation remount doesn't lose them", () => {
    const { unmount } = renderForm();
    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "38" } });

    // Simulates exactly what happens when React Router unmounts this
    // screen (navigating to /results) and later remounts it (navigating
    // back via "Editing answers →") -- a fresh component instance reading
    // from the same sessionStorage.
    unmount();
    renderForm();

    expect(screen.getByLabelText("Age")).toHaveValue(38);
  });

  it("the sample-household shortcut replaces the whole profile, including already-touched fields", () => {
    const { rerender } = renderForm({ sampleTrigger: 0 });
    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "99" } });

    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={null} sampleTrigger={1} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    // Unlike the scan auto-fill, the explicit sample action overrides
    // even a field the user already typed into.
    expect(screen.getByLabelText("Age")).toHaveValue(45);
    expect(screen.getByLabelText("State")).toHaveValue("Goa");
  });
});

describe("ProfileForm OCR normalization", () => {
  beforeEach(() => {
    sessionStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("normalizes a recognized-but-differently-worded occupation into the exact dropdown value", () => {
    const { rerender } = renderForm();

    // EXTRACTED.occupation is "Laborer" (capitalized, not the dropdown's
    // exact "laborer" value) -- the normalizer must still land on it.
    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={EXTRACTED} sampleTrigger={0} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    expect(screen.getByLabelText("Occupation")).toHaveValue("laborer");
    expect(screen.getByLabelText("Category")).toHaveValue("sc");
  });

  it("leaves the field alone rather than guessing when the extracted wording isn't recognized", () => {
    const { rerender } = renderForm();
    const before = (screen.getByLabelText("Occupation") as HTMLSelectElement).value;

    rerender(
      <LanguageProvider>
        <ProfileForm
          extractedFields={{ ...EXTRACTED, occupation: "Freelance Astrologer" }}
          sampleTrigger={0}
          onSubmit={vi.fn()}
          submitting={false}
        />
      </LanguageProvider>,
    );

    // Never injected as free text into a <select> that has no such option.
    expect(screen.getByLabelText("Occupation")).toHaveValue(before);
  });

  it("marks a scan-filled field visually, then clears that mark the moment the user edits it", () => {
    const { rerender } = renderForm();

    rerender(
      <LanguageProvider>
        <ProfileForm extractedFields={EXTRACTED} sampleTrigger={0} onSubmit={vi.fn()} submitting={false} />
      </LanguageProvider>,
    );

    const stateInput = screen.getByLabelText("State");
    expect(stateInput.className).toMatch(/border-teal/);

    fireEvent.change(stateInput, { target: { value: "Karnataka" } });
    expect(stateInput.className).not.toMatch(/border-teal/);
  });
});

describe("ProfileForm area_type field", () => {
  beforeEach(() => {
    sessionStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("only shows the area-type question once the kutcha-house pill is checked", () => {
    renderForm();
    expect(screen.queryByLabelText("Area type (for housing schemes)")).not.toBeInTheDocument();

    fireEvent.click(screen.getByLabelText("Lives in a kutcha house"));
    expect(screen.getByLabelText("Area type (for housing schemes)")).toBeInTheDocument();
  });
});

describe("ProfileForm focusField navigation", () => {
  beforeEach(() => {
    sessionStorage.clear();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("focuses and highlights the field named by focusField on mount", () => {
    renderForm({ focusField: "land_holding_acres" });
    expect(screen.getByLabelText("Land holding (acres, if any)")).toHaveFocus();
  });
});
