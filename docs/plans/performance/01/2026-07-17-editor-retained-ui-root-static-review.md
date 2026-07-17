---
related_code:
  - zircon_editor/src/ui/retained_host/ui/component_contract_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/reference_component_tests.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/ui/root_template_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/structure_component_tests.rs
  - zircon_editor/src/ui/retained_host/ui/template_layout_context.rs
  - zircon_editor/src/ui/layouts/views/preview_images.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05
  - docs/plans/zircon_editor/editor_ui/08
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - preview_cache_hit_accepts_borrowed_key_components
  - preview_loader_reuses_cached_svg_icons
  - existing root overlay, component contract and table layout tests
  - current-source Windows zircon_editor focused Cargo pending
  - 1k preview cache-hit allocation/decode trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained host UI root 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/ui` 除已单列的 presentation、Workbench projection、template conversion、tests与 `pane_data_conversion` 外，当前有 **6** 个根级 Rust 文件、**794** 行，已逐文件阅读 **6/6**。本批同时沿 root overlay与Workbench/pane调用链完整阅读共享的 `ui/layouts/views/preview_images.rs`。

component descriptor使用静态 retained registry；fallback分类为纯 match。root overlay只过滤显式标记节点，preview decode已有全局缓存。table context只处理 table节点，现有 token去重保持正确。三个 reference/structure文件是源码/集成契约测试，不进入产品热路径。

## 已直接优化

PERF-MVP-137：preview cache旧 key在每次调用入口立即复制 `source` 与 `icon_name`，所以缓存命中仍有两个 String分配，并在 `BTreeMap`中做有序比较。缓存现为两级 hash map，命中以借用字符串查询，owned key只在 miss插入。SVG parse/raster仍在锁外执行，随后原子插入；同一 source下不同 icon、同一 icon下不同 source继续独立。

源码借用键测试与既有 cache reuse测试已补齐，`rustfmt`通过。协调式 Cargo与1k hit allocation/decode计数未完成，因此本批仍在 `pending.md`。

## 已有结构性热点

root overlay和Workbench projection在 presentation generation变化时仍逐节点构建 host DTO；这属于 PERF-MVP-135 的 typed property/node generation与 changed-node patch，而不是再建一个 overlay私有缓存。preview cache的全局 mutex也保留；只有多线程 projection trace证明争用后，才考虑 sharded/read-optimized cache，避免在缺乏证据时扩大同步语义。
