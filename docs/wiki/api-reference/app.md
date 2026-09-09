---
related_code:
  - zircon_app/src/lib.rs
implementation_files:
  - zircon_app/src
plan_sources:
  - user: 2026-09-09 API 公开接口覆盖审计
tests:
  - zircon_app/src/entry/tests
doc_type: api-reference
---

# zircon_app API

应用入口 crate 负责将运行时、插件组和产品宿主配置组合成可执行宿主。
本清单只记录 crate root 的公开 re-export；未导出的内部模块、`pub(crate)` 符号和仅测试 API 不代表稳定调用面。Rust 调用应优先使用这里的根路径，并对 feature gate 做显式配置。

## 稳定性与调用约定

- 根路径是文档首选入口，但当前工作区仍可能调整模块归属；升级时以编译器错误和源码签名为最终依据。
- `Result` 错误不可静默丢弃；启动/编辑操作失败时必须执行清理或回滚。
- 带 `#[cfg(feature = ...)]` 的类型只在启用对应 feature 时存在，生产构建应锁定 feature 集合。
- 句柄、网关、插件注册报告和执行回执都携带生命周期信息，跨线程传递前确认 `Send/Sync` 约束。

## 根导出清单

| 来源模块 | 完整路径 | 用途摘要 |
| --- | --- | --- |
| `entry` | `zircon_app::bootstrap_export_runtime` | 公开入口函数；执行启动、发现、运行或清理流程。 |
| `entry` | `zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::discover_export_root` | 公开入口函数；执行启动、发现、运行或清理流程。 |
| `entry` | `zircon_app::EditorApplicationComposition` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryConfig` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryModuleSelection` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryModuleSelectionReport` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryProfile` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryRunMode` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::EntryRunner` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ExportRuntimeBootstrapConfig` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ExportRuntimePluginFeatureRegistrationProvider` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ExportRuntimePluginRegistrationProvider` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::first_party_runtime_plugin_registrations_for_config` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::first_party_runtime_plugin_registrations_for_manifest` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::first_party_runtime_plugin_registrations_for_runtime_profile` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::HeadlessController` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::HeadlessHostError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `entry` | `zircon_app::HeadlessRunReport` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::HeadlessStopReason` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductArtifactDeliveryStatus` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductArtifactKind` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductArtifactManifest` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductCapabilityRequirement` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductComposition` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductCompositionRequest` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductConfigSource` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductConfigSourceSet` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductEntryKind` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductExitClass` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductHostCapabilityPolicy` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductHostConfigError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `entry` | `zircon_app::ProductHostConfigProvenance` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductPlatformClass` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductProcessExitCode` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductRoleDescriptor` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductRoleRequest` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductRunnerKind` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ProductRuntimeLinkage` | 公开数据类型；用于引擎配置、输入事件或编辑器工作流。 |
| `entry` | `zircon_app::ProductShutdownPolicy` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::ResolvedProductHostConfig` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `entry` | `zircon_app::retry_runtime_startup_cleanup` | 公开入口函数；执行启动、发现、运行或清理流程。 |
| `entry` | `zircon_app::RuntimeSessionCreateFailure` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `plugins` | `zircon_app::DefaultPlugins` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::DevPlugins` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::HeadlessPlugins` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::MinimalPlugins` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::PluginGroup` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::PluginGroupBuilder` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |
| `plugins` | `zircon_app::PluginGroupError` | 错误类型；调用方应匹配具体变体并保留上下文。 |
| `plugins` | `zircon_app::ResolvedPluginGroup` | 公开配置、状态或句柄类型；连接模块边界并承载可观察结果。 |

## 入口示例

```rust
use zircon_app::{EntryConfig, EntryProfile, EntryRunner};

// EntryConfig::new 只建立请求；compose 才会解析配置并启动模块。
fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let config = EntryConfig::new(EntryProfile::Runtime);
    let composition = EntryRunner::compose(config)?;
    // ProductComposition 持有本次 generation 的 runtime owner。
    drop(composition);
    Ok(())
}
```

`EntryRunner` 是无状态入口类型，当前公开构造路径是 `EntryRunner::compose(config)`；也可以直接调用 `ProductCompositionRequest::new(config).compose()`。若需要命令行解析的完整 runtime loop，使用 `EntryRunner::run_runtime_with_args(args)`（需要对应 host/diagnostic feature）。

## 与现有 Wiki 的关系

- 启动、产品宿主和插件组的行为见 [app-runtime-api](../app-runtime-api/index.md) 与 [tutorial-runtime-startup](../tutorials/runtime-module-startup.md)。

## 审计命令

```powershell
rg --no-heading --line-number '^pub use|^pub mod|^pub trait|^pub struct|^pub enum|^pub type|^pub const|^pub fn' zircon_app/src
cargo rustdoc -p zircon_app --all-features
```
