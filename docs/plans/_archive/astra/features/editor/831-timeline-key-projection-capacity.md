---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75/2026-09-19-key-projection-capacity.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/808-keyframe-lane-capacity.md
  - docs/plans/astra/features/editor/809-track-list-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/timeline_strip/generation.rs
  - zircon_editor/src/ui/timeline_strip/tests.rs
tests:
  - tools/tests/test_editor_timeline_key_projection_capacity_performance_contract.py
---

# Editor831 · timeline key projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor75 timeline key normalization | Reserve `input.keys.len()` before finite-key filtering and duration clamping, preserving source order, labels, selection, and generation semantics. | TDD source contract RED/GREEN `3/3`; existing Timeline generation contract plus the new contract pass `8/8`; lower semantic regression and ignored `EDITOR75_TIMELINE_KEY_PROJECTION_CAPACITY_BENCH_V1` marker are wired; modeled 4,096-key growth events `11→0`. | implemented_pending_validation |

## Complexity boundary

This slice changes only the owned key-vector allocation shape. It does not
change filtering, clamping, key identity, selection, static/dynamic
generation, timeline cache ownership, document state, or virtualization.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline_strip/generation.rs` | `ED7A994A1CF020224834D112B5E28EE8A9B0A976B32F47F8090B15980BC3CD67` |
| `zircon_editor/src/ui/timeline_strip/tests.rs` | `F2E6BCBBF146D259BB0F1EFC1C02D77BB14AC9A0E230A551C1E45D20FB831E85` |
| `tools/tests/test_editor_timeline_key_projection_capacity_performance_contract.py` | `01C3EA860E06FCD0AEDC77A3214A85B66EC0484F73098460F6ACB4622BDA5045` |

## Managed gate

No Cargo process is started locally and the coordinator is not polled. Keep
this entry `implemented_pending_validation` until the owner-attributed batched
Windows Release lane proves current-source compilation, key parity, allocation
behavior, and timeline product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration.
