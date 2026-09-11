import { Link } from "react-router-dom";
import { Logo } from "./Logo";
import { useLanguage } from "../lib/i18n/LanguageContext";
import { useTheme } from "../lib/theme/ThemeContext";
import type { Lang } from "../lib/i18n/translations";

interface NavProps {
  /** Main screen shows nav links + trust pill; Results shows an "editing" link instead. */
  right?: React.ReactNode;
}

export function Nav({ right }: NavProps) {
  const { t, lang, setLang } = useLanguage();
  return (
    <div className="flex items-center justify-between gap-3 border-b border-border px-6 py-5 dark:border-dark-border sm:px-14">
      <Link to="/" className="flex shrink-0 items-center gap-3">
        <Logo />
        {/* Wordmark hidden below sm: the icon alone plus a `right` slot
            (e.g. Results' "Editing answers" link) collides with it on a
            360px-class phone otherwise -- verified by screenshot. */}
        <span className="hidden font-serif text-xl font-semibold sm:inline sm:text-[22px]">LabhSathi</span>
      </Link>
      <div className="flex items-center gap-3 sm:gap-6">
        {right ?? (
          <>
            <span className="hidden text-sm font-medium text-ink-secondary dark:text-dark-secondary sm:inline">
              {t.nav.howItWorks}
            </span>
            <Link to="/privacy" className="hidden text-sm font-medium text-ink-secondary dark:text-dark-secondary sm:inline">
              {t.nav.privacy}
            </Link>
            <TrustPill label={t.nav.noDocumentsStored} />
          </>
        )}
        <ThemeToggle />
        <LanguageToggle lang={lang} setLang={setLang} />
      </div>
    </div>
  );
}

/** Subtle icon-only toggle -- shows the icon for the theme you're currently
 * in (sun while light, moon while dark), same as the trust pill/language
 * toggle's understated pill styling rather than a loud switch control. */
function ThemeToggle() {
  const { theme, toggleTheme } = useTheme();
  const isDark = theme === "dark";
  return (
    <button
      type="button"
      onClick={toggleTheme}
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
      title={isDark ? "Switch to light mode" : "Switch to dark mode"}
      className="flex h-7 w-7 items-center justify-center rounded-full border border-border text-ink-tertiary transition-colors hover:text-ink dark:border-dark-border dark:text-dark-secondary dark:hover:text-dark-text"
    >
      {isDark ? <MoonIcon /> : <SunIcon />}
    </button>
  );
}

function SunIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.2 4.2l1.4 1.4M18.4 18.4l1.4 1.4M2 12h2M20 12h2M4.2 19.8l1.4-1.4M18.4 5.6l1.4-1.4" />
    </svg>
  );
}

function MoonIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M20 14.5A8.5 8.5 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5Z" />
    </svg>
  );
}

function LanguageToggle({ lang, setLang }: { lang: Lang; setLang: (lang: Lang) => void }) {
  const base = "rounded-full px-2.5 py-1 text-xs font-semibold transition-colors";
  const active = "bg-ink text-cream dark:bg-dark-text dark:text-dark-bg";
  const inactive = "text-ink-tertiary hover:text-ink dark:text-dark-secondary dark:hover:text-dark-text";

  const cls = (isActive: boolean) => `${base} ${isActive ? active : inactive}`;

  return (
    <div className="flex items-center gap-0.5 rounded-full border border-border px-0.5 py-0.5 dark:border-dark-border">
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
    <div className="flex items-center gap-1.5 rounded-full border border-success-border bg-success-bg px-3.5 py-1.5 dark:border-success-border/40 dark:bg-success-bg/10">
      <div className="h-1.5 w-1.5 rounded-full bg-success-accent" />
      <span className="text-xs font-semibold text-success-text dark:text-success-accent">{label}</span>
    </div>
  );
}
