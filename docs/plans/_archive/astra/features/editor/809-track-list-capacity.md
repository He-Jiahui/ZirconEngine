---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75/2026-09-19-track-list-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/808-keyframe-lane-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/timeline/track_list.rs
tests:
  - tools/tests/test_editor_track_list_capacity_performance_contract.py
---

# Editor809 · timeline track-list projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor75 track-list projection | Reserve the exact input track bound before ordered row materialization while preserving lane classification and cloned row fields. | TDD source/model contract `4/4`; lower source regression and ignored `EDITOR809_TRACK_LIST_CAPACITY_BENCH_V1` marker are wired; deterministic 4,096-track model removes geometric growth events; merged current-worktree non-tooling batch passes `3689/3689` across `876` files in `57.003s`. Managed Windows/Cargo/Release and timeline product percentile evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only the output vector's allocation shape. It does not alter
track order, lane classification, key/section counts, row identity, timeline
selection, document state, virtualization, or the parent Editor75 authority.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline/track_list.rs` | `733604EF81F880133DA37C893D15373983E049922B982C51FDB0834D343FF4E4` |
| `tools/tests/test_editor_track_list_capacity_performance_contract.py` | `7F03786C38F3C586A374AA0F0AAD1913870AC498C6F30FB5A419E4030ED2ED1B` |

## Managed gate

No Cargo process is started locally and the external coordinator is not
polled. Local source/model evidence is retained for the merged validation
batch; tooling production remains deferred for the later Rust migration.
