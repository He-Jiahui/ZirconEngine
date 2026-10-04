---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-14-route-terminal-state-bit.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/749-runtime-primary-pointer-source-counter.md
implementation_files:
  - zircon_runtime/src/ui/surface/input/route_steps.rs
tests:
  - zircon_runtime/src/ui/surface/input/route_steps_tests.rs
  - tools/tests/test_runtime_ui_route_terminal_scan_performance_contract.py
---

# Runtime750 · Route terminal state bit

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime200 / route-step projection | Carry the ancestor-stop state into the out-of-route terminal append guard instead of scanning all accumulated steps. | implemented_pending_validation | TDD RED/GREEN route contract `3/3`; latest combined Runtime/Editor pointer/reference/history/input batch `76/76`; current Editor `622/622` and Runtime `1201/1201` performance-contract batches; Rust regression and ignored `RUNTIME200_ROUTE_TERMINAL_SCAN_BENCH_V1` marker; scoped Rustfmt, Python compilation, and diff checks pass. Managed Cargo/Release and product p50/p95/p99 evidence remain pending. |

## 性能边界

The route builder's preview and target branches return on a stop, while the
ancestor branch only breaks after recording its terminal step. The state bit
therefore replaces an `H`-row scan with one branch for a path of depth `H`; no
route node, step ordering, or public diagnostic field changes.

## 源码指纹

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/route_steps.rs` | `A76F6137B4AEF851A84CA94DF5D3247782EADC97D2632E5CFE95B83FFCB5B409` |
| `zircon_runtime/src/ui/surface/input/route_steps_tests.rs` | `3387D739327055D90D5213F539C246EB1FE66E52A9B800C049F4002EA3790389` |
| `tools/tests/test_runtime_ui_route_terminal_scan_performance_contract.py` | `AF72D6F5D4ADA614C84515894ED6449F65CF8E5C3B5773743308B2102AF1AA41` |

## 受管验证

This slice joins the existing batched Runtime/Editor Windows Release lane. Keep
the status at `implemented_pending_validation` until the owner-attributed gate
proves compile, route semantics, allocation behavior, and input latency. No
standalone Cargo invocation or coordinator polling was performed; tooling
production work remains deferred.
