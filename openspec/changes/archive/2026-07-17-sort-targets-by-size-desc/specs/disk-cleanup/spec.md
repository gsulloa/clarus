## ADDED Requirements

### Requirement: Targets and subitems ordered by size within their group

The system SHALL order cleanup candidates by measured size **descending** (largest first) within their group, where a "group" is a single tier for targets and a single container target for its subitems. Ordering SHALL never cross group boundaries: the fixed Tier 1 → Tier 2 → Tier 3 group order SHALL be preserved, and a larger target in a later tier SHALL NOT appear above a smaller target in an earlier tier. When two candidates in the same group have equal `sizeBytes` — including the pre-measurement state where sizes are still `0` — the system SHALL fall back to a deterministic alphabetical order (by target `name`, and by subitem `label`) so that skeleton rows shown before measurement remain stable.

#### Scenario: Targets ordered largest-first within a tier

- **WHEN** the review surface shows measured targets within a tier
- **THEN** those targets SHALL appear ordered by measured size from largest to smallest
- **AND** the Tier 1 → Tier 2 → Tier 3 grouping order SHALL be unchanged

#### Scenario: Size order never crosses tiers

- **WHEN** a Tier 3 target is larger than every Tier 1 target
- **THEN** the Tier 3 target SHALL still render within the Tier 3 group, below all Tier 1 and Tier 2 targets

#### Scenario: Subitems ordered largest-first within a container target

- **WHEN** a container target is expanded to show its subitems
- **THEN** the subitems SHALL appear ordered by measured size from largest to smallest

#### Scenario: Deterministic order before and during measurement

- **WHEN** the catalog is shown before sizes are measured (all sizes `0`) or several candidates in a group share the same size
- **THEN** those candidates SHALL be ordered alphabetically (targets by `name`, subitems by `label`)
- **AND** rows SHALL settle into size-descending order as each measurement streams in
