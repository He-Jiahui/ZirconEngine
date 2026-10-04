---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-14-route-path-borrow.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/750-route-terminal-state-bit.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/route_policy.rs
tests:
  - zircon_runtime/src/ui/surface/input/route_policy_tests.rs
  - tools/tests/test_runtime_ui_route_path_borrow_performance_contract.py
---

# Runtime751 · Generic route preview path borrow

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime200 / generic route diagnostics | Select the bubble/focus preview source by borrow instead of cloning an intermediate route vector. | implemented_pending_validation | TDD RED/GREEN source/behavior contract `3/3`; latest focused Runtime/Editor batch `76/76`; current Runtime/Editor performance-contract discoveries `1201/1201` and `622/622`; Rust route-precedence regression and ignored `RUNTIME200_ROUTE_PATH_BORROW_BENCH_V1` marker; scoped Rustfmt, Python compilation, and diff checks pass. Managed Cargo/Release and product p50/p95/p99 evidence remain pending. |

## 性能边界

The public trace still owns `bubble_path`, `focus_path`, and the reversed
`preview_tunnel`. Only the temporary selection clone is removed; bubble
precedence and empty-bubble fallback remain identical.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/route_policy.rs` | `1533D38FE2E7E4C7FE4E18255D20BD8EBCAAC315AD04010EF72417CB7FDAA22F` |
| `zircon_runtime/src/ui/surface/input/route_policy_tests.rs` | `4650255E9CED7F61372B346D375AB107F179AC7431F2AADA7861653C496EB172` |
| `tools/tests/test_runtime_ui_route_path_borrow_performance_contract.py` | `7165F2DF96805654A370860298F8B948037B9E3AD027B2AFC4EED3A8D1663E56` |

## 受管验证

This slice remains `implemented_pending_validation` until the shared
owner-attributed Windows Release lane proves compilation, allocation behavior,
and input latency. No standalone Cargo run or coordinator polling was used;
tooling production work is deferred.
