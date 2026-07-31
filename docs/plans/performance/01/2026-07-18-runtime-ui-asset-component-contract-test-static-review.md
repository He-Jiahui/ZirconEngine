---
related_code:
  - zircon_runtime/src/ui/tests/asset_component_contract.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
tests:
  - 15 component-contract semantic tests reviewed
  - one source-level RED to GREEN repeated-reference cache guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo and repeated-instance scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset component contract测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/asset_component_contract.rs` 1/1个tracked Rust文件；原始641行、15个语义测试，本轮加入1个源码性能守卫后为659行、16个测试。范围覆盖public parts、selector privacy、API version、root class、binding/focus target、imported stylesheet与structured diagnostic parity。

## PERF-MVP-311：重复reference重建静态contract/privacy index

原`validate_reference_privacy`对每个instance reference重新执行`validate_component_contract`，并为同一imported component重新遍历component tree、构建node/control/public/private BTreeSet。若R个实例引用同一含N节点的widget，静态contract/privacy部分由O(R×N)放大；现有fixture每种import通常只有1个instance，无法暴露该成本。

本轮新增源码守卫先确认RED，再在单次validation调用内按完整`component_ref`缓存已验证reference和`ComponentPrivacyIndex`，守卫转GREEN。每instance的required API version检查及selector diagnostic path仍逐实例执行，错误顺序与结果保持不变；静态tree/index工作降为每个唯一imported component一次。更大的selector targets×references扫描与跨validator共享index仍归PERF-MVP-311/EditorUI04/05。

## 验收要求

对1/100/10k instances、1/100 unique components、1/100/10k component nodes与selector targets记录contract tree visits、privacy index builds、BTreeSet inserts、selector probes、alloc bytes和compile p95。相同`component_ref`的contract scan/privacy build每次validation均<=1；不同instance API要求和首错diagnostic保持一致。当前源码16项Cargo、规模counter和F4多实例widget compile trace完成前，本文件留在`pending.md`。
