---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/73/2026-09-20-runtime-selector-path-single-buffer.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/selector_path_tests.rs
tests:
  - tools/tests/test_runtime_selector_path_single_buffer_performance_contract.py
---

# Runtime854 · runtime selector path single-buffer

## 完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime V2 selector path | Build the retained-tree selector path directly into one reversible `Vec<SelectorPathNode>`, removing the intermediate ancestor-ID buffer while preserving root-first order and `:host` matching. | TDD source/model contract `4/4` after a RED run with three structural failures; lower Rust order/host regression and ignored `RUNTIME854_RUNTIME_SELECTOR_PATH_SINGLE_BUFFER_BENCH_V1` marker are wired. The deterministic depth-128 model changes one intermediate buffer per build to zero. The current expanded source-contract loader passes `4072/4072` across `962` files in `139.499s` (performance-or-contract filename filter, tooling/export/coordinator excluded); managed Cargo/Release, allocator, and product selector-style p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only selector-path construction in the Runtime V2 style
hot path. It does not alter selector parsing, rule indexing, pseudo-state
collection, tree mutation, or tooling production. The shared `style.rs` file
also carries adjacent Runtime785 and Runtime853 changes; those are tracked by
their own records.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/v2/style.rs` | `C7A9BDABD7E0DA0CC98595662DFD7858CEDB2B6827C323D1339B20174A145DCE` |
| `zircon_runtime/src/ui/v2/style/selector_path_tests.rs` | `082C6C96269932C99B33EAAAAC69510708606ED19A15D5255C7FDF5B4203A7E5` |
| `tools/tests/test_runtime_selector_path_single_buffer_performance_contract.py` | `8CEBCE81653879B4478D06E4DD6245219BDEFDEA84EDCE1A6BFD6D1253B7D7B3` |

## Managed gate

No managed Windows Cargo/Release validation command is started locally and
coordinator status is not polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation,
lower-test reachability, allocator behavior, and selector-style product
p50/p95/p99 evidence.
