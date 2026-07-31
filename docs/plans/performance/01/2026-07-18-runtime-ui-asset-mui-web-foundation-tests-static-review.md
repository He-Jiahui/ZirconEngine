---
related_code:
  - zircon_runtime/src/ui/tests/asset_mui_web_badge_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_collection_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_form_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_form_style/form_controls.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_lab_style.rs
  - zircon_runtime/src/ui/tests/asset_mui_web_layout_style.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - five MUI Web style semantic tests reviewed across six tracked Rust files
  - current-source Cargo, selector scale counters and F4 product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI MUI Web基础样式测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读badge、collection、form/form-controls、lab与layout样式测试共6/6个tracked Rust文件，合计2,344行/5测试。范围覆盖badge/avatars/chips/lists/tables、inputs/select/radio/checkbox/switch、timeline/tree/loading按钮，以及container/grid/stack/masonry/collapse/media-query语义。

## PERF-MVP-307：语义广度不等于selector规模证据

各测试均解析一份template与一份stylesheet并只编译一次，能锁定大量class/property语义，但fixture仍是小树。断言通过递归`find_node`反复扫描测试树，属于测试代码O(Q×N)，不会进入产品帧；当前没有记录rule-per-node selector probes、parsed-selector复用、node/class visits、compile allocations或冷/热编译p95，因此不能证明PERF-MVP-307的10k-node预算。

## PERF-MVP-275/276/315：MUI生成与动态失效仍缺产品证据

这些fixture证明Web样式属性可被typed/compiled结果观察，但没有覆盖MUI构建器生成成本、稳定generation下的重复property/class转换、viewport resize触发频率、collapse transition逐帧dirty域或layout/paint stage counter。相关工作继续回链PERF-MVP-275/276；transition与响应式动态失效回链PERF-MVP-315。本切片未发现可在不改变测试可读性与产品契约前提下直接修复的新独立热路径。

## 验收要求

补充1/100/10k nodes与rules、1/10/100 classes-per-node，以及连续viewport resize/collapse transition场景，记录selector parses/probes、tree visits、property conversions、allocation bytes、dirty domains与compile/layout/paint p95；再运行当前源码Cargo和F4编辑器产品trace。完成前这6文件仅标记静态读完，留在`pending.md`。
