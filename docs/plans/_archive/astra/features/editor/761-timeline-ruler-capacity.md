---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75/2026-09-14-ruler-capacity.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/timeline/ruler.rs
  - zircon_editor/src/ui/timeline/tests.rs
tests:
  - zircon_editor/src/ui/timeline/tests.rs
  - tools/tests/test_editor_timeline_ruler_capacity_performance_contract.py
---

# Editor761 · Timeline ruler upper-bound capacity

Timeline ruler generation now reserves the derived interval bound plus two
possible endpoint slots before its non-zero-duration loop. Nice-step selection,
hard capping, endpoint insertion, labels, ordering, and the zero-duration path
remain unchanged. This is a bounded allocation improvement for PERF-MVP-215;
the broader timeline cache/LOD and shared-generation goals remain open.

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor75 / timeline ruler | Reserve the finite interval estimate (capped at 4,096) plus endpoint slots before tick materialization. | TDD RED/GREEN source contract `4/4`; lower capacity/endpoint/order regression; ignored `EDITOR761_TIMELINE_RULER_CAPACITY_BENCH_V1` marker; focused capacity batch `21/21` in `0.127s`; merged Runtime/Editor performance contracts `1844/1844` across `515` modules in `47.661s`; scoped Rustfmt/diff. | implemented_pending_validation |

## 性能边界

The tick loop remains bounded by `MAX_RULER_TICKS` and linear in the generated
ticks. The deterministic growth model is allocation-shape evidence only; managed
Cargo/Release and product CPU/RSS/p50/p95/p99 evidence remain required.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline/ruler.rs` | `84BE8F51B569A57334A64C4B7219DE9E76A686C31E4E440E9A68407AF1D78B2E` |
| `zircon_editor/src/ui/timeline/tests.rs` | `29CE143FE85D1F2E63D76D37590CED8D10C6217726380C674978B832C0CDD66A` |
| `tools/tests/test_editor_timeline_ruler_capacity_performance_contract.py` | `EE0F65576D90CC1527F9E0ED5A77BDC509A232DF2C509DA29B31E3FB74672172` |

## 受管验证

This feature joins the existing multi-task Runtime/Editor Windows Release lane.
No per-task Cargo run or coordinator status query was made. Keep the record
`implemented_pending_validation` until current-source compile, behavior parity,
allocation, and percentile evidence arrive. Tooling production changes remain
deferred.
