import { flushSync } from "react-dom";

function prefersReducedMotion(): boolean {
  try {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    return false;
  }
}

/** True when the browser supports the View Transitions API and the user
 * hasn't asked for reduced motion -- the only conditions under which
 * withViewTransition actually animates rather than applying the DOM
 * update directly. */
function canUseViewTransitions(): boolean {
  return typeof document !== "undefined" && "startViewTransition" in document && !prefersReducedMotion();
}

/** Runs `update` inside a View Transition so the DOM change (e.g. swapping
 * every piece of text on language toggle) crossfades instead of popping.
 * Falls back to calling `update` directly when transitions aren't
 * supported or the user prefers reduced motion -- an instant swap there is
 * correct, not degraded.
 *
 * `update` is wrapped in `flushSync` so the DOM is actually updated by the
 * time this callback returns, which is what the View Transition API needs
 * to capture the right "after" snapshot -- React state setters don't
 * commit synchronously on their own. */
export function withViewTransition(update: () => void): void {
  if (!canUseViewTransitions()) {
    update();
    return;
  }
  document.startViewTransition(() => flushSync(update));
}

const LIGHT_BG = "#faf7f2"; // tailwind.config.ts `cream`
const DARK_BG = "#1c1917"; // tailwind.config.ts `dark-bg`

/** Theme toggle "iris": an opaque overlay in the *outgoing* theme's flat
 * background color, clipped to a circle centered at `(x, y)`, that shrinks
 * from covering the whole viewport down to nothing right at that point --
 * revealing the already-switched theme underneath as it recedes. `growing`
 * true means switching to the lighter theme (the overlay is the dark
 * color receding); false means switching to the darker theme (the overlay
 * is the light color receding). Falls back to an instant `update()` call
 * when the user prefers reduced motion.
 *
 * Deliberately NOT the View Transitions API. Three attempts at that
 * (JS Element.animate() after transition.ready, then a CSS @keyframes
 * animation, then flushSync/useLayoutEffect to fix snapshot timing) never
 * visibly fixed the reported flicker -- and this automation environment
 * can't run View Transitions at all to verify further (Chrome aborts them
 * on a backgrounded tab, which every tab this tool drives is). Rather than
 * keep shipping unverified fixes against a browser API with that much
 * snapshot-timing subtlety, this reimplements the same visual effect with
 * a plain DOM element and Element.animate() on a real (non-pseudo)
 * element -- fully within our control, and actually testable via
 * screenshots regardless of tab visibility. */
export function withIrisTransition(update: () => void, x: number, y: number, growing: boolean): void {
  if (prefersReducedMotion()) {
    update();
    return;
  }

  // The real theme swap happens immediately -- it's invisible under the
  // opaque overlay until the overlay recedes past it, so there's no flash
  // waiting on it; and not gating it behind any animation lifecycle event
  // removes the exact class of timing bug the View Transitions attempts
  // kept running into.
  update();

  const overlay = document.createElement("div");
  overlay.setAttribute("aria-hidden", "true");
  Object.assign(overlay.style, {
    position: "fixed",
    inset: "0",
    zIndex: "2147483647",
    pointerEvents: "none",
    background: growing ? DARK_BG : LIGHT_BG,
  });
  document.body.appendChild(overlay);

  const radius = Math.hypot(Math.max(x, window.innerWidth - x), Math.max(y, window.innerHeight - y));

  const animation = overlay.animate(
    [{ clipPath: `circle(${radius}px at ${x}px ${y}px)` }, { clipPath: `circle(0px at ${x}px ${y}px)` }],
    { duration: 550, easing: "ease-in-out", fill: "forwards" },
  );

  animation.finished
    .catch(() => {
      // Rejects if the animation was cancelled rather than completing --
      // the overlay still needs removing either way.
    })
    .finally(() => overlay.remove());
}
