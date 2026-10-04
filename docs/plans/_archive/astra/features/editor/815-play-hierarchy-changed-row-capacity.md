---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/247-editor-scene-world-authoring-play-hierarchy-document-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/07/2026-09-19-play-hierarchy-changed-row-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/editor/813-animation-timeline-projection-capacity.md
  - docs/plans/astra/features/editor/814-animation-curve-projection-capacity.md
implementation_files:
  - zircon_editor/src/ui/host/play_hierarchy_projection.rs
tests:
  - tools/tests/test_editor_play_hierarchy_changed_row_capacity_performance_contract.py
---

# Editor815 · Play hierarchy changed-row capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor07 same-topology Play hierarchy refresh | Reserve the current-row bound and changed-row anchor bound before ordered append, preserving sparse row order, generation/selection deltas, no-op behavior, and inspection-message semantics. | TDD source/model contract `4/4`; lower Rust sparse-row/anchor semantic regression and ignored `EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1` marker are wired; the combined Runtime/Editor focused batch passes `65/65`; strict non-tooling performance/pressure batch passes `2638/2638` across `684` files in `30.325s`; deterministic `4,096`-row model changes changed-row collector growth `11→0`. Managed Windows/Cargo/Release and Play hierarchy product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the allocation and iterator shape of the same-topology
Play hierarchy delta projection. It does not change world querying, hierarchy
identity, generation fencing, selection authority, reparenting, inspection
semantics, or tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/play_hierarchy_projection.rs` | `4DAFAD96DED7D36491854EF2512D567329E7CAB33002318508568C44B0847271` |
| `tools/tests/test_editor_play_hierarchy_changed_row_capacity_performance_contract.py` | `2CE537DD6D3D165D227460ED34611684ED1CA43E1018299B0B1F76BFD4D5E99F` |

## Managed gate

No Cargo process is started locally and the external coordinator is not
polled. The local source/model evidence is folded into the existing combined
Runtime/Editor validation handoff; tooling production remains deferred for the
later Rust migration.
