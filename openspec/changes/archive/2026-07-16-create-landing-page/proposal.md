## Why

The public site at `clarus.gulloa.click` is a placeholder: a single centered hero and a raw link to `download.json`. Installers are now live and published per-platform by the release pipeline, but a visitor has no real way to download the app, no reason to trust it, and no explanation of what Clarus does or why it is safe. The landing page is the only public surface for the product and currently converts nobody.

## What Changes

- Replace the placeholder `packages/infra/static/landing/index.html` with a full product-led landing page for Clarus, following `DESIGN.md` (industrial minimalism, dark OLED, cyan accent, no marketing bloat).
- Add a **hero** with the Clarus mark, wordmark, one-sentence value proposition, and a primary download CTA above the fold.
- Add **OS-aware download**: at runtime, fetch `download.json` from `releases.clarus.gulloa.click`, detect the visitor's platform, and surface the matching installer as the primary CTA with the resolved version and file size; provide a secondary "All platforms" list for the other builds.
- Add a small number of **product sections** that explain the safety model (scan → explain → review → act), each grounded in evidence rather than cleanup theater, per the design voice.
- Add graceful **fallback states** when `download.json` is unreachable or a platform build is missing (link to GitHub Releases; never show a broken button).
- Keep the page a **static, dependency-free** HTML/CSS asset (optionally one small vanilla-JS file) so it continues to deploy unchanged through `LandingStack`'s `Source.asset(static/landing)` bucket deployment with no build step.
- Ensure **accessibility and responsiveness**: WCAG AA contrast on dark, visible focus, `prefers-reduced-motion` support, and layout that holds from 375px to desktop.

## Capabilities

### New Capabilities
- `landing-page`: The public marketing/download page for Clarus — its content structure, OS-aware download behavior sourced from the release `download.json` manifest, fallback handling, and design/accessibility requirements.

### Modified Capabilities
<!-- None. LandingStack already deploys the static/landing directory unchanged; no infra requirement changes. -->

## Impact

- **Code:** `packages/infra/static/landing/` (rewritten `index.html`; possibly `main.js`, `styles.css`, and an SVG mark asset).
- **Deployment:** No change to `packages/infra/lib/LandingStack/index.ts` — it already deploys the `static/landing` asset and invalidates `/*`. The page stays build-free.
- **Runtime dependency:** The page reads `https://releases.clarus.gulloa.click/download.json` (schema: `{ version, pub_date, installers: { <platform>: { url, filename, size } } }`, served with `no-cache`). This is a soft dependency — the page must render and remain useful if the fetch fails.
- **No changes** to the Tauri app, the release workflow, or the `download.json` schema.
