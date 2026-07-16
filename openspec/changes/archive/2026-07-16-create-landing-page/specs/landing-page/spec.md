## ADDED Requirements

### Requirement: Product hero above the fold
The landing page SHALL present, within the first viewport, the Clarus mark, the `Clarus` wordmark, the one-sentence value proposition, and a primary download call-to-action.

#### Scenario: First viewport content
- **WHEN** a visitor loads `https://clarus.gulloa.click` on a 1440px-wide desktop viewport
- **THEN** the Clarus mark, the `Clarus` wordmark, the sentence "Precise local disk intelligence for reviewing optimization candidates before any file operation.", and a primary download button are all visible without scrolling

#### Scenario: Hero on small screens
- **WHEN** a visitor loads the page at 375px width
- **THEN** the hero content reflows without horizontal scrolling and the primary download CTA remains visible and tappable

### Requirement: OS-aware primary download
The landing page SHALL detect the visitor's operating system and present the matching installer from `download.json` as the primary download CTA, labeled with the platform and showing the resolved version and human-readable file size.

#### Scenario: macOS visitor
- **WHEN** a visitor on macOS loads the page and `download.json` is fetched successfully
- **THEN** the primary CTA links to a macOS installer URL from the manifest, defaults to the Apple Silicon (`darwin-aarch64`) build, and displays the manifest `version` and a human-readable size (e.g. "12.3 MB")

#### Scenario: Windows visitor
- **WHEN** a visitor on Windows loads the page and the manifest contains `windows-x86_64`
- **THEN** the primary CTA links to the `windows-x86_64` installer URL and is labeled for Windows

#### Scenario: Linux visitor
- **WHEN** a visitor on Linux loads the page and the manifest contains `linux-x86_64`
- **THEN** the primary CTA links to the `linux-x86_64` installer URL and is labeled for Linux

#### Scenario: Version and size come from the manifest
- **WHEN** a new release publishes an updated `download.json`
- **THEN** the page reflects the new version, URLs, and sizes on next load with no code change or redeploy of the landing site

### Requirement: All-platforms download list
The landing page SHALL provide a list of every installer present in `download.json`, so a visitor can choose a build other than the auto-detected one.

#### Scenario: Intel Mac user overrides the default
- **WHEN** a macOS visitor whose machine is Intel views the download options
- **THEN** both `darwin-aarch64` and `darwin-x86_64` builds are listed and selectable, and the page states that the primary button defaults to Apple Silicon

#### Scenario: All available builds listed
- **WHEN** `download.json` is fetched successfully
- **THEN** the page lists a labeled download link for each platform key present in `installers`, each showing its file size

### Requirement: Graceful fallback when the manifest is unavailable
The landing page SHALL remain functional and never present a broken or dead-end download control when `download.json` cannot be fetched or a platform build is missing.

#### Scenario: Manifest fetch fails
- **WHEN** the request for `download.json` fails (network error, CORS block, or non-200 response)
- **THEN** the download CTA links to the Clarus GitHub Releases page instead, and no button points to a missing or empty target

#### Scenario: Detected platform has no build
- **WHEN** the manifest loads but contains no installer for the visitor's detected platform
- **THEN** the page shows the available builds and a link to GitHub Releases rather than a disabled or broken primary button

#### Scenario: JavaScript disabled
- **WHEN** the page is loaded with JavaScript disabled
- **THEN** the hero, product content, and a working download entry point (link to the release manifest or GitHub Releases) are still rendered from static HTML

### Requirement: Evidence-first product content
The landing page SHALL explain Clarus through its evidence-first safety model and SHALL NOT use generic cleanup-tool framing or claim capabilities the app does not ship.

#### Scenario: Safety model is communicated
- **WHEN** a visitor scrolls past the hero
- **THEN** the page explains the scan → explain → review → act flow, conveying that Clarus analyzes and explains candidates and requires review before any file is moved

#### Scenario: No unsupported claims or banned framing
- **WHEN** the page copy is reviewed against `DESIGN.md`
- **THEN** it contains no "clean now / fix everything / delete junk / optimize instantly" style claims, no generic three-feature grid, no stock laptop imagery, and no promise of automatic cleanup

### Requirement: Adheres to the Clarus design system
The landing page SHALL follow the visual system defined in `DESIGN.md`: dark background, cyan (`#2EE8D6`) as the only primary accent, the documented `:root` color and radius tokens, and the marketing typography direction.

#### Scenario: Design tokens applied
- **WHEN** the page is inspected
- **THEN** it uses the `DESIGN.md` core palette (e.g. `--bg-base #081014`, `--text-primary #E7EEF2`, `--accent #2EE8D6`) and does not introduce Inter/Roboto/Montserrat/Poppins as the primary brand voice or a non-cyan primary accent

### Requirement: Accessible and responsive
The landing page SHALL meet WCAG AA contrast on its dark surfaces, provide keyboard focus states, respect reduced-motion preferences, and render without horizontal scroll across mobile and desktop widths.

#### Scenario: Reduced motion respected
- **WHEN** a visitor has `prefers-reduced-motion: reduce` set
- **THEN** non-essential entrance animations and the hero glow are disabled or minimized

#### Scenario: Keyboard focus visible
- **WHEN** a keyboard user tabs through the page
- **THEN** every interactive element (mark link, primary CTA, per-platform links, footer links) shows a visible focus indicator in a logical order

#### Scenario: Responsive across breakpoints
- **WHEN** the page is viewed at 375px, 768px, 1024px, and 1440px
- **THEN** content reflows appropriately with no horizontal scrolling at any of those widths

### Requirement: Zero-build static deployment
The landing page SHALL ship as static assets in `packages/infra/static/landing/` deployable by the existing `LandingStack` with no build step or added runtime dependency.

#### Scenario: Deploys through existing pipeline unchanged
- **WHEN** `ClarusLandingStack` is deployed
- **THEN** the assets in `packages/infra/static/landing/` are uploaded and served as-is, with `index.html` as the root object, without any bundler or compile step

#### Scenario: No added dependencies
- **WHEN** the landing page assets are reviewed
- **THEN** they consist only of static HTML, CSS, JavaScript, and image/font files, with no npm package or third-party runtime script required to render the page
