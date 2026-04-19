---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/module/mod.rs
  - zircon_ui/src/module/ui_module_descriptor.rs
  - zircon_ui/src/module/ui_module_name.rs
  - zircon_ui/src/event_ui/manager/ui_event_manager.rs
  - zircon_core/src/runtime/contexts/plugin_context.rs
  - zircon_core/src/runtime/descriptors/plugin_descriptor.rs
  - zircon_core/src/runtime/descriptors/service_factory.rs
  - zircon_core/src/runtime/handle/registration.rs
  - zircon_core/src/runtime/handle/resolution.rs
  - zircon_core/src/runtime/state/service_entry.rs
  - zircon_module/src/lib.rs
  - zircon_module/src/service_factory.rs
  - zircon_script/src/vm/module/module_descriptor.rs
  - zircon_script/src/vm/backend/backend_registry.rs
  - zircon_script/src/vm/backend/vm_backend.rs
  - zircon_script/src/vm/plugin/vm_plugin_instance.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/runtime/vm_plugin_manager.rs
  - zircon_editor/src/editing/ui_asset/mod.rs
  - zircon_editor/src/editing/ui_asset/session.rs
  - zircon_editor/src/editing/ui_asset/session/mod.rs
  - zircon_editor/src/editing/ui_asset/session/ui_asset_editor_session.rs
implementation_files:
  - zircon_ui/src/module/ui_module_descriptor.rs
  - zircon_core/src/runtime/contexts/plugin_context.rs
  - zircon_core/src/runtime/descriptors/plugin_descriptor.rs
  - zircon_script/src/vm/runtime/vm_plugin_manager.rs
  - zircon_editor/src/editing/ui_asset/session/ui_asset_editor_session.rs
plan_sources:
  - user: 2026-04-18 formalize convergence-gap repair spec and detailed implementation plan
  - .cursor/plans/基本路线图.md
  - .codex/plans/全系统重构方案.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_script/src/vm/tests.rs
  - zircon_editor/src/tests/editing/ui_asset.rs
  - zircon_editor/src/tests/editing/ui_asset_palette_drop.rs
  - zircon_editor/src/tests/host/manager.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
doc_type: milestone-detail
---
# Convergence Gap Closure Design

## 背景

这轮只处理当前审计仍未闭合的三处结构缺口，不开新的平行主线：

- `zircon_ui` 仍然依赖 `stub_module_descriptor`，被判定为 `skeleton`。
- `zircon_script` 已有 package discovery、slot record、backend registry，但 `PluginContext` 仍是 core-only 薄抽象，被判定为 `needs-refactor`。
- `zircon_editor/src/editing/ui_asset/session.rs` 仍然保留约 3100 行真实实现，`session/` 目录目前主要只是 façade，被判定为 `needs-refactor`。

本设计只做最小但权威的收敛：补齐真实 runtime contract、补齐 plugin host context、把 editor session 真正下沉到目录树。

## 审计基线

截至 2026-04-18 的固定事实：

| crate | 当前状态 | 证据 | 本轮目标 |
| --- | --- | --- | --- |
| `zircon_ui` | `skeleton` | `zircon_ui/src/module/ui_module_descriptor.rs` 使用 `stub_module_descriptor` | 变成真实 `ModuleDescriptor`，至少拥有一个真实 driver 和一个真实 manager |
| `zircon_script` | `needs-refactor` | 审计仍报告 `plugin-runtime-gap` | 把 host context、backend family、slot lifecycle 变成显式契约 |
| `zircon_editor` | `needs-refactor` | `editing/ui_asset/session.rs` 仍是唯一大型生产热点 | 把真实实现迁入 `session/` 子树，消除热点 |

额外验证事实也必须记录：`cargo test -p zircon_editor --no-run --message-format short` 当前先被 `zircon_asset` 的 `project_asset_manager/loading` 回归阻塞，所以 editor 的最终验收需要区分“editor 本地拆分完成”和“全链 cargo 验证已恢复”。

## 设计目标

- `zircon_ui` 不再依赖任何 stub descriptor，并能通过 core 生命周期解析真实 runtime service。
- `zircon_script` 不再只靠 `CoreHandle` 回查自己，而是拥有真实的 `PluginContext -> VmPluginHostContext` 传递链。
- `zircon_script` 的 backend 不再只是若干实例 map，而是显式的 backend family。
- `zircon_editor` 的 `UiAssetEditorSession` 真正回到 `session/` 子树；旧 `session.rs` 不再承载实现中心角色。

## 非目标

- 不在这轮同时收敛 `zircon_app`、`zircon_scene`、`zircon_render_server` 命名漂移。
- 不在这轮接入外部真实 VM 执行器。
- 不顺带改 editor 产品行为或 UI 交互设计。

## 总体顺序

执行顺序固定为：

1. `zircon_ui`
2. `zircon_core` + `zircon_module` plugin contract
3. `zircon_script`
4. `zircon_editor`

原因：

- 先消掉 `zircon_ui` 的 stub，能最快减少确定性红点。
- `zircon_script` 的真正缺口在 core plugin contract 过薄，不先补 core，后面 host context 只能继续走旁路。
- `zircon_editor` 最重，应该在下层 contract 稳定后再拆。

## 子系统设计

### `zircon_ui`

目标文件结构固定为：

```text
zircon_ui/src/module/
  mod.rs
  ui_module_name.rs
  ui_config.rs
  ui_runtime_driver.rs
  ui_module_descriptor.rs
```

设计决定：

- `UiModule` 保持为唯一 owner。
- 新增 `UiRuntimeDriver`，允许它是 no-op runtime anchor，但必须是真实类型和真实 `DriverDescriptor`。
- `UiEventManager` 升格为真实 `ManagerDescriptor` 服务。
- 增加稳定服务名：
  - `UI_RUNTIME_DRIVER_NAME = "UiModule.Driver.UiRuntimeDriver"`
  - `UI_EVENT_MANAGER_NAME = "UiModule.Manager.UiEventManager"`
- `UiEventManager` 依赖 `UiRuntimeDriver`，不再只是本地直接构造对象。

目标状态：

- `module_descriptor()` 不再调用 `stub_module_descriptor`
- `CoreRuntime` 激活 `UiModule` 后能解析出 `UiRuntimeDriver` 与 `UiEventManager`
- 审计结果中 `zircon_ui.status == "converged"`

### `zircon_core` 与 `zircon_module`

当前 plugin 路径的问题不在 `resolve_plugin` 是否存在，而在 `PluginDescriptor` 仍复用普通 `ServiceFactory = Fn(&CoreHandle)`。这迫使 plugin runtime 构造时只能拿到 core handle，真正的 plugin host 语义无法进入 factory 面。

本轮固定收敛为：

```text
zircon_core/src/runtime/descriptors/
  service_factory.rs
  plugin_factory.rs
  plugin_descriptor.rs

zircon_core/src/runtime/contexts/
  plugin_context.rs

zircon_core/src/runtime/state/
  service_entry.rs

zircon_core/src/runtime/handle/
  registration.rs
  resolution.rs
```

设计决定：

- 保留 `ServiceFactory` 给 driver/manager。
- 新增 `PluginFactory`，签名固定为显式接收 `&PluginContext`。
- `PluginDescriptor` 改为保存 `PluginFactory`。
- `ServiceEntry` 显式区分普通 service factory 与 plugin factory。
- `zircon_module` 新增 `plugin_factory()` helper，而不是让 subsystem 直接手写 `Arc<dyn Fn(&PluginContext)>`。

`PluginContext` 最小字段：

- 插件注册名
- `CoreWeak`
- `package_root: Option<PathBuf>`
- `source_root: Option<PathBuf>`
- `data_root: Option<PathBuf>`

这些 root 在 core 层允许先为空，但契约必须存在。

### `zircon_script`

`zircon_script` 当前不是“没有 runtime”，而是 runtime 元素没有被一个真实宿主上下文绑定起来。本轮目标结构固定为：

```text
zircon_script/src/vm/backend/
  backend_registry.rs
  builtin_vm_backend_family.rs
  vm_backend.rs
  vm_backend_family.rs

zircon_script/src/vm/host/
  vm_plugin_host_context.rs
  vm_plugin_slot_lifecycle.rs

zircon_script/src/vm/runtime/
  hot_reload_coordinator.rs
  vm_plugin_manager.rs
```

核心决定：

- 新增 `VmPluginHostContext`，放在 core `PluginContext` 之上，至少包含：
  - `plugin: PluginContext`
  - `capabilities: CapabilitySet`
  - `backend_selector: String`
  - `package_source: VmPluginPackageSource`
  - `host_registry: HostRegistry`
  - `slot_lifecycle: Arc<dyn VmPluginSlotLifecycle + Send + Sync>`
- 新增 `VmBackendFamily`；`VmBackendRegistry` 改成 family registry。
- 内建实现使用 `BuiltinVmBackendFamily`，支持 `builtin:mock`、`builtin:unavailable`，并兼容现有 `mock`、`unavailable` 别名。
- `VmBackend::load_package()` 和 `VmPluginInstance::activate()` 改成接收 `&VmPluginHostContext`。
- `VmPluginManager` 负责保存 base `PluginContext`、构造 host context、解析 backend family、暴露 slot lifecycle façade。

### `zircon_editor`

当前 `zircon_editor/src/editing/ui_asset/mod.rs` 已经通过 `#[path = "session/mod.rs"] mod session;` 指向目录模块，但 `session/ui_asset_editor_session.rs` 仍然通过 `#[path = "../session.rs"]` 把全部真实实现导回旧文件。这个结构必须真正收敛。

目标目录固定为：

```text
zircon_editor/src/editing/ui_asset/session/
  mod.rs
  ui_asset_editor_session.rs
  lifecycle.rs
  command_entry.rs
  preview_compile.rs
  style_inspection.rs
  hierarchy_projection.rs
  palette_state.rs
  binding_state.rs
  session_state.rs
```

责任分配：

- `ui_asset_editor_session.rs`：仅保留 struct、error、字段定义和窄 façade。
- `lifecycle.rs`：`from_source`、revalidate、preview rebuild、canonical save/load。
- `command_entry.rs`：`apply_command`、`apply_command_with_effects`、`undo`、`redo`。
- `palette_state.rs`：palette selection、drag target、target cycling、promote/reference 相关 palette 状态。
- `binding_state.rs`：binding selection、payload 变更、binding editor 状态。
- `preview_compile.rs`、`style_inspection.rs`、`hierarchy_projection.rs`、`session_state.rs`：承接从旧大文件迁出的对应逻辑。

完成条件：

- `session.rs` 被删除，或降为不会再触发审计热点的极薄 shim。
- `UiAssetEditorSession::from_source` 对外签名保持兼容。
- `ui_asset`、`ui_asset_palette_drop`、`ui_asset_sessions` 结构测试维持通过。

## 验收标准

- `zircon_ui`：无 `stub_module_descriptor`，并且能解析 `UiRuntimeDriver` 与 `UiEventManager`。
- `zircon_script`：plugin factory 构造链路走 `PluginContext`，runtime 能显式构建 `VmPluginHostContext`，backend selection 走 family。
- `zircon_editor`：`editing/ui_asset/session.rs` 不再是大型生产文件热点，真实实现留在 `session/` 子树。
- 审计脚本结果中：
  - `zircon_ui.status == "converged"`
  - `zircon_script.status == "converged"`
  - `zircon_editor.status == "converged"`
  - 不再出现 `stub-module-descriptor`
  - 不再出现 `plugin-runtime-gap`
  - 不再出现 `large-production-file`

## 默认假设

- `UiRuntimeDriver` 可以是 no-op runtime anchor，但必须是真实 service type。
- `PluginContext` 的 root 字段在 core 构造时可以为空；script runtime 负责在具体 package load 阶段补齐 package/source 语义。
- `real backend` 的定义是“可插拔 backend family + 完整 host context + slot lifecycle contract 已到位”，不是必须接通外部执行器。
- editor 的最终 cargo 绿灯依赖上游 `zircon_asset` 当前回归先被清理；如果该回归仍在，实现报告必须明确区分“本轮结构改动完成”与“全链验证仍被上游阻塞”。
