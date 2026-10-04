---
title: Editor07 Play hierarchy changed-row capacity
category: zircon_editor
report_id: Editor815-play-hierarchy-changed-row-capacity-2026-09-19
date: 2026-09-19
related_to:
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/247-editor-scene-world-authoring-play-hierarchy-document-current-working-tree-review.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor815 · Play hierarchy changed-row capacity

## Scope

The same-topology Play hierarchy refresh compared old and current rows with a
lazy `filter_map(...).collect()` and then rebuilt hierarchy anchors with a
second iterator collector. The row input bound is known, and the changed-row
count is an upper bound for the anchor list. This slice makes both output
bounds explicit while preserving sparse order, generation/selection handling,
and the no-op fast path.

## Implementation

- Reserve the current row bound before ordered changed-row cloning.
- Preserve the existing zip boundary and only clone rows whose value changed.
- Reserve the changed-row bound before ordered hierarchy-anchor projection.
- Keep the generation, selection-revision, empty-delta, and message-field
  semantics unchanged.
- Add the ignored
  `EDITOR815_PLAY_HIERARCHY_CHANGED_ROW_CAPACITY_BENCH_V1` Release marker.

## Regression and performance contract

The lower Rust regression covers sparse changed-row ordering and the emitted
anchor entity/parent/depth/hash, while the `optimization_batch_tests` module
retains the empty/no-op shape and emits the ignored Release marker. The
deterministic `4,096`-row model
changes the legacy geometric growth count from `11` to `0` for the changed-row
collector; the anchor collector is likewise explicitly bounded by the number
of changed rows. This is allocation-shape evidence only, not a product
hierarchy p50/p95/p99 claim.

The Python source/model contract is
`tools/tests/test_editor_play_hierarchy_changed_row_capacity_performance_contract.py`.

## Local receipt

- The focused source/model contract passes `4/4`.
- The combined Runtime/Editor focused batch (Runtime804/807/808/809 and
  Editor805-815) passes `65/65` in one process with zero failures, errors, or
  skips.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- The strict non-tooling performance/pressure batch loads `684` files and
  passes `2638/2638` tests in `30.325s`, with zero failures, errors, or skips.
- Managed Cargo/Release and Play hierarchy allocation/product percentile
  evidence remain pending; tooling production remains deferred for the later
  Rust migration.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/host/play_hierarchy_projection.rs` | `4DAFAD96DED7D36491854EF2512D567329E7CAB33002318508568C44B0847271` |
| `tools/tests/test_editor_play_hierarchy_changed_row_capacity_performance_contract.py` | `2CE537DD6D3D165D227460ED34611684ED1CA43E1018299B0B1F76BFD4D5E99F` |

## Validation boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies Play hierarchy
allocation and product p50/p95/p99 evidence. No coordinator status is polled by
this session.
