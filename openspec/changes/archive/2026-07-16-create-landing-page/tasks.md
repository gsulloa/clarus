## 1. Scaffold static assets

- [x] 1.1 Add the design tokens from `DESIGN.md` (`:root` palette, radius, spacing) into `packages/infra/static/landing/styles.css`
- [x] 1.2 Set up marketing typography per `DESIGN.md` (display + body + JetBrains Mono) with `font-display: swap` and a system-stack fallback; self-host fonts under `static/landing/` (preferred) or preconnect a font CDN
- [x] 1.3 Add the Clarus mark as an inline SVG (or asset) reusing the existing mark from the current `index.html` / `packages/app/src/assets/clarus-mark.svg`

## 2. Page structure and content (static HTML)

- [x] 2.1 Build the hero: mark, `Clarus` wordmark, the one-line value proposition, and a primary download CTA region — all above the fold
- [x] 2.2 Write the safety-model section (scan → explain → review → act) with evidence-first copy, honoring `DESIGN.md` voice and anti-patterns
- [x] 2.3 Write the trust/evidence note (dry-run first, review before Trash, MIT/open-source, signed & notarized builds)
- [x] 2.4 Build the final download block containing the static "All platforms" region
- [x] 2.5 Add a minimal footer (GitHub link, version placeholder, license)
- [x] 2.6 Add a no-JS/fallback static download entry point (link to GitHub Releases and/or the release manifest) so the page works with JavaScript disabled

## 3. OS-aware download behavior (progressive enhancement)

- [x] 3.1 In `main.js`, fetch `https://releases.clarus.gulloa.click/download.json` on load
- [x] 3.2 Detect the visitor platform and map to a manifest key (`darwin-aarch64` | `darwin-x86_64` | `linux-x86_64` | `windows-x86_64`), defaulting macOS to Apple Silicon
- [x] 3.3 Render the matching installer as the primary CTA with platform label, resolved `version`, and human-readable file size (bytes → KB/MB)
- [x] 3.4 Populate the "All platforms" list from `installers`, each with label, link, and size; state that the primary button defaults to Apple Silicon on macOS
- [x] 3.5 Implement fallbacks: on fetch failure / non-200 / missing platform key, point the CTA at the GitHub Releases page and never render a dead-end button

## 4. Accessibility, motion, and responsiveness

- [x] 4.1 Add visible focus states to all interactive elements and verify logical tab order and heading hierarchy
- [x] 4.2 Verify WCAG AA contrast for text/`--text-secondary` pairs on the dark surfaces; ensure state is not conveyed by color alone
- [x] 4.3 Add subtle motion within the `DESIGN.md` budget (80ms hover, ≤240ms entrance) and gate it behind `@media (prefers-reduced-motion: reduce)`
- [x] 4.4 Verify layout reflows with no horizontal scroll at 375px, 768px, 1024px, and 1440px

## 5. Verify and deploy

- [x] 5.1 Confirm CORS: check whether `download.json` returns `Access-Control-Allow-Origin` permitting `https://clarus.gulloa.click`; if blocked, keep the GitHub fallback (and note a possible `ReleasesStack` CORS follow-up)
- [x] 5.2 Test locally: open `index.html`, exercise success + failed-fetch + JS-disabled paths, and check each breakpoint
- [x] 5.3 Replace the placeholder `packages/infra/static/landing/index.html` with the new page and confirm `pnpm infra:synth` still succeeds (asset path unchanged)
- [ ] 5.4 Deploy `ClarusLandingStack` and verify the live page: OS detection, version/size from the manifest, and fallback behavior
