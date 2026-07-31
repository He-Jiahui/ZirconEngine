---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: retained-template-table-row-identity-selection
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_editor/editor_ui/01
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/template_nodes/node.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows/commands.rs
tests:
  - powershell -NoProfile -Command "$hit = Get-Content 'zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/hit.rs' -Raw; if ($hit -notmatch 'row_identity_field') { throw 'retained host hit contract does not preserve table row identity' }"
  - cargo test -p zircon_editor --lib retained_host --locked
---

# EditorUI01: retained template table row identity selection missing

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：插件 V2 pane 的 retained-host typed action dispatch
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`
- 交接原因：Editor12 已能将 V2 `UiActionRef` 解析为 typed operation invocation，但 native retained host 的 pointer hit 只返回 `Table` 容器节点，未解析行命中或 `row_identity_field`。行选择、control state 和 input routing 属于 EditorUI01 的最低共享输入/hit-test owner。

## 失败现象与复现证据

Navigation 的 V2 panel 使用 `Table` 的 `row_identity_field = "surface_entity"`，其 bake/clear action 读取 `=control.NavigationBakeSurfaceList.prop.selected_row_identity`。当前 `hit_test_template_nodes` 只以 surface node id 找回一个 `TemplatePaneNodeData`，`TemplateNodePointerHit` 不含 row identity，`dispatch_template_node_button` 也没有更新选择状态。因此 native retained-host 点击不能从真实行 A/B 生成 `selected_row_identity`，插件 action 无法可靠地携带 A/B 的 typed `surface_entity`。

## 最低共享层根因

EditorUI01 的 retained template pointer contract 尚未将 table collection geometry、row identity 和 selected state 收束为同一 generation-scoped control-state mutation。当前渲染 `TemplatePaneNodeData` 的 collection 数据与 native hit/dispatch 路径是分离的，不能把表格行点击作为通用 `UiSurface` interaction。

## 架构修复验收

- native table hit 必须根据投影 collection 的行几何命中真实 row，读取声明的 `row_identity_field`，而非索引、首行或字符串猜测。
- primary click 必须在同一 retained template document generation 内更新 control 的 `selected_index` 与 `selected_row_identity`；热重载/disable 后旧 generation 的状态和 route 不得继续可达。
- typed template action 必须在该状态之后解析；选择 A 后 bake A、选择 B 后 clear B、无选择或 disabled 时均不提交 operation。
- 为 retained host 添加覆盖两行 identity 切换、空选择、disabled 和 generation replacement 的 focused regression；随后重跑 Editor12 plugin V2 action gate 与 Plugins05 M6-T1 Windows package gate。

## 禁止临时方案

- 不得在 Editor12 dispatcher 或插件中按 `navigation.*` route、`NavigationBake*` control id 或 pane type 特判。
- 不得用显示索引、第一行、`0`、缓存的旧 identity 或 plugin-local selection state 代替 native table row hit。
- 不得通过 test-only callback、payload 伪造、兼容 shim 或跳过 disabled 检查掩盖该输入状态缺口。

## 修复结果与回传

Open state: `待 EditorUI01 建立 retained template table row identity/control-state interaction contract`; Editor12 typed action 链路仅在该状态可用后才可完成向 Plugins05 的回传。
