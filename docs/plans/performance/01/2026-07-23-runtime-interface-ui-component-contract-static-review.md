---
related_code:
  - zircon_runtime_interface/src/ui/component
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
reference_sources:
  - dev/slint/internal/core/item_tree.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - current-source Windows component contract tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface UI component 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/component/**`当前源 **25/25** 个受跟踪且clean的 Rust 文件、**2,044** 行已逐文件阅读，并反查runtime component catalog/reducer/property mutation/accessibility/render、editor palette/template/pane projection及interface合同。本轮未修改Rust源码。

## 性能结论

- `UiComponentDescriptor`长期拥有id/role/classes、recursive default props、prop/state/slot schema、event Vec与capability sets；`prop()`、`state_prop()`、`slot_schema()`和`supports_event()`都是线性查找。产品reducer/input/render/accessibility会在一次动作中重复调用这些入口，稳定catalog仍按schema长度反复字符串比较。归 **PERF-MVP-265/283**：catalog generation必须编译canonical field/event/slot id与dense lookup，不在event热路径扫descriptor Vec。
- descriptor builder的class/event去重使用`Vec::contains`，批量构造最坏O(K²)；`default_node_template()`又把完整template clone进palette并保留第二份。该成本主要发生于catalog generation，继续归 **PERF-MVP-264**，要求每generation只构建一次、共享descriptor，不把builder放进palette/event调用。
- `UiDefaultNodeTemplate::instantiate()`为每node深clone widget/control/classes/props/layout；editor palette插入真实调用该API。compiled template已知generation时应直接引用canonical template payload并只物化node identity/override，归 **PERF-MVP-274/305**，禁止实例数×模板bytes复制。
- `UiValue`在TOML↔typed两向递归重建String/Vec/BTreeMap；`display_text()`即使String-like也clone正文，flags会join，vector/numeric会format。runtime render、accessibility、property mutation和editor pane projection均有生产调用，放大 **PERF-MVP-265/266/268/290**。`fixed_float_array`还为2/3/4个数先建临时Vec，但只是import/compile微成本，不单独编号。
- `UiComponentEventEnvelope`重复拥有document/control/component/target/event kind与recursive event；adapter result/projection patch又以String-key BTreeMap携wide UiValue。普通event应使用generation-scoped document/control/field handle、single value owner与compact changed receipt，authoring/serde/error边界才物化wide DTO，归 **PERF-MVP-265/278**。
- drag metrics、category/layout/event-kind与state flags均为Copy或固定match正向基线。`UiDragSourceMetadata::summary()`会format/clone，但当前产品反查仅测试消费；未来overlay/diagnostic接线时按需物化，不为每个drag-move生成String。
- Slint的`ItemRc`只持shared `ItemTree` owner与`u32` index，静态`ItemTreeNode`保存parent/children/item-array索引；Zircon采用“compiled generation owner + dense handle + mutable state delta”原则，而不是复制Slint vtable实现。

## 动态验收

1. descriptors 1/100/10k、props/events/slots 1/32/1k、连续1M actions：记录schema String comparisons、lookup probes、catalog builds、descriptor/template clone bytes与p95；stable lookup近O(1)，catalog build≤1/generation。
2. instances 1/100/10k、template props/classes/layout 1/100/10k：记录template bytes cloned、TOML nodes、node writes与peak RSS；stable compiled payload owner=1，每instance复制随实际override而非模板全量。
3. UiValue depth 1/8/64、entries 1/1k/100k、payload 0/1KiB/1MiB：记录TOML↔typed conversions、display String/recursive clone bytes、temporary Vec/BTree nodes与render/accessibility/property p95；stable generation parse/display=0，normal event value owner=1。
4. event envelope/patch 1/1k/100k：记录document/control/field String、recursive value owners、patch maps、transactions与dirty commits；每逻辑动作single compact receipt，wide serde只在显式边界。
5. current interface合同、runtime reducer/catalog与editor palette/template用例通过；运行current-source Windows Cargo及F4 component/palette/input产品trace。

current-source Cargo、规模counter与F4产品trace未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
