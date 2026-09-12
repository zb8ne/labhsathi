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
 * correct, not degraded. */
export function withViewTransition(update: () => void): void {
  if (!canAnimate()) {
    update();
    return;
  }
  document.startViewTransition(update);
}

/** Runs `update` inside a View Transition, then animates an expanding or
 * contracting circle clipped to the *new* or *old* page snapshot,
 * originating at `(x, y)` -- the classic "iris" theme toggle. `growing`
 * true means the incoming view floods outward from the origin (used for
 * switching to the brighter theme); false means the outgoing view
 * collapses inward to the origin, uncovering the new view everywhere else
 * (used for switching to the darker theme). Falls back to a plain `update`
 * call under the same conditions as `withViewTransition`. */
export function withIrisTransition(update: () => void, x: number, y: number, growing: boolean): void {
  if (!canAnimate()) {
    update();
    return;
  }

  const root = document.documentElement;
  root.dataset.irisTransition = growing ? "grow" : "shrink";

  const endRadius = Math.hypot(Math.max(x, window.innerWidth - x), Math.max(y, window.innerHeight - y));
  const clipPath = [`circle(0px at ${x}px ${y}px)`, `circle(${endRadius}px at ${x}px ${y}px)`];

  const transition = document.startViewTransition(update);

  transition.ready
    .then(() => {
      root.animate(
        { clipPath: growing ? clipPath : [...clipPath].reverse() },
        {
          duration: 550,
          easing: "ease-in-out",
          pseudoElement: growing ? "::view-transition-new(root)" : "::view-transition-old(root)",
        },
      );
    })
    .catch(() => {
      // transition.ready rejects if the transition was skipped (e.g. the
      // document went hidden mid-transition) -- the DOM update itself
      // already happened via startViewTransition's callback, so there's
      // nothing left to recover here.
    });

  transition.finished.finally(() => {
    delete root.dataset.irisTransition;
  });
}
