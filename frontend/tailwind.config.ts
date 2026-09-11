import type { Config } from "tailwindcss";

// Every value here is transcribed directly from the published LabhSathi
// visual identity artboards -- see docs/CONVENTIONS.md if these ever need
// to change; don't introduce a second ad-hoc palette in a component.
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        serif: ["Newsreader", "ui-serif", "Georgia", "serif"],
        sans: ["'IBM Plex Sans'", "ui-sans-serif", "system-ui", "sans-serif"],
        mono: ["'IBM Plex Mono'", "ui-monospace", "monospace"],
      },
      colors: {
        cream: "#faf7f2",
        ink: "#1c1917",
        "ink-secondary": "#57534e",
        "ink-tertiary": "#78716c",
        "ink-muted": "#44403c",
        border: "#e7e0d4",
        "input-bg": "#faf9f7",

        terracotta: { DEFAULT: "#9a3412", hover: "#7c2d12" },
        amber: { DEFAULT: "#fbbf24", orange: "#fb923c", tan: "#d6a679" },
        badge: { bg: "#fef3c7", border: "#fde68a" },
        success: { text: "#065f46", accent: "#059669", bg: "#ecfdf5", border: "#a7f3d0" },
        teal: { DEFAULT: "#0f766e", light: "#5eead4" },

        // dark screen (Privacy) only
        "dark-bg": "#1c1917",
        "dark-text": "#faf7f2",
        "dark-secondary": "#a8a29e",
        "dark-border": "#44403c",
        "dark-border-soft": "#292524",
        "dark-card": "#292524",

        error: { text: "#f87171", light: "#fca5a5", border: "#7c2d2d", bg: "#3f1d1d" },
      },
      backgroundImage: {
        "gradient-bar": "linear-gradient(90deg,#9a3412,#c2703d,#0f766e)",
      },
    },
  },
  plugins: [],
} satisfies Config;
