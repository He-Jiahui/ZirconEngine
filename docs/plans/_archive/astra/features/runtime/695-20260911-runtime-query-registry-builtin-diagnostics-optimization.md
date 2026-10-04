---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/04/2026-08-26-unstable-asset-registry-entry-sort.md
  - docs/plans/optimize/zircon_runtime/187-runtime-scene-ecs-world-archetype-query-schedule-generation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/188-runtime-asset-resource-lifecycle-locator-registry-load-cache-import-cook-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/205-runtime-resource-lifecycle-load-ticket-cache-residency-generation-reload-cancellation-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/44-process-diagnostic-log-router-filter-record-queue-sink-durability-rotation-crash-multi-session-product-integration-review.md
related_records:
  - docs/plans/astra/features/runtime/694-runtime629-manifest-membership-completion.md
  - docs/plans/astra/features/runtime/685-20260911-runtime-diagnostics-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/743-runtime206-contract-repair.md
  - docs/plans/astra/features/runtime/744-runtime206-secondary-query-postings.md
  - docs/plans/astra/features/runtime/745-runtime206-referencer-binary-sort.md
  - docs/plans/astra/features/runtime/746-runtime206-registry-build-capacity.md
---

# Runtime Query, Registry, Builtin, And Diagnostic Optimization Completion List

This completion list tracks bounded Runtime hot-path changes selected from the
`docs/plans/optimize` reviews. Existing node-backed projections and serialized
contracts remain stable; the unbounded component query also fixes the old
componentless-entity omission by matching the bounded query's authoritative
entity semantics. Managed Windows compile/test and Release measurements remain
a separate acceptance gate.

## Plan completion list

| Area | Implementation | Static/regression evidence | Status |
| --- | --- | --- | --- |
| Runtime04 / ASTREG | `AssetRegistryIndex` maintains a `BTreeMap` canonical-path index and exposes a borrowed ordered iterator. Insert, source removal, and remove/reinsert paths maintain the index, eliminating copy-plus-sort on reads. | Legacy-order equivalence, removal, remove/reinsert, iterator, and source-contract tests in `asset_registry_index/optimization_tests.rs`. | implemented_pending_validation |
| ECS4-P1-023 | The unbounded reflected `Components` query projects directly from `entity_ids_for_query()` and no longer materializes full `node_records()` before field selection. Bounded budget/deadline behavior is unchanged, and stable entities without reflected fields now retain the same empty-row semantics as the bounded path. | Source guard plus unbounded/bounded filter, selection, scrambled-batch entity-ID ordering, empty-entity retention, JSON, generation, and not-modified equivalence tests, plus an ignored Release comparison against the retired projection path in `scene/inspection/snapshot/component_projection_tests.rs`. | implemented_pending_validation |
| AST4-P2-014 | Builtin descriptors now carry typed kind/factory metadata. Residency uses a single-entry lazy lookup; bootstrap registration retains the eager five-entry materialization contract. Missing/non-builtin lookups construct no payload. | Selected-only, missing, bootstrap/lazy equivalence, residency source guard, and ignored Release p50/p95/p99 benchmark in `builtin_resources/lazy_lookup_tests.rs`. | implemented_pending_validation |
| R44-P1-12 adjacent formatter slice | Diagnostic series formatting reserves one output `String` and writes numeric fields directly, avoiding per-field temporary `format!` strings. Existing output ordering and omission rules remain unchanged. | Sixteen optional-field combinations, empty-current snapshot, source guard, and ignored Release paired benchmark in `diagnostics/tests/format_schedule.rs`. | implemented_pending_validation |
| Runtime206/Runtime85 contract repair | Source-contract assertions for the existing asset type-posting and project-root dedup optimizations now tolerate rustfmt whitespace/import ordering without weakening the required lookup, capacity, or ordering guards. | Focused repair set `7/7`; comprehensive non-tooling Runtime/Editor performance-contract and pressure loader `2039/2039` across 548 files. | implemented_pending_validation |
| Runtime206 / secondary query postings | `AssetRegistryIndex` now owns tag, package, and ordered path-prefix postings; insert/source removal retire all UUID memberships, and non-type filters begin from the smallest direct posting before exact predicate retention. Path-prefix ranges borrow the filter key and are lazy when a direct posting exists. | Runtime206 source contracts plus the type guard pass `17/17` (the adjacent Runtime85 root guard brings the combined set to `20/20`); behavior coverage includes labeled subassets, empty-bucket retirement, and direct-posting/path mismatch; deterministic tag-pressure model is `32x` fewer candidates. The current batched non-tooling loader count is recorded below. | implemented_pending_validation |
| Runtime206 / referencer binary sort | `AssetUuid::binary_key()` exposes a borrowed 16-byte key and referencer queries sort with `sort_unstable_by`, removing UUID display-string construction from the ordering hot path. | New source contract passes `3/3`; the combined Runtime206 focused set now passes `17/17`, and the deterministic 65,536-referencer model reports zero owned sort keys after the change. | implemented_pending_validation |
| Runtime206 / registry build capacity | Bulk index construction reserves the iterator lower bound for primary maps and streams dependency-path projection one entry at a time instead of retaining an all-path staging vector; batch bootstrap also defers reverse-path empty-bucket pruning to one final pass. | New source/pressure contract passes `4/4`; behavior coverage preserves resolved and unresolved dependency/referencer semantics; the declared 1,048,576-entry model reduces transient path staging to at most four records and reverse-path pruning from 1,048,576 scans to one. | implemented_pending_validation |

## Batched local evidence

- The focused Runtime/Editor contract batch passed `53/53` in `0.165s`.
- A broader batched Python contract run passed `157/157` in `50.015s`; the
  focused Runtime/Editor subset passed `7/7` in `0.081s` after the ordering
  regression was added.
- After adding the Release comparison for direct component projection, the
  affected Runtime/Editor contract batch was rerun as `15/15` in `0.044s`.
- After locking the stable-entity-without-reflected-fields boundary, the same
  focused Runtime/Editor contract batch was rerun as `15/15` in `1.613s`;
  the component projection source and diff checks also passed.
- Final optimization/compile-support source-contract assertions passed
  `22/22` across the touched Runtime and Editor production/test areas.
- The later Runtime206/Runtime85 contract-robustness repair set passes `7/7`,
  and the comprehensive non-tooling Runtime/Editor performance-contract and
  pressure loader passes `2039/2039` across 548 files.
- Rustfmt parse-only validation passed for all 20 touched Rust files; this is
  syntax evidence, not a Cargo compile.
- `git diff --check` plus trailing-whitespace checks passed for all 20 touched
  files (only existing line-ending notices were emitted by Git).
- Tooling remains intentionally out of scope for this batch.
- A repository-wide `unittest discover` was started for diagnostics only and
  interrupted after unrelated dirty-worktree failures/errors appeared; it is
  not counted as a pass or a failure for this scoped batch.
- After Runtime206 P1-047 and the bulk-build capacity/streaming slice, the
  final one-process non-tooling Runtime/Editor loader covered `551` files and
  passed `2052/2052` tests in the latest current-source batch (`80.282s` under the current local load); this remains local static/model
  evidence and does not replace the managed Cargo/Release gate.

## Managed acceptance gate

The four ignored Release benchmarks define the relative gates used for
acceptance: canonical registry iteration at most 50% of copy-and-sort P95;
direct component projection at most 50% of the retired `node_records()` P95;
lazy builtin lookup at most 70% of eager P95; and single-buffer diagnostic
formatting at most 95% of the legacy formatter P95. No local run is promoted to
product evidence.

Current managed Cargo admission is still blocked before compilation by the
external dirty worktree `E:\Git\zr_vm` and the existing `--locked` lockfile
drift recorded in the asynchronous admission log. Therefore this record does
not claim Rust tests, Windows Release timings, allocation counts, or completed
performance acceptance.
