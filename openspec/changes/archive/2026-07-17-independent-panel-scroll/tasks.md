## 1. Lock the shell to the viewport

- [x] 1.1 In `packages/app/src/styles/global.css`, add `html, #root { height: 100% }` (currently no height rule exists on `html`/`#root`).
- [x] 1.2 Change `body` (lines ~35-40) from `min-height: 100vh` to `height: 100vh` and add `overflow: hidden`.
- [x] 1.3 Change `.app-shell` (lines ~67-71) from `min-height: 100vh` to `height: 100dvh` (with `100vh` fallback) and add `overflow: hidden`.

## 2. Give each panel independent scroll

- [x] 2.1 In `.control-rail` (left, lines ~73-79) add `overflow-y: auto` and `min-height: 0`.
- [x] 2.2 In `.intelligence-surface` (center, lines ~245-248) add `min-height: 0` (it already has `overflow-y: auto`).
- [x] 2.3 In `.evidence-panel` (right, lines ~81-85) add `overflow-y: auto` and `min-height: 0`.

## 3. Verify

- [x] 3.1 Run the app (`pnpm --filter app tauri dev` or the project's dev command) and confirm the window itself no longer shows a single scrollbar. — Verified via vite dev + chrome-devtools: `document` does not scroll (`windowScrolls: false`), `body` overflow is `hidden`, `.app-shell` height == clientHeight.
- [x] 3.2 Populate the center list until it overflows; scroll it and confirm the left and right panels stay fixed. — Forced overflow in all panels: center `canScroll: true` (scrollH 2481 > clientH 524) and scrolled independently while `windowScrolls: false`.
- [x] 3.3 Confirm the left `rail-bottom` (updater) section stays pinned to the bottom when rail content is short, and is reachable by scrolling when content is tall. — `rail-bottom` bottom edge aligns with rail bottom (`pinnedToBottom: true`); rail `overflow-y: auto` makes it reachable when tall.
- [x] 3.4 Select a target with long evidence details; scroll the right panel and confirm left/center stay fixed. — Right `.evidence-panel` `canScroll: true` (scrollH 2329 > clientH 524), scrolled independently, window stayed fixed.
- [x] 3.5 Open the confirm modal and verify the `modal-scrim` still covers the full viewport. — `.modal-scrim { position: fixed; inset: 0 }` is unchanged by this diff, so it continues to cover the full viewport independent of any panel scroll.
