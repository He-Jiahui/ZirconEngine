---
related_code:
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_sdk/mod.rs
  - zircon_editor/src/core/editor_plugin_sdk/lifecycle.rs
  - zircon_editor/src/core/editor_plugin_sdk/examples.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/minimal_host_contract.rs
  - zircon_editor/src/ui/host/host_capability_bridge.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/editor_manager_minimal_host.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/registration_projection.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/workbench/model/menu/view_menu.rs
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
implementation_files:
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_sdk/mod.rs
  - zircon_editor/src/core/editor_plugin_sdk/lifecycle.rs
  - zircon_editor/src/core/editor_plugin_sdk/examples.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/host/minimal_host_contract.rs
  - zircon_editor/src/ui/host/host_capability_bridge.rs
  - zircon_editor/src/ui/host/editor_capabilities.rs
  - zircon_editor/src/ui/host/editor_subsystems.rs
  - zircon_editor/src/ui/host/editor_manager_minimal_host.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/mod.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/native_registration/registration_projection.rs
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/panes.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/workbench/model/menu/view_menu.rs
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
  - zircon_editor/fixtures/workbench/default-layout.json
  - zircon_editor/fixtures/workbench/view-descriptors.json
  - zircon_editor/fixtures/workbench/view-instances.json
plan_sources:
  - user: 2026-04-26 Editor Host 最小化插件加载
  - user: 2026-04-27 zircon_plugins 全量插件化收敛规划
  - user: 2026-05-02 ZirconEngine Unity 式编辑器优先补齐计划
  - .codex/plans/全系统重构方案.md
  - .codex/plans/zircon_plugins 全量插件化收敛规划.md
  - .codex/plans/ZirconEngine Unity 式编辑器优先补齐计划.md
tests:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/workbench/view_model/document_workspace.rs
  - cargo test -p zircon_app --locked bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module --lib
  - cargo test -p zircon_runtime --locked host_registry_exposes_stable_capability_records_without_concrete_objects --lib
  - cargo test -p zircon_editor --locked minimal_host_contract --lib
  - cargo test -p zircon_editor editor_plugin_toggle_refreshes_snapshot_and_view_gate --lib --locked
  - cargo test -p zircon_editor --lib editor_runtime_consumes_plugin_registration_reports_with_capability_gate --locked
  - cargo test -p zircon_editor --lib editor_plugin_sdk --locked
  - cargo test -p zircon_editor --lib default_preview_fixture_ --locked --jobs 1
  - cargo test -p zircon_editor --lib module_plugin --locked --jobs 1
  - cargo test -p zircon_editor --lib live_backend --locked --jobs 1
  - cargo test -p zircon_runtime --lib native_live_host --locked --jobs 1
  - cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_pane_projects_editable_field_nodes_and_actions --locked --jobs 1
doc_type: module-detail
---

# Editor Host Minimal Plugin Loading

## Purpose

`zircon_editor` 的 host 现在固定一条最小启动合同，并把扩展加载分成两条轨道：

- 核心 runtime 插件走 `zircon_app` 的 EngineModule 选择，在 `CoreRuntime` 注册/激活阶段进入。
- 工具和业务扩展走标准 editor plugin/catalog/capability 入口；VM 插件入口仍然只通过 host capability handle 消费 editor host 能力。

这条边界的目标是让关闭扩展后的 editor 仍然可启动、可显示最小 UI、可访问基础 asset 能力，并保留基础 scene interaction 合同；扩展失败只产生诊断，不阻断 host 主流程。

## Minimal Host Contract

最小 host 合同由 `zircon_editor/src/ui/host/minimal_host_contract.rs` 固化。当前白名单是：

- `editor.host.ui_shell`
- `editor.host.asset_core`
- `editor.host.scene_interaction`
- `editor.host.runtime_render_embed`
- `editor.host.plugin_management`
- `editor.host.capability_bridge`

这些能力代表 host 必须可自检的基础壳层，而不是全部 editor tooling。扩展黑名单明确把作者态或业务态能力挡在最小 host 之外：

- `editor.extension.animation_authoring`
- `editor.extension.ui_asset_authoring`
- `editor.extension.runtime_diagnostics`
- `editor.extension.native_window_hosting`

`EditorManager::minimal_host_report()` 返回当前最小能力自检结果。当前实现是静态合同自检：所有白名单能力都视为 host baseline，`missing_capabilities()` 必须为空。后续如果某个基础能力变成按需初始化，必须先更新这份合同和对应测试，再调整启动顺序。

`EditorCapabilitySnapshot` 把最小 host 能力、当前启用的 builtin editor subsystem，以及通过 `EditorManager::set_editor_capabilities_enabled(...)` 显式启用的插件 capability 合并成 UI 可直接消费的快照。`editor_subsystems.rs` 只用内建 optional subsystem 集合计算 disabled subsystem，但不会丢弃插件注册报告贡献的自定义 capability；这些自定义 capability 会继续参与 view、menu、template 和 operation gate。`ViewDescriptor.required_capabilities` 是窗口可见和可打开的唯一门控依据；禁用插件后，registry 的 `list_descriptors()` 会隐藏对应 view，`open_descriptor()` 和 workspace restore 也会拒绝缺能力的 view。

`editor.module_plugins` 是最小 host 的内置 activity view，用户侧标题固定为 `Plugin Manager`。它不依赖可选 subsystem，用来承载模块列表、启停动作、诊断和导出入口。默认 workbench fixture 会把它放进左下抽屉并保持折叠，同时通过 View 菜单暴露 `Plugin Manager` 入口；这样最小 host 启动时已经具备 Unity 式插件管理入口，但不会抢占默认 Project/Hierarchy 工作区。它展示 builtin runtime/editor catalog 与 native discovered packages 的合并状态；启停动作统一调用 `EditorManager::set_project_plugin_enabled(...)` 或 native-aware 变体。

Plugin Manager pane payload 现在为每个插件行稳定投影 `primary_action_label/id`、`packaging_action_label/id`、`target_modes_action_label/id`、`unload_action_label/id` 和 `hot_reload_action_label/id`。宿主生成的 action id 使用 `Plugin.Enable.<id>`、`Plugin.Disable.<id>`、`Plugin.Packaging.Next.<id>`、`Plugin.TargetModes.Next.<id>`、`Plugin.Unload.<id>` 和 `Plugin.HotReload.<id>`，Slint host 通过 `ModulePluginAction` 控件入口分发。Enable/Disable、packaging 和 target modes 会回写现有 project manifest；Unload/Hot Reload 会进入 `SlintEditorHost.module_plugin_live_host_backend` 持有的 `ModulePluginLiveHostBackend` adapter seam。默认 backend 现在是 runtime-owned `NativePluginLiveHost`：Hot Reload 会以当前 project root 递归发现 `plugin.toml`，通过 runtime 的 `NativePluginLoader::load_discovered_editor(...)` 加载 editor native package，并替换同 id 的已加载 library handle；Unload 会调用已加载插件的 editor behavior unload hook，然后释放 handle。该 runtime-owned host 同时提供从导出根目录 `plugins/native_plugins.toml` 批量装载 runtime/editor native packages 的入口，并在 runtime 路径返回 `RuntimePluginRegistrationReport`，让后续 runtime startup 可以持有同一组 live handles 而不是临时加载后丢弃动态库。若项目还没有构建 native dynamic library、插件没有 editor behavior、或没有先热重载过，backend 会返回具体诊断。`UnavailableModulePluginLiveHostBackend` 仍保留给测试和未来 mock/unavailable host 场景。必需插件的 primary action 会显示为 `Required` 且没有 action id，避免 UI 发起会被 manager 拒绝的禁用请求；诊断仍然随插件行和 pane 级 `payload_diagnostics` 一起投影。

运行时启动现在通过 `NativePluginRuntimeBootstrap` 持有 `CoreHandle` 与 `NativePluginLiveHost`。`EntryRunner::bootstrap_with_native_plugins_from_export_root(...)` 会先让 live host 装载导出根目录中的 runtime native packages，再把同一份 host 产生的注册报告交给 runtime bootstrap；调用方持有该 bootstrap bundle 时，已经加载成功的 native dynamic library handle 会跟运行时一起存活。这个 bundle 直接暴露运行时行为入口：可以按插件查询或批量列出 `NativePluginRuntimeBehaviorDescriptor`，并通过 `invoke_runtime_plugin_command(...)`、`save_runtime_plugin_state(...)`、`restore_runtime_plugin_state(...)` 调用 ABI v2 的 command/state 表；也可以用 `dispatch_runtime_plugin_command(...)` 对所有已加载 runtime native 插件广播命令，用 `save_runtime_plugin_states(...)` / `restore_runtime_plugin_states(...)` 捕获和恢复插件状态快照。`enter_runtime_play_mode(...)` 与 `exit_runtime_play_mode(...)` 已经把状态快照和 `play-mode.enter` / `play-mode.exit` 广播组合成后端 contract。对应 report 提供 `is_clean(...)`、失败调用计数和聚合诊断，后续 Runtime Diagnostics 或播放模式 UI 可以直接消费这些结果。需要组装更长生命周期 session 的调用方也可以用 `into_parts(...)` 一次取出 `CoreHandle`、live host 和诊断。后续 runtime session 或播放模式调度层只需要消费这个 owner，而不需要重新加载动态库。

Plugin Manager 的可视行渲染现在也进入 host contract。`ModulePluginsPaneData.nodes` 会包含静态 pane body 节点、`ModulePluginListSlotAnchor`，以及每个插件行的 `ModulePluginRow.<id>`、标题/状态/诊断文本和一组 `ModulePluginAction` button 节点。每个 button 的 `action_id` 直接使用上面的 stable action id，行节点的 `actions` 集合也带同一组 label/id，方便后续 Slint 或其它 host renderer 选择“按钮组”或“单独按钮”两种消费方式。

`editor.runtime_diagnostics` 仍然是可选 `editor.extension.runtime_diagnostics` subsystem 的 activity view。默认启用和默认 preview fixture 会把它放进右下抽屉并保持折叠，同时通过 View 菜单暴露 `Runtime Diagnostics` 入口；当该 subsystem 被配置禁用时，descriptor gate 和 workspace restore 仍然按 capability snapshot 隐藏或拒绝它。

## Dual Loading Tracks

### EngineModule Track

`zircon_app::EntryConfig` 使用 `ProjectPluginManifest` 描述启动时 runtime 插件选择。`builtin_modules_for_config(...)` 把选择交给 `zircon_runtime::runtime_modules_for_target(...)`，该函数先生成运行模式基线，再用项目清单覆盖匹配插件 ID。`BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(...)` 接收 LibraryEmbed/SourceTemplate 产物传入的 `RuntimePluginRegistrationReport`，把插件贡献的 `ModuleDescriptor` 作为启动模块注册。

这条轨道只用于 core/runtime 级 EngineModule。它仍然走 module descriptor、driver/manager/plugin descriptor、依赖解析和 `CoreRuntime::activate_module(...)`，不会被 editor host 的 VM 插件入口绕过。

### Editor Plugin Track

`EditorPluginCatalog` 聚合 editor-side package manifests、capabilities 和 editor extension registry。`EditorManager::complete_project_plugin_manifest(...)` 会把 builtin runtime-backed 和 editor-only packages 都补成项目插件选择，默认 disabled。`EditorManager::set_project_plugin_enabled(...)` 是 builtin 插件的标准启停入口：runtime-backed 和 editor-only 插件都走这条路径，能力快照立即刷新，view registry gate 随能力状态变化。

真实 editor plugin crate 产生的 `EditorPluginRegistrationReport` 进入事件层时走 `EditorEventRuntime::register_editor_plugin_registration(...)`，而不是退回成裸 view 声明。该入口会把 report 的 capability 列表绑定到 `EditorExtensionRegistration`，注册出来的 workbench view descriptor 和 editor operation descriptor 都会携带 required capabilities。View registry 允许先保存带 capability gate 的 descriptor；`list_descriptors()`、`open_descriptor()`、workspace restore、reflection menu/template projection、operation discovery 和 operation invocation 再按当前 `EditorCapabilitySnapshot` 过滤。这样插件被禁用时对应 view/menu/template/operation 不可见且不可打开或调用，启用 capability 后同一 registration report 可以立即投影到 activity descriptor、View 菜单和 editor operation 路由。

### Editor Plugin SDK v1

`zircon_editor::core::editor_plugin_sdk` 是给插件作者使用的稳定 facade。它集中 re-export `EditorPlugin`、`EditorPluginDescriptor`、`EditorExtensionRegistry`、`EditorOperationDescriptor`、窗口/菜单/组件 drawer/UI template 描述符，以及本轮新增的 asset importer、asset editor 和 lifecycle 类型。插件实现仍然把真实注册写进 `register_editor_extensions(...)`，但外部作者不需要追踪 editor core 内部文件布局。

SDK v1 的生命周期是报告型合同，不直接把 editor 内部对象暴露给插件。`EditorPluginRegistrationReport::from_plugin(...)` 会记录 `Loaded` 和 `Enabled` 两个阶段，并调用插件的 `on_lifecycle_event(...)` hook。注册完成后，host 可以继续通过 `EditorPluginRegistrationReport::record_lifecycle_event(...)` 或 `EditorPluginCatalog::record_lifecycle_event(...)` 派发 `Disabled`、`Unloaded`、`HotReloaded`、`EnteredPlayMode`、`ExitedPlayMode`、`SceneChanged`、`AssetChanged`、`UiMessage` 等后续事件；subject 字段用于携带场景路径、资源路径、UI message id 或 native library 路径这类轻量上下文。未知插件不会被触发 hook，而是返回 lifecycle diagnostic。native package projection 也会为 projected editor package 生成同样的 lifecycle report。hook 失败只写入 `EditorPluginLifecycleReport` 和 report diagnostics，扩展注册结果仍然保留，符合当前 host 的“扩展失败降级、主流程继续”策略。

资源 authoring 贡献现在进入同一个 extension registry：

- `AssetImporterDescriptor` 声明 importer id、显示名、绑定 operation、源文件扩展名、输出资源 kind、priority 和 required capabilities。扩展名会去掉前导点、转小写并去重。
- `AssetEditorDescriptor` 声明资源 kind、打开的 view id、显示名、绑定 operation 和 required capabilities。
- `EditorEventRuntime::register_editor_extension_with_required_capabilities(...)` 会在提交前确认 importer/editor 绑定的 operation 已经存在，并把 importer/editor 的 id 或 asset kind 纳入跨插件重复贡献校验。
- `EditorEventRuntime::asset_importers_for_extension(...)` 和 `asset_editor_descriptor(...)` 按当前 capability snapshot 过滤结果，和 component drawer/template 查询保持同一门控语义。

`editor_plugin_sdk::examples` 提供两个可编译示例：`ExampleWindowEditorPlugin` 贡献窗口、菜单和 operation；`ExampleAssetInspectorPlugin` 贡献模型 importer、asset editor、UI template 和组件 drawer。它们用于 SDK contract 测试，也作为后续真实插件 crate 的最小参考形状。

Inspector host contract 先补上了插件 drawer 可见降级出口：当 pane `info` 携带 missing/unloaded/unavailable 的插件或 drawer 诊断时，`InspectorPluginComponentFallback` 节点会进入 `InspectorPaneData.nodes`，用 warning 级别提示组件数据仍被保护。真实 drawer 后端、卸载后的组件 schema 保留策略和 undoable field commit 仍要等 `editor_extension` / `editor_operation` 活跃边界释放后接入。

### VM Plugin Track

`EditorManager::load_vm_extension_package(...)` 是 editor host 侧的 VM 扩展加载入口。它解析 `ScriptModule.Manager.VmPluginManager`，调用 VM manager 加载包，并把错误收敛成 `EditorVmExtensionLoadReport`。

这条入口不直接泄露 editor 内部对象。VM 插件看到的是 `VmPluginHostContext` 中共享的 `HostRegistry` 和 capability handles；真实 editor manager、asset manager、layout manager、scene objects 都不会直接交给插件。

## Capability Bridge

`zircon_runtime::script::vm::HostRegistry` 现在保存 `HostCapabilityRecord { handle, label }`，并提供：

- `register_capability(label)` 创建稳定 `HostHandle`
- `capability(handle)` 查询 handle 对应的 label record
- `capabilities()` 以 handle 顺序返回快照
- `is_valid(handle)` 保留轻量有效性检查

`zircon_editor/src/ui/host/host_capability_bridge.rs` 在 `EditorUiHost::new(...)` 中尝试解析 `ScriptModule.Driver.PluginHostDriver`，并把最小 host 白名单注册到 VM host registry。注册成功后，`EditorManager::vm_extension_capability_report()` 能返回每个 capability 对应的 `HostHandle`。

如果 `ScriptModule` 不在当前 runtime 中，bridge 只记录诊断并返回空 handle 集。这个行为是刻意的：最小 editor host 不应该因为工具插件通道缺失而启动失败。

## Failure Isolation

扩展加载失败不会 panic，也不会让 `EditorManager` 不可用：

- 缺少 `PluginHostDriver` 时，`vm_extension_capability_report().diagnostics()` 记录缺失 driver。
- 缺少 `VmPluginManager` 时，`load_vm_extension_package(...)` 返回诊断而不是错误传播到 host 生命周期。
- 默认 unavailable backend 拒绝包时，`EditorVmExtensionLoadReport` 记录 `BackendUnavailable`，`loaded_slot()` 为 `None`。

这保持了“扩展失败降级、主流程继续”的策略。真实 VM backend 接入后也必须保持这个边界：backend 错误转换成 load report，不能直接破坏 editor host baseline。

## Validation

新增回归覆盖三层边界：

- `bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module` 验证 app 入口能通过 linked runtime plugin registration 满足 required 外部插件并注册其 EngineModule。
- `host_registry_exposes_stable_capability_records_without_concrete_objects` 验证 VM host registry 只暴露稳定 handle record，不暴露具体对象。
- `minimal_host_contract` 测试组验证最小能力白名单、扩展黑名单、ScriptModule 缺席时不中断 host、ScriptModule 存在时注册 VM handles，以及 unavailable backend 失败被报告而不是破坏 host。
- `editor_plugin_sdk` 测试组验证 SDK 示例插件能聚合窗口和资源 authoring 贡献、注册后 lifecycle dispatch 能写回 registration/catalog report、未知插件不会误触发 hook、生命周期失败不会丢弃 extension registry、asset importer/editor 描述符会规范化扩展名和 capability gate。
- `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` 验证 Plugin Manager pane payload 会携带插件行启用/禁用、打包策略切换、target mode 切换、Unload 和 Hot Reload action id。
- `live_backend_dispatch_routes_unload_and_hot_reload_commands` 验证 Plugin Manager 的 Unload/Hot Reload action 会路由到可替换 live host backend seam；`runtime_native_live_backend_reports_missing_editor_package_on_hot_reload` 和 runtime `native_live_host_reports_missing_editor_package_on_hot_reload` 验证默认 runtime-owned native backend 在项目根缺失或 editor native package 不存在时会给出明确失败诊断；`native_live_host_loads_runtime_export_diagnostics_without_handles` 锁定从导出根目录加载 runtime native packages 时的诊断和注册报告 contract；`native_live_host_runtime_behavior_calls_report_unloaded_plugin` 锁定 runtime behavior descriptor、command、save-state 和 restore-state 在插件未加载时返回同一 live-host 诊断；`native_live_host_runtime_broadcasts_and_snapshots_empty_when_no_plugins_loaded` 与 `native_live_host_runtime_snapshot_restore_reports_unloaded_plugins` 锁定广播 command、play-mode 状态快照和卸载后 restore 诊断 contract；`native_live_host_treats_missing_unload_hook_as_noop_unload` 锁定缺少 unload callback/behavior table 时释放 handle 并保留诊断的降级策略。
- `inspector_pane_projects_editable_field_nodes_and_actions` 和 `inspector_pane_marks_plugin_component_drawer_fallback` 锁定 Inspector host contract：字段节点必须带稳定 edit/commit action id，插件 drawer 不可用时必须投影 warning fallback。

当前 focused validation 命令记录在 frontmatter 的 `tests` 字段中。更大范围验收仍以 workspace CI 命令为准：`cargo build --workspace --locked --verbose` 和 `cargo test --workspace --locked --verbose`。
