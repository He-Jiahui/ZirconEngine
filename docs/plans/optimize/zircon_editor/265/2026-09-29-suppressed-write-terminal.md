---
related_code:
  - zircon_editor/src/core/settings/authority.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/persistence.rs
  - zircon_editor/src/core/settings/mutation.rs
  - zircon_editor/src/core/settings/mutation/health.rs
  - zircon_editor/src/core/settings/persistence_suppressed_write_tests.rs
  - zircon_editor/src/core/context/builder/settings_persistence_health.rs
  - zircon_editor/src/ui/settings/persistence_health_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_settings_window/persistence_health.rs
  - tools/tests/test_editor17_settings_persistence_health_contract.py
plan_source: docs/plans/optimize/zircon_editor/265-editor-settings-preferences-project-settings-scope-schema-overlay-persistence-migration-restart-plugin-window-current-working-tree-review.md
plan_item: E-SET-P1-59
status: partial_source_candidate_managed_validation_pending
---

# Editor265: distinguish a skipped settings write from a completed write

## Failure and repair

`SettingsAuthority::prepare_persistent_layer_for_write` returned `Ok(None)` when a project
transition was active, the queued store belonged to a stale project path, or the active project
settings source was invalid. `SettingsStore::save_authority_layer` converted every such case to
`Ok(())`. The Runtime I/O lane then reported `Succeeded`, and Editor health promoted it to
`Durable` although no settings file was written.

The authority now classifies `Ready`, `SkippedStale`, and `BlockedInvalid`. The store reports
`Written` only after its existing atomic write returns successfully. Each admitted request retains
its own result slot; the worker fills it before returning to the Runtime lane. The Settings
terminal and health observer combine that result with the lane terminal. A successful lane result
with no recorded write disposition is `MissingWriteDisposition`, never `Durable`. Stale and invalid
results are non-retryable, and the invalid/protocol states use the existing localized failure
message without creating a retry action. The settings window paints that message even without a
retry scope. Runtime lane and fence semantics are unchanged; the ticket's legacy `terminal` and
`wait_until` methods report lane state only.

`NoTarget` remains an admission rejection (`TargetUnavailable` or `ProjectRootRequired`) before a
ticket exists. It is not fabricated as a worker result. This slice does not change that contract.

## Regression and performance gates

New focused tests exercise an unbound project, a stale store submitted after a switch, a write
queued before a project switch, and an invalid project source. They require the Settings terminal
to distinguish skipped/blocked work and
require the source files to remain absent or byte-identical. The invalid-source test also checks
the terminal observer. The preexisting real-write and retry regressions remain in the unfiltered
Editor library test set. Pinned Rustfmt, source checks, and 31 Editor settings Python contract
tests passed. These are static evidence only; managed Cargo execution is pending in the grouped
successor batch.

This repair adds one small per-request disposition slot outside the settings hot read path. No
Release latency, allocation, or product-memory result has been measured, so no performance pass is
claimed.

## Remaining Editor265 contract

The worker still serializes the live authority at execution time. It does not carry immutable
bytes or a digest for the ticket's file generation; therefore `Written` proves a file write, but
does not prove that the bytes correspond to the original ticket. The plan's exact-generation
`Durable` requirement and `E-SET-P1-15` remain open. Transition-window, invalid-source recovery,
user-visible reason-specific localization, and product close/restart qualification remain open.
