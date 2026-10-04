---
title: Runtime206 durability publication fence ownership handoff
category: zircon_runtime
report_id: Runtime206-ASSETREG-P0-002-handoff-2026-09-29
date: 2026-09-29
session_id: astra-optimize-20260926-batch-a
plan_source: docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
implementation_status: blocked_by_active_shared_scope
validation_status: not_run_no_implementation
performance_status: not_measured
---

# Runtime206: durability publication fence handoff

`ASSETREG-P0-002` remains open. In `ProjectManager::scan_and_import` and the watch path, a completed file commit installs `candidate` into the live manager before `ensure_durable()` checks the commit marker. A marker sync failure can therefore return `Err` after the new generation is visible. The existing `unsynced_commit_point_installs_the_live_generation_before_reporting_pending_recovery` test asserts that behavior; it is a regression fixture for the current implementation, not acceptance evidence for the plan's publication contract.

The transaction engine retains a complete visible commit frame and journal for restart recovery after this fault. Moving `*self = candidate` below `ensure_durable()` alone would leave a usable live manager stale against the on-disk generation. The repair must give the operation an observable `Durable` or typed `AcceptedRecoveryPending(OperationId)` disposition, and make the API result, live project/resource/catalog/watch state, generation event, and restart recovery agree on that disposition. A pending generation must prevent blind follow-up writes until resolved. A normal error must not leave an unreported published candidate.

The required test file `zircon_runtime/src/asset/tests/project/manager/full_generation.rs` was in the active `Source comment audit M1` Session `1d9a143c-c6a1-4da3-a702-eeeff3e26b8f` write scope at the 2026-09-29 ownership check. The source paths inspected had no live lease, but editing production code without replacing the contradictory test and covering the manager, watch, and restart contract would create a partial fix. No Runtime206 source or test was changed in this handoff. Reconcile that active scope before implementing the full slice.

Required acceptance: a fault-after-commit-marker regression that checks a typed pending receipt or no live publication on an ordinary error; follow-up write behavior; full/watch/targeted and project-asset-manager state and event consistency; restart recovery with the same operation and generation identity. Managed Rust validation and product performance remain pending.
