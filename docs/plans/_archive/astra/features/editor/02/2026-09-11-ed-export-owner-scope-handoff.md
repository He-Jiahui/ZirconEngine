---
record_kind: dependency_handoff
status: blocked_owner_scope
created_at: 2026-09-11
plan: docs/plans/astra/features/editor/02-export-failure-receipts.md
milestone: ED-EXPORT-P0-002/003
session: astra-ed-export-owner-handoff-20260911-01a090c1
---

# Editor export P0 owner-scope handoff

## Decision

This bounded ED-EXPORT-P0-002/003 lane did not modify production or test
source. The implementation surface is not cleanly owned: the CompileHost
adapter, core receipt pipeline, wizard adapter, terminal report, manager, and
their focused regressions are already dirty or staged under several archived
or stale sessions. The goal's P0 authorization permits the repair, but does
not permit taking over foreign dirty bytes. A new owner must reconcile the
existing diff before any narrow repair or acceptance claim.

ED-A5 and ED-A6 were not touched.

## Contract and current-source evidence

The plan `02-export-failure-receipts.md` remains `in_progress` and calls for
nonzero/unknown CompileHost exits to fail, failed core reports to replace a
previous Passed receipt, versioned/corrupt receipt handling, atomic writes,
and cancellation preservation. The adjacent plan
`04-export-terminal-outcome.md` remains `in_progress` and calls for fatal
diagnostics, missing providers/receipts, and nonzero/unknown host/native
outcomes to remain failures.

The current dirty source already contains the following behavior, which must
be reviewed against the owning diff rather than duplicated:

| Boundary | Evidence in current bytes | Ownership state |
| --- | --- | --- |
| Native CompileHost exit | `zircon_editor/src/core/export/stages/compile_host.rs:218-228` writes the output manifest, then maps every `!status.success()` (including an unknown `Option<i32>` code) to typed `ZirconBuildCommandError::Exit`. | Modified; stale owner `astra-optimize-20260909-root`; no live lease. Matrix request `856f482dca7b4763b539df092c356847`; current hash `01ee684edd865a238c8c3212ee12dc62c1293da8fcf0a7ba92d4bb2d9a26a2ad`. |
| Core pipeline failure report | `zircon_editor/src/core/export/pipeline.rs:52-115` records `Failed` on prepare/execute errors and returns `ExportPipelineRunError`; `:134-143` gates reuse through `can_reuse`. | Modified; stale owner `astra-optimize-20260909-root`; no live lease. Matrix request `128ddbb20b144301aa88087c35d20588`; current hash `890bedaaed9d0b6b703ff25da375f0e0cada1614109c9f6d00455cf4dd9ea85b`. |
| Wizard/core receipt boundary | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/core_pipeline.rs:60-86` maps pipeline failures and persists them; `:150-161` preserves the original failure when receipt persistence fails; `:179-215` handles receipt version/corruption/NotFound; `:218-249` uses a staging file and atomic replacement; `:332-357` retires the old success before execution and rejects any exit other than `Some(0)`. | Modified; archived owner `astra-export-failure-20260905`; no live lease. Matrix request `3683048a02df4a80b99260523dff5bf3`; current hash `9ee585fa9836fb1ccb25528dd78ee6702e7fc43944fa54963d958b97a5b08b56`. |
| Typed wizard error | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/error.rs:54-59` carries both primary failure and receipt-write failure; `:87-91` carries `Option<i32>` stage outcomes. | Modified under archived/stale attribution (no executable owner or live lease). |
| Terminal report result | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/report.rs:19-54` checks fatal diagnostics, required-provider absence, receipt consistency, and host/native status; `:56-64` converts any such reason to `ReportFailed`. | Modified; archived owner `astra-export-outcome-20260905`; no live lease. Matrix request `37bbacf31a184747abd2d02d91e969d6`; current hash `e8158a790195e96def236ea337e24b6dd2321c2102637556bdbcf402253a26b5`. |
| Manager production paths | `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs:255-290` and `:333-375` construct and finalize reports, including fatal materialization paths. | Modified; archived owner `astra-export-outcome-20260905`; no live lease. Matrix request `c1dc380ec3ba43f28c9dc42acb89f454`; current hash `c76deb42c4e1e597ee58fd582fc668961ec959d51f380f568e596228d3c047d9`. |

The index also contains focused additions including
`export_build/manager/astra_outcome_tests.rs`,
`export_build/report/astra_outcome_tests.rs`,
`wizard/execution/core_pipeline/failure_receipt_tests.rs`, and
`wizard/job/astra_terminal_tests.rs`, while the core/export and wizard trees
have many additional modified files. These tests have no clean executable
owner in the current matrix and must be reconciled with the source owner.

## Why this is blocked

P0-003 crosses process status, pipeline failure records, receipt load/write,
and wizard error mapping. P0-002 consumes the same result through
`EditorExportBuildReport::failure_reason` and manager finalization. A one-file
patch would either duplicate behavior already present in the dirty bytes or
leave a mismatched receipt/report contract. The coordinator reports a
degraded baseline with an active maintenance blocker; the matrix reports
`attribution_hash_stale`, `attribution_baseline_stale`,
`owner_not_executable`, and `live_lease_missing` on the behavior-bearing
paths. Therefore no exact implementation lease can be safely claimed in this
lane.

## Dependency-ready owner scope

1. **Core export owner (P0-003):** reconcile the existing diff, then claim as
   one atomic source/test set:
   `zircon_editor/src/core/export/stages/compile_host.rs`,
   `zircon_editor/src/core/export/pipeline.rs`,
   `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard/execution/core_pipeline.rs`,
   `zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/error.rs`,
   and `wizard/execution/core_pipeline/failure_receipt_tests.rs`. Keep the
   typed exit, failure-receipt, retry, version/corruption, cancellation, and
   atomic-replace assertions together. Do not broaden into ED-A6 projection.

2. **Terminal outcome owner (P0-002):** reconcile and claim
   `export_build/manager.rs`, `export_build/report.rs`,
   `export_build/manager/astra_outcome_tests.rs`, and
   `export_build/report/astra_outcome_tests.rs` as one report/result set.
   Include wizard controller/job paths only if the reconciled diff proves they
   are required to carry the typed terminal outcome; otherwise leave them to
   their existing owners.

3. After source ownership is restored, run the plan-declared focused managed
   validation and product/native gates. A successful static review or staged
   test file is not acceptance. This handoff intentionally ran no Cargo,
   native process, DLL, or product validation.

## Coordination receipt

Session `astra-ed-export-owner-handoff-20260911-01a090c1` is the sole owner of
this handoff record. No production or test source lease was claimed. The
record lease is released after attribution, and the session transitions to
`waiting_validation`; managed validation and acceptance remain pending.
