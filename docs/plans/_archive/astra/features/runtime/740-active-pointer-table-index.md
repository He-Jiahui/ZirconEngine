---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-13-active-pointer-index.md
related_code:
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table/index_tests.rs
tests:
  - tools/tests/test_runtime_ui_active_pointer_index_performance_contract.py
  - zircon_runtime/src/ui/tests/runtime_input_manager
---

# Runtime Active Pointer Table Index

Runtime 的 active pointer table 现在保留有序 `Vec` 作为公开遍历视图，
并以私有 `HashMap<UiPointerId, usize>` 支持 exact-key 查找。upsert、
remove、clear 会同步维护索引；中间删除仍使用稳定顺序的 `Vec::remove`，
不改变 hover、pressed、capture 或 primary 状态语义。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime200 / pointer input hot path | Active pointer lookup uses an indexed key map while ordered entries remain stable | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; the comprehensive non-tooling Runtime/Editor performance/pressure batch covers 548 files and passes `2039/2039` in `22.986s`; scoped py_compile, rustfmt, and diff checks pass; managed Cargo, Release allocation, and pointer-input p50/p95/p99 evidence remain pending. |

## 性能边界

`P` 个 active pointers 时，entry/entry_mut/upsert 的查找从 `O(P)` 降为
expected `O(1)`；为保留有序 slice，middle remove 仍为 `O(P)`。该记录只
说明局部结构性优化，不把静态合同当作产品延迟达标证据。

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo；当前仍为
`implemented_pending_validation`。
