import { Link } from "react-router-dom";
import { Logo } from "./Logo";

interface NavProps {
  variant: "light" | "dark";
  /** Main screen shows nav links + trust pill; Results shows an "editing" link instead. */
  right?: React.ReactNode;
}

export function Nav({ variant, right }: NavProps) {
  const isDark = variant === "dark";
  return (
    <div
      className={`flex items-center justify-between px-6 py-5 sm:px-14 ${
        isDark ? "" : "border-b border-border"
      }`}
    >
      <Link to="/" className="flex items-center gap-3">
        <Logo />
        <span className="font-serif text-xl font-semibold sm:text-[22px]">LabhSathi</span>
      </Link>
      <div className="flex items-center gap-6">
        {right ?? (
          <>
            <span className={`hidden text-sm font-medium sm:inline ${isDark ? "text-dark-secondary" : "text-ink-secondary"}`}>
              How it works
            </span>
            <Link
              to="/privacy"
              className={`hidden text-sm font-medium sm:inline ${isDark ? "text-dark-secondary" : "text-ink-secondary"}`}
            >
              Privacy
            </Link>
            <TrustPill />
          </>
        )}
      </div>
    </div>
  );
}

function TrustPill() {
  return (
    <div className="flex items-center gap-1.5 rounded-full border border-success-border bg-success-bg px-3.5 py-1.5">
      <div className="h-1.5 w-1.5 rounded-full bg-success-accent" />
      <span className="text-xs font-semibold text-success-text">No documents stored</span>
    </div>
  );
}
