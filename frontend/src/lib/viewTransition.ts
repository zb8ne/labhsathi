import { flushSync } from "react-dom";

/** True when the browser supports the View Transitions API and the user
 * hasn't asked for reduced motion -- the only conditions under which any of
 * the helpers below actually animate rather than applying the DOM update
 * directly. */
function canAnimate(): boolean {
  if (typeof document === "undefined" || !("startViewTransition" in document)) return false;
  try {
    return !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  } catch {
    return true;
  }
}

/** Runs `update` inside a View Transition so the DOM change (e.g. swapping
 * every piece of text on language toggle) crossfades instead of popping.
 * Falls back to calling `update` directly when transitions aren't
 * supported or the user prefers reduced motion -- an instant swap there is
 * correct, not degraded.
 *
 * `update` is wrapped in `flushSync`: React state setters don't commit to
 * the DOM synchronously on their own (React 18 batches them into a
 * microtask), but the View Transition API needs the "after" DOM in place
 * the instant this callback returns so it can capture the right snapshot.
 * Without flushSync it captures a stale "after" frame, the transition
 * plays against that stale frame, and then React's real (unanimated)
 * commit lands right after and snaps the page the rest of the way --
 * which reads as a flash/flicker layered on top of the intended
 * animation, not a clean transition. */
export function withViewTransition(update: () => void): void {
  if (!canAnimate()) {
    update();
    return;
  }
  document.startViewTransition(() => flushSync(update));
}

/** Runs `update` inside a View Transition, then reveals or conceals a
 * circle clipped to the *new* or *old* page snapshot, originating at
 * `(x, y)` -- the classic "iris" theme toggle. `growing` true means the
 * incoming view floods outward from the origin (used for switching to the
 * brighter theme); false means the outgoing view collapses inward to the
 * origin, uncovering the new view everywhere else (used for switching to
 * the darker theme). Falls back to a plain `update` call under the same
 * conditions as `withViewTransition`.
 *
 * The actual clip-path animation is a plain CSS `@keyframes` (see
 * index.css) driven by CSS custom properties set synchronously below,
 * not a JS `Element.animate()` call made inside `transition.ready.then()`.
 * That async gap was the real bug behind the flicker/flash reports: the
 * pseudo-element tree can paint at least one unclipped frame (the new
 * theme shown fully, full-bleed) before the `.then()` microtask gets a
 * chance to attach the animation, and that stray frame is what reads as
 * a flash originating from wherever the browser happens to paint first,
 * not from the click point. A CSS animation declared ahead of time in the
 * stylesheet starts the instant the pseudo-element exists, with no gap. */
export function withIrisTransition(update: () => void, x: number, y: number, growing: boolean): void {
  if (!canAnimate()) {
    update();
    return;
  }

  const root = document.documentElement;
  const radius = Math.hypot(Math.max(x, window.innerWidth - x), Math.max(y, window.innerHeight - y));
  root.style.setProperty("--iris-x", `${x}px`);
  root.style.setProperty("--iris-y", `${y}px`);
  root.style.setProperty("--iris-radius", `${radius}px`);
  root.dataset.irisTransition = growing ? "grow" : "shrink";

  const transition = document.startViewTransition(() => flushSync(update));

  transition.finished.finally(() => {
    delete root.dataset.irisTransition;
    root.style.removeProperty("--iris-x");
    root.style.removeProperty("--iris-y");
    root.style.removeProperty("--iris-radius");
  });
}
