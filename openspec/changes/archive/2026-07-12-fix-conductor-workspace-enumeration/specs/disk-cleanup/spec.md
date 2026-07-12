## MODIFIED Requirements

### Requirement: Per-subitem cleanup for container targets

For container targets that the catalog iterates one by one — Docker prune components, old iOS runtimes, old nvm Node versions, Conductor workspaces, and Android SDK system-images — the system SHALL expose each subitem as an individually actionable row with its own size, metadata, and button. Subitem selection logic SHALL match the script: nvm keeps the current version and the latest LTS (only offered when more than 3 are installed); iOS runtimes keep the newest per platform; Conductor workspaces enumerate individual workspaces from a two-level directory structure and show branch and git status per workspace.

For Conductor workspaces specifically, the system SHALL distinguish between **project containers** (top-level directories with no `.git` of their own but whose children have `.git`) and **flat workspaces** (top-level directories with `.git` directly). Project containers SHALL be expanded one level so that each constituent workspace is shown as a separate subitem. Flat workspaces SHALL appear as single subitems under their own name. Each subitem label for a nested workspace SHALL include the project container name using a `project/workspace` format (e.g. `backend/paris`, `tub2/chengdu-v4`). Subitem IDs SHALL use a collision-safe separator (`project__workspace`) because two project containers can have workspaces with the same name.

#### Scenario: Container target expands to subitems

- **WHEN** a container target is displayed
- **THEN** each of its subitems SHALL appear as a row with its own action button

#### Scenario: nvm keeps current and latest LTS

- **WHEN** more than 3 nvm Node versions are installed
- **THEN** only versions other than the current and the latest LTS SHALL be offered for deletion

#### Scenario: iOS runtimes keep newest per platform

- **WHEN** old iOS runtimes are listed
- **THEN** the newest runtime per platform SHALL be kept and only older available runtimes offered for deletion

#### Scenario: Conductor project containers are expanded to individual workspaces

- **WHEN** a Conductor workspace subitem is listed
- **THEN** project containers (no own `.git`, children have `.git`) SHALL NOT appear as items themselves
- **AND** each individual workspace within a project container SHALL appear as a separate subitem labeled `project/workspace`

#### Scenario: Conductor flat workspaces appear by their own name

- **WHEN** a top-level Conductor directory has `.git` directly
- **THEN** it SHALL appear as a single subitem under its own name, without a project prefix

#### Scenario: Conductor workspace shows git state

- **WHEN** a Conductor workspace subitem is listed
- **THEN** its branch and whether it has uncommitted changes SHALL be shown
