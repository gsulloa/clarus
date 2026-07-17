## ADDED Requirements

### Requirement: Tier 1 target reclaims stale per-user Darwin temporary files

The system SHALL expose a Tier 1 catalog target named "System temporary files" that reclaims space under the per-user Darwin temporary directory. The system SHALL resolve the base path portably via `getconf DARWIN_USER_TEMP_DIR` and SHALL NOT hardcode the per-user `/private/var/folders/<hash>` segment. The target SHALL clean two areas: (1) orphaned Google Chrome `code_sign_clone` directories under the sibling `X/` directory, and (2) stale entries directly under the `T/` directory. Deletion of `T/` entries SHALL be age-gated so that only top-level entries not modified within a configurable threshold (default 3 days) are removed, to avoid deleting temporary files in use by running processes. The target SHALL be Tier 1 and SHALL NOT require double confirmation.

#### Scenario: Temp path resolved portably

- **WHEN** the target is scanned
- **THEN** the base directory SHALL be obtained from `getconf DARWIN_USER_TEMP_DIR`
- **AND** no per-user hash path SHALL be hardcoded in the catalog

#### Scenario: Stale T entries removed, recent ones preserved

- **WHEN** the user runs the cleanup and `T/` contains a directory last modified 10 days ago and another modified 1 hour ago (with a 3-day threshold)
- **THEN** the 10-day-old entry SHALL be removed
- **AND** the 1-hour-old entry SHALL remain

#### Scenario: Chrome quit before removing code-sign clones

- **WHEN** the user runs the cleanup and orphaned `code_sign_clone` directories exist under `X/`
- **THEN** Google Chrome SHALL be quit first
- **AND** the orphaned clone directories SHALL then be removed

#### Scenario: No confirmation required

- **WHEN** the user clicks the cleanup action
- **THEN** the command SHALL run immediately without a double-confirm prompt, consistent with Tier 1 behavior

#### Scenario: Absent temp areas cause no error

- **WHEN** the resolved `T/` or `X/` directories do not exist or contain nothing eligible
- **THEN** the target SHALL report an empty status and complete without error
