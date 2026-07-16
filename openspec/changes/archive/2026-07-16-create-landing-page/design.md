## Context

Clarus ships as signed Tauri bundles. Tag pushes build per-platform installers and publish two manifests to `releases.clarus.gulloa.click`: `latest.json` (for the in-app Tauri updater) and `download.json` (intended for the landing page). `download.json` has the shape:

```json
{
  "version": "0.5.0",
  "pub_date": "2026-07-12T...Z",
  "installers": {
    "darwin-aarch64": { "url": "https://releases.clarus.gulloa.click/<file>", "filename": "<file>", "size": 12345 },
    "darwin-x86_64":  { ... },
    "linux-x86_64":   { ... },
    "windows-x86_64": { ... }
  }
}
```

It is served with `Cache-Control: no-cache, max-age=0` from a CloudFront distribution separate from the landing site.

The landing site is a plain static asset: `LandingStack` deploys `packages/infra/static/landing/` to an S3 bucket via `BucketDeployment(Source.asset(...))` and invalidates `/*` on the CloudFront distribution fronting `clarus.gulloa.click` (+ `www`). There is **no build step** in that pipeline — whatever files sit in `static/landing/` are what ship. The current content is a single `index.html` with an inline `<style>` block and a hardcoded link to `download.json`.

`DESIGN.md` is the authoritative design system and already contains a "Landing Page Direction" section: spare, product-led, dark, cyan (`#2EE8D6`) as the only accent, industrial-minimalism, explicit anti-patterns (no generic three-feature grids, no abstract hero SVGs, no stock laptops, no cleanup claims the product does not support). The `ui-ux-pro-max` skill independently validated the overall shape — Dark Mode (OLED), a "Minimal + Documentation" pattern (Hero → Features → CTA with the CTA above the fold), platform-specific download CTAs, WCAG-level contrast, reduced-motion, and responsive breakpoints — but its generic palette/type suggestions (Inter, green accent) are overridden by `DESIGN.md`.

## Goals / Non-Goals

**Goals:**
- A production-quality public landing page that explains Clarus and drives a download, faithful to `DESIGN.md`.
- Real, OS-aware downloads driven live by `download.json`, resilient to fetch failure and missing platform builds.
- Zero-build, dependency-free static deployment that ships through the existing `LandingStack` unchanged.
- Accessible (WCAG AA on dark, keyboard focus, reduced motion) and responsive from 375px to desktop.

**Non-Goals:**
- No changes to `LandingStack`, the release workflow, or the `download.json` schema.
- No framework, bundler, npm dependency, or CSS/JS toolchain for the landing page.
- No analytics, cookies, email capture, blog, docs site, or pricing (Clarus is free/MIT).
- No claims about automatic cleanup or any capability the app does not yet ship.
- No light-mode variant (the brand is dark-only, consistent with the app).

## Decisions

### D1: Keep it a hand-authored static site, no build step
Author `index.html` + `styles.css` + `main.js` (+ an SVG mark) as plain files in `static/landing/`. **Why:** `LandingStack` deploys the directory verbatim with no build; introducing a bundler would mean changing the infra pipeline and CI, contradicting a non-goal. Vanilla HTML/CSS/JS is more than sufficient for a handful of sections. *Alternative considered:* a Vite/Astro build emitting to `static/landing` — rejected as disproportionate and pipeline-invasive.
- Sub-decision: prefer a separate `styles.css`/`main.js` over a single inline-everything file for readability, but this is cosmetic; a single self-contained `index.html` is acceptable if simpler to ship. Fonts: self-host or system-stack fallback rather than blocking on a font CDN (see D5).

### D2: Client-side OS-aware download via `fetch("download.json")`
On load, `fetch("https://releases.clarus.gulloa.click/download.json")`, then map `navigator.userAgent`/`navigator.platform` to a platform key (`darwin-aarch64` | `darwin-x86_64` | `linux-x86_64` | `windows-x86_64`) and render the matching installer as the primary CTA, labeled with the OS and showing the resolved version and human-readable size. **Why:** the manifest is the single source of truth for URLs/versions and is served `no-cache`, so the page never hardcodes a version or file path and stays correct across releases with no redeploy. *Alternatives considered:* (a) hardcode links — rejected, goes stale every release; (b) build-time injection — rejected, there is no build step (D1); (c) a server/redirect endpoint — rejected, over-engineered for a static site.
- Apple-silicon vs Intel Mac cannot be distinguished reliably from the browser. **Decision:** default the macOS primary CTA to `darwin-aarch64` (Apple Silicon), and always expose both Mac builds in the "All platforms" list so an Intel user can pick correctly. State the assumption in visible copy.

### D3: Progressive enhancement — the page is useful with no JS and with a failed fetch
Render all content (hero, sections, a static "Download / All platforms" region) in HTML. JS only *upgrades* the download region with the detected-OS primary button and live version/size. **Why:** resilience is an explicit requirement; a broken manifest or blocked script must never yield a dead page. **Fallbacks:** if `fetch` fails or the platform key is absent, show a CTA linking to the GitHub Releases page (`https://github.com/gsulloa/clarus/releases/latest`) plus the full "All platforms" list from the manifest when available. Never render a button with no working target.

### D4: Content structure grounded in the safety model, not a generic feature grid
Sections: (1) Hero — mark, `Clarus` wordmark, the one-line value proposition from `DESIGN.md`, primary download CTA above the fold; (2) a concise explanation of the scan → explain → review → act safety ladder as the product's core differentiator; (3) an evidence/trust note (dry-run first, review before any Trash move, open-source/MIT, signed & notarized builds); (4) a final download/CTA block with the "All platforms" list; (5) a minimal footer (GitHub, version, license). **Why:** `DESIGN.md` explicitly forbids generic three-feature grids and cleanup theater; the differentiator is the evidence-first safety model, so the page leads with that. Keep total length short and product-led.

### D5: Typography and palette come from `DESIGN.md`, not the skill defaults
Use the exact `:root` tokens from `DESIGN.md` (`--bg-base #081014`, `--accent #2EE8D6`, etc.). Marketing typography per `DESIGN.md`: a distinctive display face (Satoshi/General Sans) for the wordmark/hero and a readable body face (Geist/Source Sans 3), with JetBrains Mono for version/filename/size. **Why:** brand consistency with the app and the documented decision log; `DESIGN.md` explicitly bans Inter/Roboto/etc. as the primary voice. **Loading:** `font-display: swap`, preconnect/preload only the display face; fall back to the system UI stack so text is never invisible and the page renders even if the font host is blocked (supports D3). If self-hosting is simpler than a CDN, self-host.

### D6: Accessibility & motion as first-class requirements
WCAG AA contrast on the dark surfaces (verify text/`--text-secondary` pairs), visible focus rings on the mark link and all CTAs, semantic landmarks/heading order, `alt`/`aria` on the mark, and `@media (prefers-reduced-motion: reduce)` disabling the subtle hero glow/entrance transitions. Motion budget per `DESIGN.md`: 80ms hover, ≤240ms entrance, `cubic-bezier(0.2,0,0,1)`. **Why:** required by the spec and by the design system; the app already commits to these.

## Risks / Trade-offs

- **Cross-origin fetch to the releases distribution** → The landing site (`clarus.gulloa.click`) and manifest (`releases.clarus.gulloa.click`) are different origins. Verify CORS allows a browser `fetch` from the landing origin; if not, either add a permissive `Access-Control-Allow-Origin` response header on the `download.json` behavior (small `ReleasesStack` tweak — would then touch infra) or fall back to the GitHub Releases link. **Mitigation:** design D3's fallback so the page is fully functional even if the fetch is blocked; confirm CORS during implementation and only touch infra if needed.
- **Mac architecture ambiguity** → Browsers cannot reliably tell Apple Silicon from Intel. **Mitigation:** default to Apple Silicon, always show both Mac builds, and label the assumption (D2).
- **Manifest schema drift** → If `build-download-manifest.mjs` ever changes keys, the page breaks silently. **Mitigation:** code defensively (guard on missing keys, fall through to fallback), and treat the four platform keys as the contract documented here and in the spec.
- **Stale CloudFront cache for the page itself** → The landing distribution uses `CACHING_OPTIMIZED`; `LandingStack` already invalidates `/*` on deploy, so content updates propagate. The `download.json` fetch is `no-cache`, so version/size are always fresh regardless of page caching. **Mitigation:** none needed beyond relying on the existing invalidation.
- **No-build authoring discipline** → Hand-written CSS can drift from `DESIGN.md`. **Mitigation:** centralize the documented `:root` tokens verbatim and reference them everywhere.

## Migration Plan

1. Replace/expand files under `packages/infra/static/landing/`.
2. Verify locally by opening `index.html` (and pointing the fetch at the live `download.json`, or a local fixture) across widths 375/768/1024/1440 and with JS disabled.
3. Confirm CORS for the cross-origin `download.json` fetch; if blocked, either enable the fallback path only or add the CORS header in `ReleasesStack` (separate, optional follow-up).
4. Deploy via the normal `ClarusLandingStack` deploy; the `BucketDeployment` uploads the new asset and invalidates `/*`.
5. **Rollback:** revert the `static/landing` files and redeploy — no state, no data, no schema to migrate.

## Open Questions

- Does the `download.json` CloudFront behavior currently return an `Access-Control-Allow-Origin` header permitting `https://clarus.gulloa.click`? (Determines whether a small `ReleasesStack` change is needed or the GitHub fallback suffices.) — resolve during implementation.
- Self-host the display/mono fonts in `static/landing` vs. load from a font CDN? (Self-hosting is more resilient and privacy-friendly; CDN is less setup.) — implementer's call, default to self-host.
- Confirm the exact GitHub Releases URL to use for the JS-off / fetch-failed fallback (`.../releases/latest` vs `/releases`).
