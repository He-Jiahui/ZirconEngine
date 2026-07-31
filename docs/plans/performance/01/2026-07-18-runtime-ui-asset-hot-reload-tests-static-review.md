---
related_code:
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/08-editor-ui-performance-and-incremental-rendering.md
tests:
  - three executor and ten plan semantic tests reviewed
  - one source plus behavior RED to GREEN classifier allocation guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, watch/resource/surface scale counters and F4 trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset hot reload executor/plan测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`asset_hot_reload_executor.rs` 323行/3测试与`asset_hot_reload_plan.rs`原始361行/10测试，共2/2个tracked Rust文件；plan测试加入1项源码+行为守卫后为373行/11测试。范围覆盖template/theme/icon/font/texture分类、cache eviction、resolver invalidation、surface dirty/rebuild及execution report parity。

## PERF-MVP-309：watch URI分类无条件分配

`classify_ui_hot_reload_asset`原先对每个change先strip fragment/trim，再无条件`to_ascii_lowercase()`。产品AssetUri通常已经canonical lowercase，因此watch burst为每个URI产生一次无收益String。新增守卫先确认RED，再让normalizer返回`Cow<str>`：无ASCII大写时借用slice，仅兼容大写输入时分配；同时新增大写+fragment行为断言，守卫转GREEN。第51组局部优化不改变suffix优先级或分类结果。

## 仍开放的执行放大

`from_watch_report`复制changed/removed及各分类target，execution report再次复制四组plan vectors；resolver invalidation仍对每个URI清理cache，surface apply依赖asset→surface但缺asset→node ownership，template/theme变更因此标记root聚合dirty。3个executor fixture最多3 changes/1 surface，10个plan fixture最多2 edges，未量化列表字节、cache scans、roots/nodes dirtied或阶段重建。继续回链PERF-MVP-309和EditorUI05/08。

## 验收要求

对1/100/1k changes、1/100/10k targets/cache entries/surfaces/nodes记录classifier alloc、plan/report clone bytes、resolver scans、roots/nodes dirtied、stage visits和reload p95。canonical lowercase URI classifier allocation=0；stable resource change只触发拥有该asset的nodes/surfaces与必要dirty domains。当前源码executor 3项/plan 11项Cargo、规模counter及F4 asset edit/hot-reload产品trace完成前，两文件留在`pending.md`。
