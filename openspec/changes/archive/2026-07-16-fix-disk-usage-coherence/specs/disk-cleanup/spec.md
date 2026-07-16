## MODIFIED Requirements

### Requirement: Scan measures targets without deleting

The system SHALL provide a scan operation that measures size and availability of every catalog target and returns the current disk usage of the data volume — **free space, total capacity, and used space** — without deleting or modifying anything. Sizes SHALL be computed in bytes for accurate totals and sorting. Free, total, and used space SHALL use the same `df` semantics as the script (`/System/Volumes/Data`) and SHALL be read from a **single `df` snapshot** (one invocation) so the three figures are mutually consistent. The **used** figure reported to the UI SHALL be derived as `total − free` (not the APFS per-volume `Used` column), so that `used + free = total` holds exactly for the reported figures.

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
- **AND** these figures SHALL come from a single `df /System/Volumes/Data` snapshot used for free space
- **AND** the reported used space SHALL equal total capacity minus free space

#### Scenario: Reported figures are mutually consistent

- **WHEN** the scan reports used, free, and total for the data volume
- **THEN** used plus free SHALL equal total for both the numeric (GB) fields and the human-readable strings within rounding
- **AND** the numeric and human-readable variants SHALL be derived from the same `df` snapshot rather than from independent invocations taken at different instants

#### Scenario: Target status reflects availability

- **WHEN** a target's path is empty, its backing tool is missing, or the app is not installed
- **THEN** the target SHALL report a status distinguishing these cases (e.g. Available, Empty, ToolMissing, NotInstalled)

### Requirement: Disk usage summary is displayed

The system SHALL present a disk-usage summary for the data volume that shows used space, free space, and total capacity — not only cleanup opportunities and free space. The three figures SHALL be mutually coherent such that used plus free equals total. The summary SHALL include a proportional capacity indicator (used relative to total, equivalently `1 − free/total`) so the user can judge at a glance how full the disk is. The summary SHALL keep exposing the cleanup-session progress (free space before, free space now, and total freed).

#### Scenario: Summary shows used, free, and total

- **WHEN** a scan has completed
- **THEN** the readout SHALL display the used space, the free space, and the total capacity of the data volume

#### Scenario: Displayed figures add up

- **WHEN** the used, free, and total figures are displayed together
- **THEN** the displayed used and free SHALL sum to the displayed total (within unit rounding), so the three numbers reconcile for the user

#### Scenario: Capacity indicator reflects fullness

- **WHEN** the disk-usage summary is shown
- **THEN** a proportional indicator SHALL represent used space relative to total capacity
- **AND** the indicator's filled proportion SHALL be consistent with the displayed free space (i.e. the unfilled portion corresponds to free ÷ total)

#### Scenario: Cleanup progress remains visible

- **WHEN** one or more cleanups have run in the session
- **THEN** the summary SHALL still show free space before, free space now, and the total amount freed
