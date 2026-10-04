---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/06/2026-08-24-plugin-admission-borrowed-dfs.md
  - docs/plans/optimize/zircon_editor/06/2026-08-25-plugin-registration-index.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-capability-snapshot-hash-union.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-plugin-replacement-hash-membership.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-reused-native-status-diagnostics.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-runtime-capabilities-unstable-sort.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-single-pass-native-load-state-classification.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-swap-remove-failed-lifecycle-stage.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-contribution-capability-sort.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-event-consumer-sort.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-extension-normalization-sort.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-plugin-capability-index.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-plugin-capability-projection.md
  - docs/plans/optimize/zircon_editor/06/2026-08-26-unstable-viewport-overlay-capability-sort.md
  - docs/plans/optimize/zircon_editor/06/2026-08-27-borrowed-extension-view-validation.md
related_records:
  - docs/plans/astra/features/editor/24-plugin-manager-contract-maintenance.md
  - docs/plans/astra/features/editor/988-editor06-borrowed-extension-view-validation.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor06 · plugin-manager optimization completion list

The Editor06 plugin-manager slices are implemented in the current source and
retain lifecycle ordering, capability precedence, duplicate rejection, status
diagnostics, and extension-view error semantics. This list records all fifteen
leaf plans while keeping managed Cargo, allocator, Release, and product
qualification pending. Tooling remains out of scope.

| Plan slice | Optimization boundary | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Plugin admission borrowed DFS | Borrow plugin dependency traversal and preserve cycle/ordering diagnostics. | Focused source/model and managed plugin-admission tests. | implemented_pending_validation |
| Plugin registration index | Index registered plugin identities for direct admission lookup. | Registration parity, duplicate/error behavior, managed Release evidence. | implemented_pending_validation |
| Capability snapshot hash union | Build capability unions through hash membership before deterministic projection. | `EDITOR06_CAPABILITY_SNAPSHOT_HASH_UNION_BENCH_V1`, 60% P95 gate. | implemented_pending_validation |
| Plugin replacement hash membership | Replace repeated ordered membership scans with borrowed hash membership. | `EDITOR06_PLUGIN_REPLACEMENT_HASH_MEMBERSHIP_BENCH_V1`, 60% P95 gate. | implemented_pending_validation |
| Reused native-status diagnostics | Reuse status/diagnostic projections without repeated owned strings. | Native lifecycle parity and managed allocation/latency evidence. | implemented_pending_validation |
| Runtime capability unstable sort | Use unstable sorting where capability order is non-observable. | Capability ordering contract and managed Release evidence. | implemented_pending_validation |
| Single-pass native load-state classification | Classify native load state in one traversal. | State/error parity and managed source/test gate. | implemented_pending_validation |
| Swap-remove failed lifecycle stage | Remove failed entries without preserving irrelevant order. | Failure retention, diagnostics, and managed allocation gate. | implemented_pending_validation |
| Unstable contribution/event/extension sorts | Remove stable-sort overhead where descriptors have explicit tie-breakers or no observable order. | Capability/event/extension ordering contracts and Release markers. | implemented_pending_validation |
| Unstable capability index/projection | Reuse indexed capability admission and project only required entries. | Hash/index parity and managed P50/P95 evidence. | implemented_pending_validation |
| Unstable viewport-overlay capability sort | Apply the same non-observable-order sort policy to viewport overlays. | Overlay capability contract and managed Release evidence. | implemented_pending_validation |
| Borrowed extension-view validation | Borrow `ViewDescriptorId` keys while validating a candidate batch. | Detailed evidence is recorded in [Editor988](988-editor06-borrowed-extension-view-validation.md). | implemented_pending_validation |

The current grouped Runtime/Editor validation submission covers the Editor06
source paths together with the Runtime batches. No per-plan Cargo invocation is
started and no asynchronous result is inferred here.
