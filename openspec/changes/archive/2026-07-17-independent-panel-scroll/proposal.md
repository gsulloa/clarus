## Why

The Clarus main window is a three-column layout (left `control-rail`, center `intelligence-surface`, right `evidence-panel`), but the whole page scrolls as a single unit because `.app-shell` uses `min-height: 100vh` with no bounded height. When any column's content is tall, the entire window scrolls and the user loses sight of the other panels (e.g. the scan button, disk readout, or selected-target details scroll off-screen). Each panel should scroll independently within a viewport-locked shell.

## What Changes

- Lock the app shell to the viewport height (fixed `100vh`) and clip its overflow so the window itself no longer scrolls as one unit.
- Give each of the three panels its own independent vertical scroll: `control-rail` (left), `intelligence-surface` (center), and `evidence-panel` (right).
- Ensure grid children can shrink and scroll (`min-height: 0`) so each panel's `overflow-y: auto` actually activates.
- Preserve the pinned bottom section of the left rail (`rail-bottom`, `margin-top: auto`) and the confirm-modal overlay behavior.

## Capabilities

### New Capabilities
- `panel-scroll`: Viewport-locked three-column shell where the left, center, and right panels each scroll independently instead of the whole window scrolling together.

### Modified Capabilities
<!-- No existing spec covers the shell layout; nothing to modify. -->

## Impact

- `packages/app/src/styles/global.css` — `.app-shell` (fixed height + overflow clip), `.control-rail` and `.evidence-panel` (add `overflow-y: auto` + `min-height: 0`), `.intelligence-surface` (add `min-height: 0`), and `body`/`html`/`#root` height rules.
- No JavaScript/TSX changes expected; `packages/app/src/App.tsx` markup stays as-is.
- Purely CSS/layout change — no API, dependency, or data impact.
