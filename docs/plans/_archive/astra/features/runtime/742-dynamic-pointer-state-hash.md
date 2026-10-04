---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/200/2026-09-13-dynamic-pointer-state-hash.md
related_code:
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui/tests.rs
tests:
  - tools/tests/test_runtime_dynamic_pointer_state_hash_performance_contract.py
---

# Runtime Dynamic Pointer State Hash Lookup

跨 surface 的 pointer capture 与 last-position 表已改为私有
`HashMap<Option<u64>, _>`。这些表只做 exact-key get/insert/remove，不依赖
顺序；`None` 鼠标键、Down/Up/Cancel 清理和 fallback routing 语义保持不变。

## 计划完成列表

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime200 / dynamic pointer routing | Capture and position state use exact-key hash tables | implemented_pending_validation | TDD RED/GREEN source contract `3/3`; adjacent pointer/input contracts `23/23`; the comprehensive non-tooling Runtime/Editor performance/pressure batch covers 548 files and passes `2039/2039` in `22.986s`; managed pointer-input p50/p95/p99 evidence remains pending. |

## 性能边界

active pointer 数量为 `P` 时，capture/position 查询由 `O(log P)` 降为
expected `O(1)`；这不改变尚未完成的 multi-seat/window/generation authority。

## 受管验证

该切片加入现有 Runtime/Editor 合批，不单独启动 Cargo；当前仍为
`implemented_pending_validation`。
