---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: template-v2-pane-dynamic-control-state-projection
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/runtime/template_action_registry.rs
  - zircon_editor/src/ui/template_runtime/runtime/projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_runtime_projection.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
tests:
  - cargo test -p zircon_editor --lib template_runtime --locked
  - cargo test -p zircon_editor --lib retained_host --locked
---

# Editor12: V2 pane dynamic control-state projection missing

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：Navigation V2 surface list typed bake/clear action
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：Navigation V2 surface list 暴露了动态 action state 缺口。EditorUI01 另行负责通用 table row hit/selection 事件；Editor12 的 V2 pane runtime 则仍把动态 component patch 仅写入 retained projection，未写入构造 native `UiSurface` 与 typed action 的 generation-scoped control state。动态选择后的动作参数无法从同一 pane/document generation 重新求值，属于 V2 runtime host 的最低 owner。

## 失败现象与复现证据

Navigation 的 V2 panel 为 `NavigationBakeSurfaceList` 声明 `row_identity_field = "surface_entity"`，其 bake/clear action 读取 `=control.NavigationBakeSurfaceList.prop.selected_row_identity`。`pane_payload_projection.rs` 的 `inject_pane_projection_attributes` 仅将 `PaneBodyPresentation.component_patches` 合并到 retained `TemplatePaneNodeData`。随后 `runtime_host.build_shared_surface(document_id)` 仍从未带 pane projection 的 compiled document 构造 `UiSurface`，而 `template_runtime_projection.rs` 只在该 surface 之后合并 retained metadata。当前 surface 与 typed action resolver 因此看不到同一份动态 rows、identity、selection state；即使 EditorUI01 产生真实 row hit，也不能使当前 generation 的 V2 action 重新解析为 A/B 对应的 typed payload。

## 最低共享层根因

Editor12 缺少从 `PaneBodyPresentation` 到 `UiSurface` 的 typed dynamic control-state projection，以及 pane/document generation 归属的 invalidation/re-resolution contract。当前 `TemplatePaneNodeData` 的 collection 只保留字符串 collection data，`RetainedUiProjection.component_patches` 和 native surface/action state 分裂，无法为所有插件统一承载 `rows`、`row_identity_field`、`selected_row_identity` 和 disabled state。

## 架构修复验收

- 对同一 pane/document generation，V2 body component patch 必须以 typed form 同时投影到 native `UiSurface`、retained node data 和 template action evaluation context；行数据及 identity 不能退化为字符串或显示索引。
- EditorUI01 的通用 row selection mutation 发生后，Editor12 必须在当前 generation 上更新 control state 并重新解析对应 `UiActionRef`；选择 A 后只允许 A 的 operation payload，选择 B 后只允许 B 的 payload。
- document replacement、plugin unload、pane teardown 和 disabled state 必须移除旧 control state 与 action token。旧 generation 的 click、row selection 或 action payload 不得继续可达。
- 为 runtime host 添加 focused regression，覆盖动态 rows/identity、两次 selection 后的 typed action re-resolution、无选择、disabled 和 generation replacement；随后重跑 EditorUI01 row-selection、Editor12 V2 action 与 Plugins05 M6-T1 Windows package gates。

## 禁止临时方案

- 不得在 Editor12 dispatcher、V2 projection 或插件中按 `navigation.*` route、`NavigationBake*` control id、pane type 或 plugin id 特判。
- 不得通过显示索引、首行、缓存 identity、plugin-local selection state、payload 伪造或 test-only callback 代替 shared `UiSurface` control state。
- 不得把 body patch 只复制到 retained rendering metadata 后宣称 native input/action 已支持；不得保留旧 generation token 或绕过 disabled/no-selection 检查。

## 修复结果与回传

Open state: `待 Editor12 将 V2 pane dynamic component patch 与 generation-scoped UiSurface/action state 收束；该能力完成后再消费 EditorUI01 的通用 table row selection contract。`

## 产出记录与时间

### 2026-07-28 Editor12 V2 pane dynamic state projection

- 状态：`resolving_failure`。
- 完成项目与验证证据：`PaneBodyPresentation` component patch 已在 typed projection、共享 surface 和 action registry 间以 pane/document/owner-generation 归属收束；rebind、document replacement、disabled/no-selection 和 stale token 均走通用失效路径，未按 Navigation 标识特判。`test_editor12_plugin_v2_pane_contract.py` 为 3/3，四个精确 Rust leaf 的 `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过。受管 Cargo 仍必须等待同一 Coordinator01 immutable-copy materialization failure 返回；本 failure 保持 open，尚无 Rust Cargo、独立复审、fixed return 或 commit。
