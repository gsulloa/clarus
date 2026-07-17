# project-artifact-cleanup Specification

## Purpose
Tier 1 catalog target that discovers git repositories under conventional developer-root directories and cleans only regenerable artifact directories inside each project, without deleting source files or git history.

## Requirements

### Requirement: Tier 1 target cleans regenerable artifacts in discovered project repositories

The system SHALL expose a Tier 1 catalog target named "Project build artifacts" that discovers git repositories under conventional developer-root directories and offers per-repository cleanup of only regenerable artifact directories. The system SHALL determine candidate roots by checking a fixed set of conventional directory names relative to the user's home directory (`dev`, `Developer`, `Projects`, `src`, `code`, `repos`, `work`, `git`, `workspace`) and SHALL only consider those that exist; it SHALL NOT hardcode any single personal path. Within each existing root, the system SHALL enumerate directories containing a `.git` entry up to a bounded depth and treat each as one project. For each project, the system SHALL clean the same artifact directory names as the Conductor artifact target (`node_modules`, `.next`, `dist`, `cdk.out`, `.turbo`, `target`, `__pycache__`, `.venv`, `venv`, `build`, `.cache`, `.parcel-cache`) wherever they appear inside the project up to a bounded depth. The cleanup SHALL NOT delete source files, `.git` directories, or any directory not in this list. No confirmation is required beyond a single click.

#### Scenario: Roots are discovered by convention, not hardcoded

- **WHEN** the target is scanned on a machine where `~/dev` and `~/Projects` exist but `~/src` does not
- **THEN** repositories under `~/dev` and `~/Projects` SHALL be listed
- **AND** the absent `~/src` root SHALL contribute no items and cause no error

#### Scenario: No conventional roots present

- **WHEN** the target is scanned on a machine where none of the conventional dev-root directories exist
- **THEN** the target SHALL report a not-installed/empty status and offer no cleanup items

#### Scenario: Artifact cleanup leaves the repository intact

- **WHEN** the user cleans artifacts for a discovered project
- **THEN** only directories matching the artifact name list SHALL be removed
- **AND** the project's `.git` directory, source files, and configuration files SHALL remain untouched

#### Scenario: Conductor workspaces are not duplicated

- **WHEN** the target enumerates projects
- **THEN** repositories located under `~/conductor/workspaces` SHALL be excluded, as they are already covered by the Conductor artifact-cleanup target

#### Scenario: Only git repositories are treated as projects

- **WHEN** a conventional root contains a subdirectory without a `.git` entry
- **THEN** that subdirectory SHALL NOT be listed as a project and its contents SHALL NOT be cleaned

#### Scenario: Artifact cleanup requires no confirmation

- **WHEN** the user clicks the cleanup action for a project
- **THEN** the command SHALL run immediately without a double-confirm prompt, consistent with Tier 1 behavior
