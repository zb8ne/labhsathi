import { Link } from "react-router-dom";
import { Logo } from "./Logo";
import { useLanguage } from "../lib/i18n/LanguageContext";
import type { Lang } from "../lib/i18n/translations";

interface NavProps {
  variant: "light" | "dark";
  /** Main screen shows nav links + trust pill; Results shows an "editing" link instead. */
  right?: React.ReactNode;
}

export function Nav({ variant, right }: NavProps) {
  const { t, lang, setLang } = useLanguage();
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
              {t.nav.howItWorks}
            </span>
            <Link
              to="/privacy"
              className={`hidden text-sm font-medium sm:inline ${isDark ? "text-dark-secondary" : "text-ink-secondary"}`}
            >
              {t.nav.privacy}
            </Link>
            <TrustPill label={t.nav.noDocumentsStored} />
          </>
        )}
        <LanguageToggle lang={lang} setLang={setLang} isDark={isDark} />
      </div>
    </div>
  );
}

function LanguageToggle({
  lang,
  setLang,
  isDark,
}: {
  lang: Lang;
  setLang: (lang: Lang) => void;
  isDark: boolean;
}) {
  const base = "rounded-full px-2.5 py-1 text-xs font-semibold transition-colors";
  const activeLight = "bg-ink text-cream";
  const inactiveLight = "text-ink-tertiary hover:text-ink";
  const activeDark = "bg-dark-text text-dark-bg";
  const inactiveDark = "text-dark-secondary hover:text-dark-text";

  const cls = (isActive: boolean) => {
    if (isDark) return `${base} ${isActive ? activeDark : inactiveDark}`;
    return `${base} ${isActive ? activeLight : inactiveLight}`;
  };

  return (
    <div
      className={`flex items-center gap-0.5 rounded-full border px-0.5 py-0.5 ${
        isDark ? "border-dark-border" : "border-border"
      }`}
    >
      <button type="button" onClick={() => setLang("en")} className={cls(lang === "en")} aria-pressed={lang === "en"}>
        EN
      </button>
      <button type="button" onClick={() => setLang("hi")} className={cls(lang === "hi")} aria-pressed={lang === "hi"}>
        हिं
      </button>
    </div>
  );
}

function TrustPill({ label }: { label: string }) {
  return (
    <div className="flex items-center gap-1.5 rounded-full border border-success-border bg-success-bg px-3.5 py-1.5">
      <div className="h-1.5 w-1.5 rounded-full bg-success-accent" />
      <span className="text-xs font-semibold text-success-text">{label}</span>
    </div>
  );
}
