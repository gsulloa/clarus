# conductor-artifact-cleanup Specification

## Purpose
Tier 1 catalog target that cleans regenerable artifact directories inside Conductor workspaces without deleting the workspace itself.

## Requirements

### Requirement: Tier 1 target cleans regenerable artifacts inside Conductor workspaces

The system SHALL expose a Tier 1 catalog target named "Conductor — regenerable artifacts" that enumerates the same individual workspaces as the Conductor delete target and offers per-workspace cleanup of only regenerable artifact directories. The target SHALL clean the following directory names wherever they appear inside a workspace (up to a bounded depth): `node_modules`, `.next`, `dist`, `cdk.out`, `.turbo`, `target`, `__pycache__`, `.venv`, `venv`, `build`, `.cache`, `.parcel-cache`. The cleanup SHALL NOT delete source files, git history, or any directory not in this list. No confirmation is required beyond a single click.

#### Scenario: Artifact cleanup leaves workspace intact

- **WHEN** the user cleans artifacts for a workspace via the Tier 1 target
- **THEN** only directories matching the artifact name list SHALL be removed
- **AND** the workspace's `.git` directory, source files, and configuration files SHALL remain untouched

#### Scenario: Artifact target enumerates same workspaces as delete target

- **WHEN** the artifact-cleanup target is scanned
- **THEN** it SHALL list the same individual workspaces as the Conductor workspace-delete target, including nested workspaces expanded from project containers

#### Scenario: Artifact cleanup requires no confirmation

- **WHEN** the user clicks the artifact-cleanup action for a workspace
- **THEN** the command SHALL run immediately without a double-confirm prompt, consistent with Tier 1 behavior

#### Scenario: Freed space is visible after cleanup

- **WHEN** an artifact cleanup command completes
- **THEN** the disk-free readout SHALL update to reflect the space reclaimed
