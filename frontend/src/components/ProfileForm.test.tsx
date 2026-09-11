import { describe, expect, it, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ProfileForm } from "./ProfileForm";
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

describe("ProfileForm auto-fill merge", () => {
  it("fills an untouched field from a scan result", () => {
    const { rerender } = render(
      <ProfileForm extractedFields={null} onSubmit={vi.fn()} submitting={false} />,
    );

    rerender(<ProfileForm extractedFields={EXTRACTED} onSubmit={vi.fn()} submitting={false} />);

    expect(screen.getByLabelText("State")).toHaveValue("Kerala");
    expect(screen.getByLabelText("Age")).toHaveValue(42);
  });

  it("still lets an untouched field pick up the scan even after a different field was edited", () => {
    const { rerender } = render(
      <ProfileForm extractedFields={null} onSubmit={vi.fn()} submitting={false} />,
    );

    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "70" } });

    rerender(<ProfileForm extractedFields={EXTRACTED} onSubmit={vi.fn()} submitting={false} />);

    // Age was touched -- stays at the manual value.
    expect(screen.getByLabelText("Age")).toHaveValue(70);
    // State was never touched -- picks up the scan.
    expect(screen.getByLabelText("State")).toHaveValue("Kerala");
  });

  it("the core no-overwrite case: edit first, then a scan result arrives on the same instance", () => {
    const { rerender } = render(
      <ProfileForm extractedFields={null} onSubmit={vi.fn()} submitting={false} />,
    );

    fireEvent.change(screen.getByLabelText("Age"), { target: { value: "65" } });
    expect(screen.getByLabelText("Age")).toHaveValue(65);

    rerender(<ProfileForm extractedFields={EXTRACTED} onSubmit={vi.fn()} submitting={false} />);

    expect(screen.getByLabelText("Age")).toHaveValue(65);
  });
});
