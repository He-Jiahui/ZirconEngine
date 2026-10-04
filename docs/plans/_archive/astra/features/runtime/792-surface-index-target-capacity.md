---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-17-surface-index-target-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/template/asset/surface_index.rs
tests:
  - zircon_runtime/src/ui/tests/asset_surface_index/target_capacity.rs
  - tools/tests/test_runtime_surface_index_target_capacity_performance_contract.py
---

# Runtime792 · Surface-index target projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime11A hot-reload target projection | Reserve the sum of the four surface/node category input lengths before borrowed-set deduplication, preserving category order, first-seen order, and duplicate semantics. | TDD RED→GREEN source contract `3/3`; lower surface/node order-capacity regressions; ignored `RUNTIME792_SURFACE_INDEX_TARGET_CAPACITY_BENCH_V1`; deterministic 16,384-target model `12→0`; managed Cargo/Release, allocator, and product percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`all_target_surfaces` and `all_target_nodes` now use saturating category-length
sums as local upper bounds before the existing `BTreeSet`-backed first-seen
projection. Duplicate targets still collapse identically, and the output order
continues to follow the four category vectors.

The lower module is wired under the existing asset-surface-index test root. Its
two non-ignored tests assert output cardinality and capacity; the ignored marker
alternates legacy and bounded collectors over four 4,096-item categories.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/surface_index.rs` | `B163046DD82B7E1BF3033EFAE24A2FC5B7F8C334AFEABC33507F43E49FE33879` |
| `zircon_runtime/src/ui/tests/asset_surface_index.rs` | `B756B98FA27D44E0CA58187544A2549EECDF8237B27D11CF153C96A2B11B7100` |
| `zircon_runtime/src/ui/tests/asset_surface_index/target_capacity.rs` | `F8924F3E53EC3B70368ACFB465ACD18C875B079E75D79C84CD520EF46BF1B14D` |
| `tools/tests/test_runtime_surface_index_target_capacity_performance_contract.py` | `E45B393ACFA97B2E6B3A58168D20A479FD3AC2EACF60058685136A57C4722CEB` |

The focused source contract passes `3/3`; exact-file Rustfmt, scoped diff
checks, and Wiki validation are green. The current explicit-file Runtime/Editor
performance-contract loader then passes `1981/1981` tests across `554` files in
`30.200s`, with zero failures, errors, or skips. These are local source/model
receipts only. The subsequent broad non-tooling Runtime/Editor regression batch
passes `3731/3731` across `918` modules in `528.898s`, also with zero failures,
errors, or skips.

## 性能与受管验证边界

The deterministic model removes 12 modeled vector-growth events for the
16,384-entry fixture (`12→0`). Local source contracts, lower Rust formatting,
and the subsequent combined Runtime/Editor batch are the required local gates;
they do not establish Rust Cargo compilation, Windows Release allocation counts,
or product hot-reload p50/p95/p99. Keep the status
`implemented_pending_validation` until the asynchronous managed batch supplies
those gates. This session does not query or poll the coordinator, and tooling
remains intentionally deferred.
