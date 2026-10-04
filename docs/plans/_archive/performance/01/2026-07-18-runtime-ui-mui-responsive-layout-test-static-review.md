---
related_code:
  - zircon_runtime/src/ui/tests/mui_responsive_layout.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/v2
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - one 295-line semantic matrix reviewed across 599/800/960 viewports
  - Grid Stack Masonry visibility and media-query parity present
  - responsive node/slot/pass visit counters and stable-viewport test pending
  - current-source Cargo and product resize trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI MUI responsive layout测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/mui_responsive_layout.rs` 1/1个tracked Rust文件、295行、1个大型测试。fixture从TOML load/compile/build surface，在599、960、800三个viewport运行layout，验证Grid/Stack/Masonry breakpoint、explicit layout优先级、visibility/visible/display以及min/max/range/up/down/between media query语义。

## PERF-MVP-259：功能矩阵没有stage成本

测试适合锁定responsive重构后的语义，但只有约15个nodes，没有记录responsive descriptor/node/slot visits、query parse、dirty-root discovery、arranged/hit rebuild或allocation。生产`responsive_mui`仍在增量root发现前执行多轮全tree/slot扫描，stable viewport也支付这些工作；现有`layout_visited_node_count`不含它们。验收必须分别暴露responsive/root/measure/arrange/arranged/hit visits，不能用最终layout visit掩盖前后全量stage。

## PERF-MVP-276/313：编译与测试规模缺口

单测每次从大段TOML重新load/compile可接受，但没有证明产品compiled generation复用typed responsive descriptor。helper `node_by_control_id`为每个断言线性扫描全部nodes，`grid_item_slot`也线性扫描slots；这属于测试侧小fixture成本，不能被误当作产品索引证据。EditorUI02/04/05应共享viewport/style/tree generation与compiled query/index，稳定viewport evaluation为0。

## 验收要求

保留当前三viewport语义矩阵，并增加100/1k/10k nodes、responsive nodes 0/1/100%、stable 300 frames及单次breakpoint crossing，记录query parse/evaluate、node/slot/pass visits、dirty nodes、allocation与CPU p95。current-source Cargo、MVP workbench连续resize trace和像素对拍完成前，本文件留在`pending.md`，不进入`review.md`。
