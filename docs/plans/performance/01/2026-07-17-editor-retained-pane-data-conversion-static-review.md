---
related_code:
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/**/*.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder/*.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/editor_pane_data_conversion.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01
  - docs/plans/zircon_editor/editor_ui/05
  - docs/plans/zircon_editor/editor_ui/08
  - docs/plans/zircon_editor/editor/15
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - pane_model_mapping_borrows_source_rows
  - pane_template_node_projection_borrows_source_rows
  - option query/set and Build/Export key tests
  - existing pane component, Build/Export, inspector, runtime diagnostics and UI asset tests
  - coordinated Windows zircon_editor performance test compiling/running
  - focused pane conversion and 1/100/10k row allocation traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained host pane data conversion 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion` 当前共 **211** 个 Rust 文件、**14,269** 行，已逐文件阅读 **211/211**：root 21/21、`build_export` 8/8、`pane_component_projection` 162/162、`pane_ui_asset_conversion` 12/12、`ui_asset_detail_fields` 8/8。公共 pane template hit-surface builder与结构约束测试也沿调用链审查。

当前协调式 Windows Cargo仍在从干净 target编译；focused pane tests、真实窗口 interaction trace和规模计数未完成，因此整个目录留在 `pending.md`。

## 已直接优化

- PERF-MVP-138：`model_projection`和`template_node_projection`改为借用 `ModelRc::iter`；plugin、performance、Build/Export可视行也直接借用。新增 clone probe，结构约束改为要求 borrowed mapping。
- PERF-MVP-139：option query不再逐行生成三个 lowercase String；状态集合先规范化trim key，再用 `BTreeSet::contains`检查id/label/raw。
- PERF-MVP-140：Build/Export targets不再复制完整row表；计数map改HashMap，normalized key只建一个buffer，wizard复用一次action id并借用首target。
- PERF-MVP-141：公共pane hit-surface的dispatchability与tree build两遍都借用host node，消除每节点两次完整DTO clone。
- PERF-MVP-138补充：Animation、Console、Hierarchy、Inspector原生回退不再先clone完整pane DTO；converter借用source并只clone最终host字段。
- PERF-MVP-142：CommandPalette/NotificationCenter由selection projection一次parse同时生成plain/structured rows；filtered command用保持首项语义的borrowed-id HashMap，query用allocation-free ASCII比较。
- PERF-MVP-144：无alias button属性直接借用原map；virtual window用`skip/take`停止于窗口末端；canvas variant、hex color与world transform不再建立只读临时String/Vec。
- PERF-MVP-145：UI asset detail直接借用`inspector_widget_prop_state_rows`，删除转换前整表clone。

## 待直接优化与移交

- PERF-MVP-143：Runtime Diagnostics每次pane conversion可做三次surface/tree事务。EditorUI08必须提供generation-owned完整debug snapshot；dispatch-only hit surface不能冒充all-node reflector source。
- PERF-MVP-142/144/145的局部确定项已写入，当前源Cargo与1/100/10k计数仍待验收；typed property generation、UI asset section index仍由EditorUI05/08负责。

## 动态验收

首次协调式Windows `performance_tests`广域批次完成42项：41通过，1项`asset_snapshot_normalizes_search_once_and_streams_parent_paths`使用编译期间旧source snapshot失败；当前文件已有目标表达式，必须用当前源重跑，不能把该批次记为通过。随后运行本目录行为测试与结构约束，记录1/100/10k pane/template/plugin/command rows的source-row clone、TOML parse、lowercase/key allocation、surface rebuild和p95。Command duplicate首项、Build/Export duplicate id、virtual window、MUI style、world-space、debug-reflector all-node语义以及UI asset binding/frame顺序全部等价后，才能进入 `review.md`。
