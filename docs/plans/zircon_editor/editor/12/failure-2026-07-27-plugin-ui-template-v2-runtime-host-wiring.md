---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: plugin-ui-template-v2-runtime-host-wiring
origin_plan: docs/plans/zircon_plugins/05-navigation.md
fixing_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
origin_child_dir: docs/plans/zircon_plugins/05
fixing_child_dir: docs/plans/zircon_editor/editor/12
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/template_runtime/runtime/plugin_documents.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_binding.rs
tests:
  - powershell -NoProfile -Command "$document_owner = Get-Content 'zircon_editor/src/ui/template_runtime/runtime/plugin_documents.rs' -Raw; $runtime_host = Get-Content 'zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs' -Raw; $retained_click = Get-Content 'zircon_editor/src/ui/retained_host/app/pane_surface_actions/click.rs' -Raw; if (($document_owner -notmatch 'EditorUiTemplateDescriptor') -or ($runtime_host -notmatch 'plugin_v2_documents') -or ($retained_click -notmatch 'UiPointerComponentEvent') -or ($retained_click -notmatch 'dispatch_template_action')) { throw 'plugin template descriptors, generation-owned documents, and retained-host actions are not connected' }"
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_plugin_navigation_editor -SkipBuild -VerboseOutput
---

# Editor12: 插件 UI 模板 V2 运行时宿主接线缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/05-navigation.md`
- 来源执行切片：M6-T1 Navigation Bake selected-surface retained-host action path
- 修复责任计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 交接原因：Navigation 已提供真实 `surface_entity` 行标识、V2 action payload 与对象形式 operation 参数；但 Editor12 是插件 template descriptor、动态生命周期和 retained-host materialization 的最低共享 owner。

## 失败现象与复现证据

当前静态调用链为：

```text
EditorExtensionRegistry::register_ui_template(EditorUiTemplateDescriptor)
  -> EditorHostEventController::register_editor_plugin_registration
  -> EditorExtensionRegistration registry storage
  -> EditorEventRuntimeAccess::ui_template_descriptor (query only)
```

`EditorUiHostRuntime` 仅通过显式本地文件路径调用
`register_v2_template_document_files` 建立 V2 文档；它不会读取已启用插件的
`EditorUiTemplateDescriptor`，也没有 `plugins://` 文档解析/生命周期卸载边界。
`RetainedEditorHost` 仅持有内建 template bridge 和内建 `PanePayload` 枚举；没有
`UiPointerComponentEvent.template_action` 的通用消费路径。结果是 Navigation Bake
面板即使产生带 `surface_entity` 和 `force_full_rebuild` 的 typed action，也不能由真实
host 点击送达 operation factory。`bake.zui` 的表仅声明
`row_identity_field = "surface_entity"`，而 `NavigationBakePanel.surface_rows` 也没有被
插件 descriptor/host 的通用数据投影契约消费，因此真实 surface 行从未进入 V2 surface。

## 最低共享层根因

插件 UI template descriptor 还只是 catalog 查询数据，尚未成为 generation-owned 的
V2 runtime document、插件拥有数据源的 pane presentation 和 pointer-action dispatch
contract。该缺口属于 Editor12 的 plugin lifecycle/catalog facade 与 retained host
integration，而不是 Navigation 的 route 或参数 schema。

## 架构修复验收

- 已启用插件的 `EditorUiTemplateDescriptor` 必须在同一 plugin generation 内被解析为唯一、可卸载的 V2 runtime document；`plugins://` 路径和 asset/import 生命周期由共享 owner 管理，不能由单个插件或 pane 自行读取文件。
- retained host 必须能以通用插件 pane presentation 承载该 document，并在同一 generation 投影插件拥有的动态数据源；Navigation 的 surface rows 必须包含实际 typed `surface_entity`，不能由显示索引、第一行或 `0` 推导。不得扩展一个只为 Navigation 服务的 `PanePayload` variant 或内建 bridge。
- V2 指针 dispatch 的 `UiPointerComponentEvent.template_action` 必须通过一个通用 editor operation dispatcher 转发，其 payload 保持 typed object；无 action、缺失依赖或禁用控件不得提交 operation。
- 真实 retained-host regression 覆盖：注册插件 template，surface 行选择 A 后 bake A，选择 B 后 clear B，无选择不提交，切换选择不复用 A；该测试不得直接调用 Navigation panel controller 来代替 host 点击。
- 插件 enable/disable/hot-reload 后 document 与 pointer routes 按 generation 原子更新，旧 generation 不残留可点击 route；随后重跑 Plugins05 M6-T1 的受管 Windows package gate。

## 禁止临时方案

- 不得按 `navigation.*` route、template id 或 `NavigationBake*` control id 在共享 dispatcher 或 host 中特判。
- 不得将插件 `.zui` 直接硬编码为内建 host 文件、在 pane 侧读取 `plugins://` 路径、复制 plugin registry、在 `.zui` 伪造 surface rows，或新增插件专用 `PanePayload` truth。
- 不得用 aliases、compatibility shims、silent fallback、duplicated truth、test-only bypasses 或 call-site exceptions 伪造生命周期接线。

## 修复结果与回传

Open state: `待 Editor12 建立 generation-owned plugin V2 document/pane/action contract`; Plugins05 M6-T1 不得在此之前宣称 retained-host action gate 已通过。

## 产出记录与时间

### 2026-07-28 Editor12 V2 document/runtime-host/pointer wiring

- 状态：`resolving_failure`。
- 完成项目与验证证据：已以 generation-owned descriptor set 驱动 `plugins://` V2 document 的批量替换和卸载；空 descriptor set 会移除 owner document 及旧 action，host 通过通用 `UiPointerComponentEvent.template_action` 进入 template action dispatcher，无 Navigation route/template/control 特判。静态契约为 `test_editor12_plugin_v2_document_runtime_contract.py` 3/3 与 Editor12 聚合 69/69；精确范围 `rustfmt --check --config skip_children=true`、`git diff --check` 通过。两次独立审查分别为 lifecycle C/I/M=0/0/0 与 native selected-registration C/I/M=0/0/0。受管 copy `416b041cd7524ae6a983f8801bf9bcfc` 仍为 `materializing`、18033-path closure、无 input hash/无 Cargo run，已由 Coordinator01 `validation-copy-cargo-materialization-nonterminal` failure 接管；本 failure 保持 open，尚无 Cargo、fixed return 或 commit。
