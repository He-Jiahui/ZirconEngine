---
related_code:
  - zircon_editor/src/lib.rs
implementation_files:
  - zircon_editor/src
plan_sources:
  - user: 2026-09-09 API 公开接口覆盖审计
tests:
  - zircon_editor/src/tests
doc_type: api-reference
---

# zircon_editor API

编辑器 crate 提供命令、网关、插件扩展、保留式宿主与启动配置。
本清单只记录 crate root 的公开 re-export；未导出的内部模块、`pub(crate)` 符号和仅测试 API 不代表稳定调用面。Rust 调用应优先使用这里的根路径，并对 feature gate 做显式配置。

## 稳定性与调用约定

- 根路径是文档首选入口，但当前工作区仍可能调整模块归属；升级时以编译器错误和源码签名为最终依据。
- `Result` 错误不可静默丢弃；启动/编辑操作失败时必须执行清理或回滚。
- 带 `#[cfg(feature = ...)]` 的类型只在启用对应 feature 时存在，生产构建应锁定 feature 集合。
- 句柄、网关、插件注册报告和执行回执都携带生命周期信息，跨线程传递前确认 `Send/Sync` 约束。

## 根导出清单

| 来源模块 | 完整路径 | 用途摘要 |
| --- | --- | --- |
| `core::commands` | `zircon_editor::CommandEvalCtx` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::CommandEvalSnapshotHandle` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::DocumentKind` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::DocumentKindError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorCommandAction` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandCategory` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandDescriptor` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::EditorCommandDispatchError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorCommandExecutionContract` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandExecutionReceipt` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::EditorCommandExecutorRegistry` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandExecutorRegistryError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorCommandPaletteEntry` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::EditorCommandRegistry` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandRegistryError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorCommandResourceBudget` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorCommandResourceBudgetError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorCommandResultCodecId` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::EditorCommandResultCodecIdError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorKeyBinding` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorKeyChord` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorKeyChordParseError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::EditorKeymap` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorKeymapConflict` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::EditorKeymapError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::commands` | `zircon_editor::MAX_EDITOR_COMMAND_EXECUTION_TIME_MS` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `core::commands` | `zircon_editor::MAX_EDITOR_COMMAND_INPUT_BYTES` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `core::commands` | `zircon_editor::MAX_EDITOR_COMMAND_OUTPUT_BYTES` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `core::commands` | `zircon_editor::NativeCommandExecutorRegistration` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::commands` | `zircon_editor::NativePluginEditorCommandBinding` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::PlayModePredicate` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::commands` | `zircon_editor::WhenClause` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::editing::intent` | `zircon_editor::EditorIntent` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::gateway` | `zircon_editor::DetachedEditorRuntimeGateway` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::EditorRuntimeFrame` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::gateway` | `zircon_editor::EditorRuntimeGateway` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::EditorRuntimeGatewayHandle` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::GatewayError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `core::gateway` | `zircon_editor::InProcessGateway` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::PluginActivationState` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::PluginSummaryEntry` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::RuntimeCapabilities` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `core::gateway` | `zircon_editor::SessionGateway` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::SessionProfileKind` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gateway` | `zircon_editor::SharedEditorRuntimeGateway` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::gui_startup_request` | `zircon_editor::EditorGuiStartupRequest` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::plugin` | `zircon_editor::EditorExtensionCatalogReport` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::plugin` | `zircon_editor::EditorPlugin` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::plugin` | `zircon_editor::EditorPluginDescriptor` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `core::plugin` | `zircon_editor::EditorPluginRegistrationReport` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::host::module` | `zircon_editor::EDITOR_ASSET_MANAGER_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EDITOR_COMMAND_REGISTRY_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EDITOR_HOST_DRIVER_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EDITOR_KEYMAP_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EDITOR_MANAGER_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EDITOR_MODULE_NAME` | 公开常量；用于注册名、大小上限、版本或 ABI 契约。 |
| `ui::host::module` | `zircon_editor::EditorHostDriver` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `ui::host::module` | `zircon_editor::EditorModule` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `ui::host::module` | `zircon_editor::module_descriptor` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::retained_host` | `zircon_editor::EditorHostRunConfig` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::retained_host` | `zircon_editor::RetainedHostAutomationResult` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::retained_host` | `zircon_editor::run_editor` | 公开入口函数；执行启动、发现、运行或清理流程。 |
| `ui::retained_host` | `zircon_editor::run_editor_with_config` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::retained_host` | `zircon_editor::run_editor_with_startup_request` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `ui::retained_host` | `zircon_editor::run_retained_host_automation` | 公开入口函数；执行启动、发现、运行或清理流程。 |

## 入口示例

```rust
use zircon_editor::{
    run_editor_with_config, EditorHostRunConfig, SharedEditorRuntimeGateway,
};
use zircon_runtime::core::CoreHandle;

fn launch_editor(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = EditorHostRunConfig::new()
        .with_exit_after_first_presented_frame(false);
    run_editor_with_config(core, runtime_gateway, config)?;
    Ok(())
}
```

`run_editor_with_config` 的参数顺序是 `(CoreHandle, SharedEditorRuntimeGateway, EditorHostRunConfig)`；不能只传入宿主配置。`CoreHandle` 必须来自应用层已完成的产品组合，并在宿主运行期间保持有效。`SharedEditorRuntimeGateway` 是 `Arc<dyn EditorRuntimeGateway>`，通常由 `zircon_app::EntryRunner` 的 Editor 组合创建并注入；需要手动组装进程内网关时，可在拥有 `LevelSystem` 后调用 `InProcessGateway::new(core, level_system)`，再通过 `Arc::new(...)` 转换为共享类型：

```rust
use std::sync::Arc;
use zircon_editor::{InProcessGateway, SharedEditorRuntimeGateway};

fn make_gateway(
    core: zircon_runtime::core::CoreHandle,
    level_system: zircon_runtime::scene::LevelSystem,
) -> SharedEditorRuntimeGateway {
    Arc::new(InProcessGateway::new(core, level_system))
}
```

编辑器宿主会通过 gateway 访问 runtime 世界、操作、视口和能力信息；不要在调用方另建第二个 `EditorManager` 或绕过 gateway 直接持有 ECS 状态。构造器参数和 feature 组合仍以当前 crate 源码为准。

## 与现有 Wiki 的关系

- 编辑器命令、网关和 UI 生命周期见 [editor](../editor/index.md)、[tutorial-editor-command-undo](../tutorials/editor-command-undo.md)。

## 审计命令

```powershell
rg --no-heading --line-number '^pub use|^pub mod|^pub trait|^pub struct|^pub enum|^pub type|^pub const|^pub fn' zircon_editor/src
cargo rustdoc -p zircon_editor --all-features
```
