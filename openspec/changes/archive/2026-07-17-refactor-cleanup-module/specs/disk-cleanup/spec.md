## ADDED Requirements

### Requirement: Stable cleanup command and serialization surface

The disk-cleanup backend SHALL expose exactly four Tauri commands — `scan_cleanup_targets`, `clean_target`, `clean_item`, and `disk_free` — reachable from the crate root as `cleanup::<command>`, regardless of how the module is internally organized. Refactoring the module's file layout SHALL NOT change these command names, their argument or return types, the camelCase JSON field names of `Target`, `Item`, `CleanupScan`, and `CleanResult`, the `cleanup://catalog` and `cleanup://target` event names and payloads, the catalog contents (target ids, tiers, statuses, and cleanup command strings), or any observable runtime behavior. Internal helpers, builders, and target constructors MAY be moved between files and MAY have their visibility narrowed to the module tree.

#### Scenario: Command surface unchanged after refactor

- **WHEN** `lib.rs` registers `cleanup::scan_cleanup_targets`, `cleanup::clean_target`, `cleanup::clean_item`, and `cleanup::disk_free` in `tauri::generate_handler!`
- **THEN** all four commands SHALL resolve without changes to `lib.rs`
- **AND** their argument and return types SHALL be unchanged

#### Scenario: Serialized shapes unchanged after refactor

- **WHEN** `scan_cleanup_targets` emits `cleanup://catalog` and `cleanup://target` events and returns a `CleanupScan`
- **THEN** the event names and the camelCase JSON field names of every payload SHALL be identical to before the refactor

#### Scenario: Catalog contents unchanged after refactor

- **WHEN** `catalog_defs()` is built after the refactor
- **THEN** it SHALL contain the same set of target ids with the same tier, status-detection logic, and cleanup command strings as before the refactor
- **AND** the existing cleanup test suite SHALL pass without modification to its assertions
