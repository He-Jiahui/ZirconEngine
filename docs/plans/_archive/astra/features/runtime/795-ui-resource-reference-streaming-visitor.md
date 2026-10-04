---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/74/2026-08-26-ui-resource-reference-streaming-visitor.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/ui/resource_references.rs
tests:
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - tools/tests/test_runtime_ui_resource_reference_visitor_performance_contract.py
---

# Runtime795 · UI resource-reference streaming visitor

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime74 UI asset references | Stream borrowed `res://`, `asset://`, and `project://` URIs directly into the existing normalizer/deduplicator instead of materializing a temporary `Vec<&str>`, preserving traversal order and first-seen semantics. | TDD source contract `3/3`; lower order/ownership regressions; ignored `RUNTIME74_UI_RESOURCE_REFERENCE_VISITOR_BENCH_V1` now uses 101 alternating pairs (`51` legacy-first, `50` optimized-first); deterministic 4,096-URI model removes the temporary pointer vector; managed Cargo/Release and product percentile evidence remain pending. | implemented_pending_validation |

## Implementation evidence

`ui_asset_references` now passes one borrowed callback through imports, tokens,
nodes, components, styles, arrays, and tables. The normalizer and `HashSet`
remain the ownership and duplicate authorities; only the intermediate URI
collection was removed.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/assets/ui.rs` | `1169780C39D785A95D4227CCD0ADD5A1EDC6C606DF324DE9AD8D0A39DC5DEBAC` |
| `zircon_runtime/src/asset/assets/ui/resource_references.rs` | `2E0771E5EDA713125D8F26673896F7465089F1BEDB6C37B0230B28778A908AC6` |
| `zircon_runtime/src/asset/tests/assets/ui.rs` | `3DD1076AA450B94F45CB62C6E1FAACC92AD8DD4650A623926261C02F78FFEC13` |
| `tools/tests/test_runtime_ui_resource_reference_visitor_performance_contract.py` | `4491E647D5C868F6407F8E81F2D203291FF8CF3B2CF67F44AE294F19ABBE83A8` |

The exact five-file Rustfmt set passes, and the new source contract is included
in the current combined Runtime/Editor batch (`2002/2002` across `561` files in
`12.503s`). Local receipts do not establish managed Cargo compilation, Windows
Release allocation counts, or product asset-reference p50/p95/p99.

## 性能与受管验证边界

The deterministic model removes one temporary URI pointer vector for the
4,096-URI fixture while retaining zero URI string clones. The benchmark now samples 101 pairs,
with the marker exposing the 51/50 alternating first-run split. Keep this record
`implemented_pending_validation` until the asynchronous managed batch supplies
the lower Rust and product gates. Tooling production code remains deferred.
