# Design System - Clarus

## Product Context

- **What this is:** Clarus is a Tauri desktop app for understanding local disk usage before taking action. It scans, classifies, explains, and lets the user review optimization candidates before any destructive operation exists.
- **Who it is for:** Mac-first technical and semi-technical users who know their disk is messy but do not trust one-click cleaners. The user wants evidence and control, not theater.
- **Category:** Desktop utility, disk analyzer, cleanup advisor, local productivity tool.
- **Project type:** Desktop application with a small public landing page and release/update infrastructure.
- **Core promise:** Clarus provides a clear, surgical view of disk state so the user can recover space without losing confidence.

## Positioning

Clarus is not a cleaner with a bigger button. It is a disk intelligence tool.

Most cleanup tools frame the user as late, careless, or in danger. Clarus should feel like an expert assistant in a controlled room: precise, calm, and transparent about uncertainty.

The user should feel:

- **Informed:** Every recommendation has a visible reason.
- **Safe:** No file is moved without review and an explicit action.
- **In control:** Actions are modular, reversible where possible, and scoped.
- **Relieved:** The app turns an unbounded mess into a structured map.

## Aesthetic Direction

- **Direction:** Industrial minimalism with surgical precision.
- **Decoration level:** Intentional, not expressive. Use subtle light, fine borders, controlled contrast, and measured density. Avoid visual noise.
- **Mood:** Dark, focused, exact. The UI should feel like an instrument panel for a sensitive operation, not a consumer cleaner.
- **Design thesis:** Clarus finds the signal inside disk chaos, so the interface should make hierarchy, causality, and risk visible.

## Visual Identity

### Logo

The mark represents a focal line completing an incomplete disk shape. It should communicate clarity, completion, and potential released.

Rules:

- Use a simple geometric mark with an incomplete circle or square and one precise diagonal or axial line.
- Keep the mark readable at 16px.
- Do not use trash cans, brooms, warning triangles, cartoon mascots, storage boxes, or crossed-out files.
- The mark can glow subtly in marketing, but inside the app it should render flat and controlled.

### Iconography

- Use lucide icons for app controls.
- Icons are 16px to 18px in dense controls, 24px to 28px in empty states.
- Stroke should feel consistent with lucide default, no filled decorative icon sets.
- Icons must clarify action, not decorate text.

## Typography

### App Typography

The desktop app should use the platform UI stack by default:

```css
font-family:
  ui-sans-serif,
  system-ui,
  -apple-system,
  BlinkMacSystemFont,
  "Segoe UI",
  sans-serif;
```

Rationale: Clarus is a native-feeling desktop utility. On macOS, San Francisco is the right base because it reduces friction and makes the tool feel installed, not embedded.

### Optional Marketing Typography

For the public landing page, the brand can be more distinctive:

- **Display:** Satoshi or General Sans for the Clarus wordmark and large claims.
- **Body:** Geist or Source Sans 3 for readable product copy.
- **Data/code accents:** JetBrains Mono for hashes, file paths, release versions, and manifest examples.

Do not use Inter, Roboto, Montserrat, Poppins, or generic tech-marketing display fonts as the brand's main voice.

### Type Scale

Use a compact desktop scale. No viewport-based font scaling inside the app.

| Token | Size | Weight | Use |
| --- | ---: | ---: | --- |
| `display-xl` | 56px | 650 | Landing hero only |
| `title-lg` | 30px | 650 | Main app surface title |
| `title-md` | 22px | 650 | Evidence panel headings |
| `title-sm` | 20px | 650 | Sidebar product subtitle |
| `body-md` | 14px | 400 | Default UI copy |
| `body-sm` | 13px | 400 | Muted descriptions, paths |
| `label` | 11px | 700 | Eyebrows and section labels |
| `metric` | 24px | 720 | Scan totals |

### Numeric Data

Use tabular numerals for file sizes, counts, percentages, and scan metrics:

```css
font-variant-numeric: tabular-nums;
```

## Color System

### Approach

Restrained. One primary accent, quiet neutrals, semantic colors only when they carry meaning.

Color should answer: "Is this safe, selected, risky, blocked, or complete?"

### Core Palette

| Token | Hex | Use |
| --- | --- | --- |
| `--bg-base` | `#081014` | App background |
| `--bg-deep` | `#02080B` | Tables, deep wells |
| `--surface` | `rgba(231, 238, 242, 0.055)` | Cards, metrics |
| `--surface-strong` | `rgba(231, 238, 242, 0.07)` | Secondary buttons |
| `--border` | `rgba(231, 238, 242, 0.11)` | Default borders |
| `--border-strong` | `rgba(231, 238, 242, 0.14)` | Interactive borders |
| `--text-primary` | `#E7EEF2` | Primary text |
| `--text-secondary` | `#8EA0A8` | Muted body |
| `--text-tertiary` | `#7F929A` | Labels and eyebrows |
| `--accent` | `#2EE8D6` | Safe action, scan complete, selected state |
| `--accent-soft` | `rgba(46, 232, 214, 0.09)` | Accent backgrounds |
| `--accent-text` | `#AEFBF3` | Accent text on dark |
| `--ink-on-accent` | `#061012` | Text on bright cyan |

### Semantic Colors

| Token | Hex | Use |
| --- | --- | --- |
| `--success` | `#2EE8D6` | Safe completion, validated state |
| `--warning` | `#F5C451` | Needs review, partial confidence |
| `--danger` | `#FFB4A8` | Error, destructive disabled, risky candidate |
| `--info` | `#8EBBFF` | Neutral system information |

### Usage Rules

- Cyan means "safe, complete, selected, or ready." Do not use it as decoration everywhere.
- Red/coral must be rare. It should indicate an error or a destructive path.
- Never show "danger" for candidates just because they are large. Large files are review-worthy, not bad.
- Avoid purple gradients, beige dashboards, blue corporate SaaS gradients, and green-only cleanup palettes.

## Spacing

- **Base unit:** 4px.
- **Density:** Compact, with enough air to prevent anxiety.
- **Default panel padding:** 24px.
- **Main surface padding:** 30px.
- **Grid gaps:** 10px to 14px for dense UI, 24px for section-level separation.

| Token | Value | Use |
| --- | ---: | --- |
| `space-2xs` | 2px | Fine optical adjustment |
| `space-xs` | 4px | Tight label spacing |
| `space-sm` | 8px | Inline icon gaps |
| `space-md` | 12px | Control groups |
| `space-lg` | 16px | Card padding |
| `space-xl` | 24px | Panels |
| `space-2xl` | 30px | Main surface |
| `space-3xl` | 38px | Major sidebar break |

## Layout

### App Shell

The main app uses a three-zone instrument layout:

```css
grid-template-columns: 280px minmax(480px, 1fr) 320px;
```

- **Left rail:** Scope, scan controls, release/update status, future exclusions.
- **Center surface:** Scan state, totals, categories, candidates.
- **Right evidence panel:** Reasoning, confidence, recovery estimate, scoped actions.

This structure should hold as the app grows. New features should fit into one of these zones instead of adding modal-heavy flows.

### Breakpoints

- Desktop app minimum width: 1024px.
- Under 1100px, preserve the rail widths and let the center surface compress first.
- Future mobile web landing page can be responsive, but the Tauri app is not a phone interface.

### Tables and Lists

Candidate rows should be stable and scannable:

- Path column gets flexible width and truncates with ellipsis.
- Type, size, and confidence columns use fixed widths.
- Row hover and selected state use accent-soft, not heavy fills.
- Long paths must never blow out the layout.

### Border Radius

Keep geometry controlled:

| Token | Value | Use |
| --- | ---: | --- |
| `radius-sm` | 4px | Small tags, tight controls |
| `radius-md` | 8px | Cards, panels, buttons |
| `radius-pill` | 999px | Status pills only |

Do not use large rounded cards. Clarus is precise, not pillowy.

## Component Rules

### Buttons

Use modular actions. Avoid one giant cleanup CTA.

- **Primary:** Folder selection or explicit safe continuation.
- **Scan action:** High emphasis, but wording must describe analysis, not deletion.
- **Quiet action:** Update checks, exclusions, secondary controls.
- **Danger action:** Disabled until review/undo exists. When enabled, it must include scope and confirmation.

Button copy examples:

- `Select folder`
- `Start deep scan`
- `Review candidates`
- `Exclude folder`
- `Move selected to Trash`

Never use:

- `Clean now`
- `Fix everything`
- `Delete junk`
- `Optimize instantly`

### Metrics

Metrics should describe the scan, not shame the user.

Good:

- `Scanned`
- `Candidates`
- `Files`
- `Folders`
- `Estimated recovery`

Avoid:

- `Wasted`
- `Junk`
- `Danger`
- `Problems`

### Evidence Cards

Every candidate needs an explanation with:

- Candidate type.
- File or folder path.
- Size or estimated recovery.
- Reason Clarus marked it.
- Confidence level.
- Any caveat that affects safety.

Evidence is the product. Do not hide it behind tooltips.

### Empty States

Empty states should be quiet and useful:

- Say what will appear here.
- Do not add marketing claims.
- Do not use illustrations unless they explain scanning or structure.

Example:

`Clarus will list candidates here with the reason for each recommendation.`

### Update Status

Autoupdate should feel like maintenance, not interruption.

- Show `Checking signed release manifest...` during check.
- Show the version when available.
- Do not use blocking modals for routine update checks.
- If an update fails, explain that the release channel is unreachable, not that the app is broken.

## Motion

- **Approach:** Minimal-functional.
- **Durations:** 80ms for hover/focus, 160ms for row selection, 240ms for panel entrance.
- **Easing:** `cubic-bezier(0.2, 0, 0, 1)` for enter and selection.

Motion should clarify state changes:

- Scan starting.
- Scan completed.
- Candidate row selected.
- Evidence panel refreshed.

Do not animate file rows while a scan is running in a way that makes the user think files are being changed.

## Product Voice

Clarus speaks like a calm expert. Precise, objective, never alarmist.

### Voice Principles

- Use evidence before recommendation.
- Describe uncertainty plainly.
- Respect that deletion is high risk.
- Say what the app did, not what the user did wrong.

### Preferred Copy

- `Deep scan started. Analyzing file structure...`
- `Clarus identified 12.4 GB as optimization candidates.`
- `Review candidates before moving anything to Trash.`
- `This folder appears to contain cache files that are commonly regenerated by the owning app.`
- `This file is large enough to deserve manual review before archiving or deleting.`

### Avoid

- `You have 100 GB of junk.`
- `Your disk is in danger.`
- `Clean now.`
- `Fix all issues.`
- `Delete safely` unless undo and recovery are actually implemented.

## Interaction Model

### Safety Ladder

Clarus should move users through increasing commitment:

1. **Select scope:** User chooses a folder or disk area.
2. **Scan:** Clarus reads and classifies only.
3. **Explain:** Candidates appear with reasons and confidence.
4. **Review:** User inspects candidates and excludes paths.
5. **Stage:** Future workflow groups selected items before action.
6. **Act:** Move to Trash, never permanent delete in MVP.
7. **Recover:** Future workflow supports restore or at least clear Trash instructions.

No step should skip over review.

### Candidate Confidence

Confidence is not a score for how bad a file is. It is how confident Clarus is about the recommendation.

- **High:** Cache, temp files, known regenerated data.
- **Medium:** Logs, empty folders, stale build outputs.
- **Low:** Large files, unknown archives, media, downloads.

Low-confidence candidates should default to review, not selection.

### Error Handling

Errors should be concrete:

- Say what failed.
- Say whether files were modified.
- Suggest the next safe step.

Example:

`Clarus could not read this folder. No files were modified. Choose a folder you have permission to inspect.`

## Accessibility

- Text contrast must meet WCAG AA on dark surfaces.
- Do not communicate state by color alone. Pair color with icon, label, or position.
- All controls need visible focus states.
- Buttons must preserve text without overflow at minimum app width.
- File paths should be copyable in future detail views.
- Respect reduced motion. Disable non-essential transitions when `prefers-reduced-motion: reduce`.

## Landing Page Direction

The landing page should be spare and product-led.

First viewport:

- Clarus mark.
- `Clarus` as the headline.
- One sentence: `Precise local disk intelligence for reviewing optimization candidates before any file operation.`
- Download or release manifest CTA once installers are live.

Avoid:

- Generic three-feature grids.
- Big abstract SVG hero scenes.
- Stock photos of laptops.
- Claims about automatic cleanup until the product supports it.

## Implementation Tokens

Future CSS should centralize these variables:

```css
:root {
  --bg-base: #081014;
  --bg-deep: #02080b;
  --text-primary: #e7eef2;
  --text-secondary: #8ea0a8;
  --text-tertiary: #7f929a;
  --accent: #2ee8d6;
  --accent-text: #aefbf3;
  --ink-on-accent: #061012;
  --danger: #ffb4a8;
  --warning: #f5c451;
  --info: #8ebbff;
  --border: rgba(231, 238, 242, 0.11);
  --border-strong: rgba(231, 238, 242, 0.14);
  --surface: rgba(231, 238, 242, 0.055);
  --surface-strong: rgba(231, 238, 242, 0.07);
  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-pill: 999px;
}
```

## Decisions Log

| Date | Decision | Rationale |
| --- | --- | --- |
| 2026-07-07 | Use industrial minimalism with surgical precision | The app handles risky file decisions, so control and evidence matter more than delight. |
| 2026-07-07 | Keep the app dark by default | Disk inspection is a focused desktop task, and the current UI already uses a dark instrument-panel shell. |
| 2026-07-07 | Use cyan as the only primary accent | Cyan reads as clarity and completion without the alarm semantics of red or the generic feel of blue SaaS. |
| 2026-07-07 | Keep destructive actions gated | MVP trust depends on dry-run analysis and review before Trash movement. |
| 2026-07-07 | Prefer native app typography in the desktop shell | The app should feel installed and exact on macOS rather than like a marketing page embedded in a webview. |
