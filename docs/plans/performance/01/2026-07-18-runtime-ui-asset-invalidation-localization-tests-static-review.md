---
related_code:
  - zircon_runtime/src/ui/tests/asset_invalidation.rs
  - zircon_runtime/src/ui/tests/asset_localization.rs
  - zircon_runtime/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/localization/collect.rs
  - zircon_runtime/src/ui/template/asset/localization/resolve.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - six invalidation and seven localization semantic tests reviewed
  - two source-level RED to GREEN traversal/allocation guards added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, document/catalog scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset invalidation/localization测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_invalidation.rs`原始215行/6测试与`asset_localization.rs`原始219行/7测试，共2/2个tracked Rust文件；分别加入1项源码性能守卫后为233行/7测试和228行/8测试。范围覆盖change→stage/dirty、broad/large/non-virtualized diagnostics、localization dependency/extraction/catalog/manifest与dotted keys。

## PERF-MVP-308：diagnostics重复node traversal

`collect_invalidation_diagnostics`原先先`iter_nodes().count()`，随后再次遍历全部nodes检查non-virtualized ScrollableBox。新增守卫先确认RED，再把count与scroll pressure合入一次tree visit；large warning最后插入首位，保持原`large → broad selector → scroll`输出顺序。第52组局部优化删除一次O(N)遍历；selector parse与cache fingerprint全序列化仍归PERF-MVP-307/308。

## PERF-MVP-311：localization catalog稳定查询分配

catalog原用`BTreeMap<(String,String), Entry>`，每个dependency lookup为locale/table各构造一个String。新增守卫先确认RED，再改为`BTreeMap<locale, BTreeMap<table, Entry>>`，查询直接用`&str`，守卫转GREEN；注册所有权和missing table/key诊断不变。第53组局部优化使稳定catalog查询key allocation=0。collector逐Value格式化完整path、多次tree pass与最终全排序仍归PERF-MVP-311/EditorUI05。

## 验收要求

对1/100/10k nodes/rules/values/dependencies和1/10/100 locales/tables记录tree/value visits、selector parses、path bytes、catalog key alloc、sort comparisons、report bytes和compile p95。diagnostics node visits=N；catalog lookup key allocation=0；single generation localization tree walk/index build有明确上限。当前源码invalidation 7项/localization 8项Cargo、规模counter与F4 localized asset compile/hot-reload trace完成前，两文件留在`pending.md`。
