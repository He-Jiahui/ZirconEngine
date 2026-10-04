---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11a/2026-09-13-reflection-node-index-hash-lookup.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/737-node-pool-owned-key-lookup.md
related_code:
  - zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
tests:
  - zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs
  - tools/tests/test_runtime_ui_reflection_node_index_hash_lookup_performance_contract.py
---

# Runtime UI reflection node-index hash lookup

`UiEventManager` 的 `node_index` 只承担 `UiNodePath` 命中查询，不承担排序或遍历。它现在
使用可复用容量的 `HashMap`，而 `trees` 与节点的有序遍历仍保留，因此重复路径的既有
优先级不变。重建时只为缺少的容量调用 `reserve`，避免热重建反复扩容。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
| --- | --- | --- | --- |
| Runtime11A P1-23 adjacent | 将反射节点路径索引从有序树查找切为容量复用的哈希查找，保留重复路径 winner 与公共查询语义。 | `implemented_pending_validation` | TDD RED/GREEN lower regressions cover `capacity`, lookup, and duplicate-path precedence; focused Runtime UI node-pool/reflection/layout batch `30/30` passes, and the direct 91-module Runtime UI batch passes `466/466` in `15.490s`; manager/reflection/test Rustfmt and scoped diff checks pass. The preceding Runtime737 broad Runtime/Editor static baseline is `548` modules / `2057/2057`; managed Release allocation/query-latency evidence is pending. |

## Acceptance boundary

This slice only optimizes the reflection index lookup and rebuild allocation. It does not close
the larger Runtime11A reflection-store authority, live-surface transaction, subscription, or
managed Cargo/product gates. Keep the record pending until the owner-attributed Windows batch
reports compile/test plus p50/p95/p99 and allocation evidence. Tooling production work remains
deferred by request.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs` | `BC047C7BCBBFADD1185276A1B93F3CCB7C86784D4BA144BE04CC0E4DB9B70B2E` |
| `zircon_runtime/src/ui/event_ui/manager/reflection_store.rs` | `1C666926BAE88B98E1BF6DBFA9D8433F45DEB268C78D204AB281CCDD27764646` |
| `zircon_runtime/src/ui/event_ui/manager/reflection_store/optimization_tests.rs` | `4E2E6E53B8E32805E2B4AA2D6D47234156E1FB8CDFA1CE58A5CF8059D05FAA8F` |
| `tools/tests/test_runtime_ui_reflection_node_index_hash_lookup_performance_contract.py` | `032E1FB3290FC4DF18505E4B6AB1FA87E9E58521D451CF0A20A919B98268AAC5` |
