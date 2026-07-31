---
related_code:
  - zircon_runtime_interface/src/ui/v2
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/slint/internal/compiler/llr/lower_to_item_tree.rs
  - dev/slint/internal/compiler/passes/apply_default_properties_from_style.rs
tests:
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime/src/ui/tests/v2_asset/asset_loading.rs
  - current-source Windows v2 compile/style tests pending
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Runtime interface UI v2 clean subset 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/v2/**`当前 **4/7** 个clean Rust文件（`arena.rs`、`compiled.rs`、`graph.rs`、`style.rs`）、**169** 行已逐文件阅读；外部dirty `asset.rs`、`mod.rs`、`repeat.rs`未读取、未吸收。已反查runtime v2 compiler/file cache/surface builder及测试。本轮未修改Rust源码。

## 性能结论

- `UiV2NodeHandle(u32)`、arena Vec及`node()/node_mut()` O(1)定位是正向基线，符合generation-owned dense arena方向。
- `UiV2CompiledDocument`同时拥有arena、String→handle BTreeMap与component graph；runtime `component_graph_from_arena()`又为每node复制source id、component String和children Vec，形成第二份identity/topology。归 **PERF-MVP-274/312**：canonical arena只存一次immutable identity/topology，parent/child/component入口使用dense side tables/handles，surface与preview共享同一generation owner。
- arena node仍以String/BTreeMap/TOML Value拥有classes/props/state/layout/style/slots/events，compiled→surface若继续重投影会放大 **PERF-MVP-276/312**；typed layout/style/event contracts应在compile generation完成，runtime只持mutable delta。
- `UiV2ResolvedStyle::merge_block()`对self/slot两张BTreeMap做完整clone后extend。当前全仓未找到产品调用，不能宣称现有帧热点；但首次接线必须按 **PERF-MVP-275** 以typed delta/borrowed declaration merge，禁止进入pointer/state稳定路径后整表复制。
- Slint lowering使用typed indices、sub-component mapping与单次lowering state发布item tree，style default在compiler pass折叠；Zircon采用“compile一次、dense owner、runtime delta”的原则，不复制其实现细节。

## 动态验收

1. nodes/components 1/100/10k、identity 0/64/4 KiB：记录arena/graph node writes、String/children clone bytes、owners、RSS与compile p95；immutable identity/topology owner=1。
2. props/state/layout/slots/events各1/100/10k：记录TOML map/value clone bytes、typed parse calls与surface build p95；stable generation parse/clone=0。
3. style blocks/rules 1/100/1k、state updates 1/10k：记录map clone bytes、merged keys、dirty nodes与p95；stable state整表clone=0，结果与当前merge语义一致。
4. current interface合同、runtime v2 asset/file-cache/surface/style tests及F4 `.zui` preview/edit trace通过。

current-source Cargo、规模counter、F4产品trace及3个dirty文件独立审查未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
