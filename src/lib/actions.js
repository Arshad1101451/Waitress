// Svelte actions replacing Flet's on_hover/GestureDetector wrappers
// (theme.py's hover_grow / _press_gesture). Actions are the natural Svelte
// equivalent: `use:hoverGrow` / `use:pressable` on any element.

/** Grow + recolor on mouse hover. Mirrors theme.hover_grow().
 *
 * IMPORTANT: a plain CSS `color` change on the wrapper only has a visible
 * effect on descendants that DON'T set their own explicit `color` — several
 * rows in Admin (item names, prices, usernames, etc.) do set an explicit
 * color, so the color half of this action is a no-op for them. The `scale`
 * transform, on the other hand, always visibly grows the whole subtree
 * regardless of any child's own color/font-size — so it's on by default
 * here (not opt-in) to guarantee every `use:hoverGrow` site actually shows
 * a visible "grows on mouseOver" effect, matching what was asked for. */
export function hoverGrow(node, { color = "var(--pink)", scale = 1.035 } = {}) {
  const baseColor = getComputedStyle(node).color;
  const baseTransform = node.style.transform || "";
  node.style.transition = "color 120ms ease, transform 120ms ease";
  node.style.transformOrigin = node.style.transformOrigin || "left center";

  function onEnter() {
    node.style.color = color;
    if (scale !== 1.0) node.style.transform = `scale(${scale})`;
  }
  function onLeave() {
    node.style.color = baseColor;
    node.style.transform = baseTransform;
  }
  node.addEventListener("mouseenter", onEnter);
  node.addEventListener("mouseleave", onLeave);
  return {
    destroy() {
      node.removeEventListener("mouseenter", onEnter);
      node.removeEventListener("mouseleave", onLeave);
    },
  };
}

/** Hover-lift + press-shrink, used by every fancy button / glow card /
 * tappable row. Mirrors theme._press_gesture(). */
export function pressable(node, { hoverScale = 1.03, pressScale = 0.93 } = {}) {
  node.style.transition = "transform 110ms ease-out, box-shadow 110ms ease-out";
  node.style.cursor = "pointer";

  function onEnter() {
    node.style.transform = `scale(${hoverScale})`;
    node.style.boxShadow = "var(--shadow-strong)";
  }
  function onLeave() {
    node.style.transform = "scale(1)";
    node.style.boxShadow = "var(--shadow-soft)";
  }
  function onDown() {
    node.style.transform = `scale(${pressScale})`;
  }
  function onUp() {
    node.style.transform = `scale(${hoverScale})`;
  }
  node.addEventListener("mouseenter", onEnter);
  node.addEventListener("mouseleave", onLeave);
  node.addEventListener("mousedown", onDown);
  node.addEventListener("mouseup", onUp);
  return {
    destroy() {
      node.removeEventListener("mouseenter", onEnter);
      node.removeEventListener("mouseleave", onLeave);
      node.removeEventListener("mousedown", onDown);
      node.removeEventListener("mouseup", onUp);
    },
  };
}
