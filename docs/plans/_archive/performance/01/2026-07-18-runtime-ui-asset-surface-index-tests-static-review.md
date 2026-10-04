---
related_code:
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/asset_surface_index
  - zircon_runtime/src/ui/template/asset/surface_index.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/08-editor-ui-performance-and-incremental-rendering.md
tests:
  - twelve surface/node ownership and dirty-target semantic tests reviewed
  - one multi-condition source-level RED to GREEN aggregation guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, index/target scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset surface index测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_surface_index.rs`与`asset_surface_index/**`共4/4个tracked Rust文件；原始768行/12测试，root加入1项多条件源码性能守卫后为780行/13测试。范围覆盖surface/node双向asset edges、compiled/resource metadata registration、template/theme/resource targets、precise node dirty、missing target与root fallback parity。

## PERF-MVP-309/278：target dedupe双份owned clone与重复scan

`collect_*_for_assets`及`all_target_*`原先把每个命中`UiTreeId`/`UiAssetNodeTarget`同时clone进seen BTreeSet和output Vec；node dirty随后再过滤theme/resource vectors两次计算raw expected count。新增守卫先确认RED，再让seen借用index/target slice内对象，最终report只clone一次；`push_nodes_for_surface`首次扫描直接返回match count，删除两个`target_surface_count` pass，守卫转GREEN。第58/59组局部优化保持BTree顺序、dedup及mixed-target必须root fallback的契约。

仍开放：`record_tree_node_resources`每次先移除整surface所有node edges，再扫描全部tree metadata并递归格式化Value path；surface/node/asset forward+reverse maps持有多份String/target，template rebuild始终surface root级。generation delta、interned asset id与asset→node authority继续归PERF-MVP-309/278及EditorUI05/08。

## 验收要求

对1/100/10k surfaces/nodes/assets/targets及1/100/1k changed resources记录tree/value visits、edge removes/inserts、target clone bytes、seen allocations、duplicate filter visits、nodes/root dirtied和apply p95。target seen clone bytes=0；raw-count额外target scans=0；stable generation edge rebuild=0；精确resource change不回退root。当前源码13项Cargo、规模counter与F4 multi-pane resource hot-reload trace完成前，4文件留在`pending.md`。
