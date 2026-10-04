---
related_code:
  - zircon_runtime/src/ui/tests/asset_component_reference_layout.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - five component-reference semantic tests reviewed
  - one two-path source-level RED to GREEN override-borrow guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo and instance/style scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset component reference layout测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/asset_component_reference_layout.rs` 1/1个tracked Rust文件；原始480行、5个语义测试，本轮加入1个双路径源码性能守卫后为497行、6个测试。范围覆盖reference instance layout/props/style override/binding保留及nested Material role token解析。

## PERF-MVP-306/307：inline override每节点深复制

普通style application与prototype无stylesheet快路原先都在每个有inline override的节点执行`node.style_overrides.clone()`，深复制整张TOML map后才合并到attributes。该复制与override项数线性增长，并在大量reference instances上逐节点放大；现有fixture每次只编译一个root，未记录clone bytes或instance scale。

本轮双文件源码守卫先确认RED，再让两个路径同时借用`node.attributes`和`node.style_overrides`两个不相交字段并直接合并，守卫转GREEN。样式优先级、MUI sx顺序和结果所有权不变；第49组局部优化回链PERF-MVP-306/307。完整compiled instance/style generation仍由EditorUI04/05负责。

## 验收要求

对1/100/10k reference instances、0/10/100 override keys和0/100/10k style rules记录override clone bytes、value visits、selector probes、compiled tree bytes及compile p95。两个inline路径的override clone必须为0，stable artifact复用时实例tree/style重建为0；layout/props/style/binding/token parity保持一致。当前源码6项Cargo、规模counter和F4多实例asset preview trace完成前，本文件留在`pending.md`。
