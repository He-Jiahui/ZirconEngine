---
related_code:
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/plugin_host_driver.rs
  - zircon_runtime/src/script/vm/host/vm_plugin_host_context.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
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
  - zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
  - zircon_editor/src/ui/host/editor_asset_manager/api.rs
implementation_files:
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/script/vm/host/host_registry.rs
  - zircon_runtime/src/script/vm/host/mod.rs
  - zircon_runtime/src/script/vm/mod.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_plugin.rs
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
  - zircon_editor/src/ui/workbench/view/view_registry_register_view.rs
plan_sources:
  - user: 2026-04-26 Editor Host 最小化插件加载
  - user: 2026-04-27 zircon_plugins 全量插件化收敛规划
  - .codex/plans/全系统重构方案.md
  - .codex/plans/zircon_plugins 全量插件化收敛规划.md
tests:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - cargo test -p zircon_app --locked bootstrap_accepts_required_external_runtime_plugin_when_linked_report_contributes_module --lib
  - cargo test -p zircon_runtime --locked host_registry_exposes_stable_capability_records_without_concrete_objects --lib
  - cargo test -p zircon_editor --locked minimal_host_contract --lib
  - cargo test -p zircon_editor editor_plugin_toggle_refreshes_snapshot_and_view_gate --lib --locked
  - cargo test -p zircon_editor --lib editor_runtime_consumes_plugin_registration_reports_with_capability_gate --locked
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

`editor.module_plugins` 是最小 host 的内置 activity view。它不依赖可选 subsystem，用来承载模块列表、启停动作、诊断和导出入口。它展示 builtin runtime/editor catalog 与 native discovered packages 的合并状态；启停动作统一调用 `EditorManager::set_project_plugin_enabled(...)` 或 native-aware 变体。

## Dual Loading Tracks

### EngineModule Track

`zircon_app::EntryConfig` 使用 `ProjectPluginManifest` 描述启动时 runtime 插件选择。`builtin_modules_for_config(...)` 把选择交给 `zircon_runtime::runtime_modules_for_target(...)`，该函数先生成运行模式基线，再用项目清单覆盖匹配插件 ID。`BuiltinEngineEntry::for_config_with_runtime_plugin_registrations(...)` 接收 LibraryEmbed/SourceTemplate 产物传入的 `RuntimePluginRegistrationReport`，把插件贡献的 `ModuleDescriptor` 作为启动模块注册。

这条轨道只用于 core/runtime 级 EngineModule。它仍然走 module descriptor、driver/manager/plugin descriptor、依赖解析和 `CoreRuntime::activate_module(...)`，不会被 editor host 的 VM 插件入口绕过。

### Editor Plugin Track

`EditorPluginCatalog` 聚合 editor-side package manifests、capabilities 和 editor extension registry。`EditorManager::complete_project_plugin_manifest(...)` 会把 builtin runtime-backed 和 editor-only packages 都补成项目插件选择，默认 disabled。`EditorManager::set_project_plugin_enabled(...)` 是 builtin 插件的标准启停入口：runtime-backed 和 editor-only 插件都走这条路径，能力快照立即刷新，view registry gate 随能力状态变化。

真实 editor plugin crate 产生的 `EditorPluginRegistrationReport` 进入事件层时走 `EditorEventRuntime::register_editor_plugin_registration(...)`，而不是退回成裸 view 声明。该入口会把 report 的 capability 列表绑定到 `EditorExtensionRegistration`，注册出来的 workbench view descriptor 和 editor operation descriptor 都会携带 required capabilities。View registry 允许先保存带 capability gate 的 descriptor；`list_descriptors()`、`open_descriptor()`、workspace restore、reflection menu/template projection、operation discovery 和 operation invocation 再按当前 `EditorCapabilitySnapshot` 过滤。这样插件被禁用时对应 view/menu/template/operation 不可见且不可打开或调用，启用 capability 后同一 registration report 可以立即投影到 activity descriptor、View 菜单和 editor operation 路由。

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

当前 focused validation 命令记录在 frontmatter 的 `tests` 字段中。更大范围验收仍以 workspace CI 命令为准：`cargo build --workspace --locked --verbose` 和 `cargo test --workspace --locked --verbose`。
