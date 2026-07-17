## Context

Clarus is a Tauri desktop app whose UI is a single React component (`packages/app/src/App.tsx`) styled by one global stylesheet (`packages/app/src/styles/global.css`). The layout is a CSS grid:

```css
.app-shell {
  display: grid;
  grid-template-columns: 288px minmax(520px, 1fr) 340px;
  min-height: 100vh;   /* grows past the viewport */
}
```

Because `.app-shell` (and `body`) use `min-height: 100vh` and the shell has no `overflow`, tall content in any column stretches the grid past the viewport and the whole window scrolls as one unit. The center panel already declares `overflow-y: auto` (`.intelligence-surface`, global.css:247) but it never activates because its parent has no bounded height. The left (`.control-rail`) and right (`.evidence-panel`) panels have no scroll rules at all.

This is a pure CSS/layout fix; no markup or JS changes are required.

## Goals / Non-Goals

**Goals:**
- Lock the shell to the viewport so the window never scrolls as a whole.
- Give the left, center, and right panels independent internal vertical scrolling.
- Keep `rail-bottom` pinned and the `modal-scrim` overlay covering the full viewport.

**Non-Goals:**
- No changes to `App.tsx` markup or component structure.
- No horizontal scrolling changes (existing `.command-block { overflow-x: auto }` stays).
- No redesign of column widths, spacing, or visual styling.
- No custom scrollbar styling (can be a follow-up).

## Decisions

**Decision 1: Fix the shell to `height: 100dvh` and clip its overflow.**
Change `.app-shell` from `min-height: 100vh` to a fixed height and add `overflow: hidden`. This bounds the grid to the viewport so children can own their scroll. Use `100dvh` (dynamic viewport height) rather than `100vh` so the layout stays correct on any window chrome; `100vh` is an acceptable fallback since this is a desktop Tauri window. Also set `html, #root { height: 100% }` and `body { height: 100vh; overflow: hidden }` (currently `min-height: 100vh` with no `height` on `html`/`#root`) so the shell has a concrete height to fill.
- *Alternative considered:* keep `min-height` and rely on `overflow: hidden` alone — rejected, because without a fixed height the grid still grows and `overflow-y: auto` on children never triggers.

**Decision 2: Add `overflow-y: auto` + `min-height: 0` to each panel.**
Grid items default to `min-height: auto`, which prevents them from shrinking below their content size — so `overflow-y: auto` would never kick in. Setting `min-height: 0` on each of the three panels (`.control-rail`, `.intelligence-surface`, `.evidence-panel`) lets them shrink to the grid track height and scroll internally. The center already has `overflow-y: auto`; add it to left and right.
- *Alternative considered:* wrap each panel's inner content in an extra scroll `<div>` — rejected as unnecessary; it would require `App.tsx` markup changes for no benefit.

**Decision 3: Left rail keeps `margin-top: auto` on `.rail-bottom`.**
The rail is `display: flex; flex-direction: column`. Adding `overflow-y: auto` does not break `margin-top: auto` on `.rail-bottom` — when content is short the bottom stays pinned; when content is tall the panel scrolls and the bottom section scrolls with it, which is the desired behavior.

## Risks / Trade-offs

- **[`100dvh` browser support]** → Tauri uses a modern WebView (WKWebView/WebView2), which supports `dvh`. `100vh` is a safe fallback if needed.
- **[Grid item won't scroll without `min-height: 0`]** → This is the classic flex/grid overflow trap; explicitly setting `min-height: 0` on all three panels mitigates it. Verify each panel scrolls after the change.
- **[Default scrollbars appear inside panels]** → Acceptable and expected; three independent scrollbars instead of one window scrollbar. Custom scrollbar styling is out of scope.
- **[`rail-bottom` scrolls away on very short windows]** → Acceptable; the updater section is reachable by scrolling the rail, which is strictly better than the current behavior where it disappears off the window.

## Migration Plan

Single-file CSS edit; ship with the app bundle. Rollback = revert the `global.css` diff. No data migration, no feature flag needed. Verify by running the app and scrolling each panel with the others populated.
