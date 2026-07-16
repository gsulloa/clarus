## MODIFIED Requirements

### Requirement: Scan measures targets without deleting

The system SHALL provide a scan operation that measures size and availability of every catalog target and returns the current disk usage of the data volume — **free space, total capacity, and used space** — without deleting or modifying anything. Sizes SHALL be computed in bytes for accurate totals and sorting, and free/total/used space SHALL use the same `df` semantics as the script (`/System/Volumes/Data`), read from a single `df` snapshot so the three figures are mutually consistent.

#### Scenario: Scan is non-destructive

- **WHEN** the user runs the scan
- **THEN** no target is deleted, quit, or modified
- **AND** each target reports its measured size and a status

#### Scenario: Free space captured before actions

- **WHEN** the scan completes
- **THEN** the current free disk space SHALL be recorded as the "before" baseline for later comparison

#### Scenario: Total and used space are reported

- **WHEN** the scan completes
- **THEN** the total capacity and the used space of the data volume SHALL be reported alongside free space
- **AND** these figures SHALL come from the same `df /System/Volumes/Data` snapshot used for free space

#### Scenario: Target status reflects availability

- **WHEN** a target's path is empty, its backing tool is missing, or the app is not installed
- **THEN** the target SHALL report a status distinguishing these cases (e.g. Available, Empty, ToolMissing, NotInstalled)

### Requirement: Per-item cleanup action

The system SHALL let the user clean each actionable target independently via its own action button. Clicking the button SHALL run only that target's exact command. Targets whose status is Empty, ToolMissing, or NotInstalled SHALL have their action disabled. After a cleanup runs, the row SHALL transition through cleaning → done (with freed amount) or error (with message), and the disk-usage readout SHALL be recomputed.

#### Scenario: Single target cleanup

- **WHEN** the user clicks the action button for an available target
- **THEN** only that target's command runs
- **AND** the row shows a cleaning state, then the freed amount on success

#### Scenario: Disabled when nothing to clean

- **WHEN** a target's status is Empty, ToolMissing, or NotInstalled
- **THEN** its action button SHALL be disabled

#### Scenario: Error is reported without cascading

- **WHEN** a target's cleanup command fails
- **THEN** the row SHALL show an error message
- **AND** no other target SHALL be affected

#### Scenario: Disk usage updates after action

- **WHEN** a cleanup action completes
- **THEN** the disk-usage readout SHALL update to reflect current free space, used space, and total capacity, and the amount freed relative to the baseline

## ADDED Requirements

### Requirement: Disk usage summary is displayed

The system SHALL present a disk-usage summary for the data volume that shows used space, free space, and total capacity — not only cleanup opportunities and free space. The summary SHALL include a proportional capacity indicator (used relative to total) so the user can judge at a glance how full the disk is. The summary SHALL keep exposing the cleanup-session progress (free space before, free space now, and total freed).

#### Scenario: Summary shows used, free, and total

- **WHEN** a scan has completed
- **THEN** the readout SHALL display the used space, the free space, and the total capacity of the data volume

#### Scenario: Capacity indicator reflects fullness

- **WHEN** the disk-usage summary is shown
- **THEN** a proportional indicator SHALL represent used space relative to total capacity

#### Scenario: Cleanup progress remains visible

- **WHEN** one or more cleanups have run in the session
- **THEN** the summary SHALL still show free space before, free space now, and the total amount freed
