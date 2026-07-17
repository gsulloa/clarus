# panel-scroll Specification

## Purpose
TBD - created by archiving change independent-panel-scroll. Update Purpose after archive.
## Requirements
### Requirement: Viewport-locked shell

The application shell SHALL occupy exactly the full viewport height and MUST NOT scroll as a single unit. The three-column grid (left, center, right) SHALL remain fixed to the viewport so that no panel content can push another panel off-screen.

#### Scenario: Tall content does not scroll the window

- **WHEN** any single panel contains more content than fits in the viewport
- **THEN** the application window itself does not scroll
- **AND** the other two panels remain fully visible in their original positions

#### Scenario: Shell matches viewport height

- **WHEN** the application window is resized
- **THEN** the shell height always equals the current viewport height
- **AND** no vertical scrollbar appears on the window as a whole

### Requirement: Independent per-panel scrolling

Each of the three panels — left (`control-rail`), center (`intelligence-surface`), and right (`evidence-panel`) — SHALL scroll independently within its own column. Scrolling one panel MUST NOT move the content of the other two.

#### Scenario: Scrolling the center panel

- **WHEN** the user scrolls within the center intelligence surface
- **THEN** only the center panel's content scrolls
- **AND** the left and right panels stay in place

#### Scenario: Scrolling the left panel

- **WHEN** the left control rail content exceeds its height and the user scrolls it
- **THEN** only the left panel scrolls
- **AND** the center and right panels stay in place

#### Scenario: Scrolling the right panel

- **WHEN** the right evidence panel content exceeds its height and the user scrolls it
- **THEN** only the right panel scrolls
- **AND** the left and center panels stay in place

### Requirement: Preserved pinned and overlay elements

The change SHALL preserve existing layout affordances: the bottom section of the left rail (`rail-bottom`) MUST remain pinned to the bottom of its column, and the confirm-modal overlay (`modal-scrim`) MUST continue to cover the full viewport.

#### Scenario: Rail bottom stays pinned

- **WHEN** the left rail content is shorter than the viewport
- **THEN** the `rail-bottom` section remains anchored to the bottom of the left panel

#### Scenario: Modal covers the viewport

- **WHEN** the confirm modal is open
- **THEN** the `modal-scrim` overlay covers the entire viewport regardless of any panel's scroll position
