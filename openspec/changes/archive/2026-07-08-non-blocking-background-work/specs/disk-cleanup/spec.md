## ADDED Requirements

### Requirement: Background work never blocks the UI

Scan and cleanup operations SHALL execute without blocking the UI thread. While a scan or any cleanup command is running, the window SHALL keep rendering and remain interactive, and the application SHALL NOT cause the operating system to display a wait/busy ("beachball") cursor. This SHALL hold for long-running operations, including cleanups that quit applications or wait for Docker to start (up to 90 seconds).

#### Scenario: Window stays responsive during a scan

- **WHEN** the user starts an analysis and sizes are being measured
- **THEN** the window SHALL continue to paint and respond to input (scrolling, selecting a target, hovering)
- **AND** no wait/busy cursor SHALL be shown

#### Scenario: Window stays responsive during a long cleanup

- **WHEN** a cleanup command runs that takes a long time (e.g. Docker prune waiting for Docker to start)
- **THEN** the rest of the catalog SHALL remain scrollable, selectable, and actionable
- **AND** the window SHALL NOT freeze and no wait/busy cursor SHALL be shown

#### Scenario: Other targets remain actionable while one is cleaning

- **WHEN** one target is in the cleaning state
- **THEN** other actionable targets SHALL still accept clicks and start their own cleanup independently

### Requirement: Scan streams progress incrementally

The scan operation SHALL report progress as it runs rather than only on completion. It SHALL make the set of catalog targets and a total target count available before sizes are measured, and it SHALL report each target's measured size as measuring completes for that target, so the UI can advance a determinate progress indicator (measured of total) that reflects real progress.

#### Scenario: Total is known before measuring completes

- **WHEN** a scan begins and the catalog has been enumerated
- **THEN** the total number of targets SHALL be available to the UI before all sizes are measured

#### Scenario: Per-target progress is reported as it completes

- **WHEN** a target's size measurement finishes
- **THEN** that target's measured size SHALL be reported to the UI at that time, not withheld until every target is measured
- **AND** the count of measured targets SHALL not exceed the total

### Requirement: Appropriate loading feedback during background work

The UI SHALL present appropriate, non-blocking loading feedback during background work instead of a frozen screen. During analysis it SHALL show catalog rows immediately with placeholder (skeleton) sizes and a determinate progress indicator that advances as targets are measured. During a cleanup it SHALL show a live per-row busy state that conveys the operation is ongoing, including elapsed time for long-running cleanups. All loading animations SHALL respect the user's reduced-motion preference.

#### Scenario: Rows appear with skeletons before sizes arrive

- **WHEN** a scan has enumerated the catalog but not yet measured sizes
- **THEN** the target rows SHALL be visible with a placeholder/skeleton size
- **AND** each row's size SHALL be filled in as its measurement completes

#### Scenario: Determinate progress during analysis

- **WHEN** a scan is measuring targets
- **THEN** a progress indicator SHALL reflect the proportion of targets measured out of the total

#### Scenario: Long cleanup shows ongoing feedback

- **WHEN** a cleanup runs longer than a moment
- **THEN** its row SHALL show an ongoing busy state with elapsed time (and, where relevant, a hint that the step may take time, such as starting Docker)
- **AND** this feedback SHALL update live rather than appearing frozen

#### Scenario: Reduced motion is respected

- **WHEN** the user has enabled the reduced-motion preference
- **THEN** non-essential loading animations (e.g. skeleton shimmer) SHALL be disabled while progress state still updates
