import { createContext, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { translations, type Lang } from "./translations";

const STORAGE_KEY = "labhsathi:lang";

function initialLang(): Lang {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "en" || stored === "hi") return stored;
  } catch {
    // localStorage can throw in a private window / blocked storage --
    // fall through to the default rather than crashing the app over a
    // language-preference convenience.
  }
  return "en";
}

interface LanguageContextValue {
  lang: Lang;
  setLang: (lang: Lang) => void;
  t: (typeof translations)["en"];
}

const LanguageContext = createContext<LanguageContextValue | null>(null);

export function LanguageProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(initialLang);

  const setLang = (next: Lang) => {
    setLangState(next);
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // same reasoning as initialLang -- a failed write just means the
      // choice doesn't persist across reloads, not worth surfacing.
    }
  };

  const value = useMemo<LanguageContextValue>(
    () => ({ lang, setLang, t: translations[lang] }),
    [lang],
  );

  return <LanguageContext.Provider value={value}>{children}</LanguageContext.Provider>;
}

export function useLanguage(): LanguageContextValue {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error("useLanguage must be used within a LanguageProvider");
  return ctx;
}

/** Handles the en/hi singular-vs-plural split for "N scheme(s)". */
export function resultsHeading(t: (typeof translations)["en"], count: number): string {
  const template = count === 1 ? t.results.heading_one : t.results.heading_other;
  return template.replace("{{count}}", String(count));
}

/** Same singular/plural split for the "N more need info" line -- only
 * rendered by the caller when count > 0. */
export function needsInfoNote(t: (typeof translations)["en"], count: number): string {
  const template = count === 1 ? t.results.needsInfoNote_one : t.results.needsInfoNote_other;
  return template.replace("{{count}}", String(count));
}
