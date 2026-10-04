---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-08-24-diagnostic-history-ring-capacity.md
  - docs/plans/optimize/zircon_runtime/03/2026-08-24-ecs-diagnostic-batch-publish.md
  - docs/plans/optimize/zircon_runtime/03/2026-08-26-perfetto-borrowed-event-projection.md
  - docs/plans/optimize/zircon_runtime/03/2026-09-09-static-diagnostic-metadata-cardinality-fast-path.md
related_records:
  - docs/plans/astra/features/runtime/682-ecs-diagnostic-batch-publish.md
  - docs/plans/astra/features/runtime/683-perfetto-borrowed-event-projection.md
  - docs/plans/astra/features/runtime/684-diagnostic-history-ring-capacity.md
---

# Runtime Diagnostics Optimization Batch Completion List

This batch records current-source Runtime03 changes that reduce diagnostic lock
contention, transient Perfetto export allocation, and retained history spare
capacity without changing published diagnostic or trace semantics.

## Plan Completion List

| Records | Work | Status | Static evidence |
|---|---|---|---|
| 682 | ECS frame diagnostics acquire the store once for the whole fixed metric projection | implemented_pending_validation | Behavior/source guards are present; 58-series lock model is deterministic. |
| 683 | Perfetto events borrow snapshot text, use typed args, and reserve output capacity | implemented_pending_validation | JSON-shape and allocation-source guards are present. |
| 684 | Bounded diagnostic histories evict before append; static metadata avoids redundant work | implemented_pending_validation | History window and metadata compatibility regressions are present. |

## Batched verification

- Runtime + Editor performance-contract discovery: `1727/1727` passed in
  `4.095s`.
- Batch-specific optimization source guards: `11/11` passed.
- Scoped Rustfmt and `git diff --check` passed for the involved Runtime and
  Editor sources; Git emitted only line-ending notices.

## Managed gate

No local Cargo lane was launched. The prior Cargo admission attempts remain
blocked by the external `E:\\Git\\zr_vm` dirty-worktree gate, and the accepted
async ticket is text-only. Therefore these rows do not claim managed compile,
Windows allocation measurements, or Release p50/p95/p99 acceptance.
