---
related_code:
  - zircon_plugins/plugin_sdk/src/declaration.rs
  - zircon_plugins/plugin_sdk/src/declaration/macros.rs
  - zircon_plugins/plugin_sdk/src/runtime.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/plugin_sdk/src/registration.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/plugin_sdk/src
  - zircon_plugins/native_dynamic_fixture
doc_type: api-reference
title: Plugin SDK 与 Rust API
status: source-audited
---

# Plugin SDK 与 Rust API

`zircon_plugin_sdk` 是插件作者使用的薄层 API。它复用 `zircon_runtime` 的真实描述符和扩展注册表，同时负责声明投影、manifest builder、linked 注册、editor contribution 和 native ABI 帮助宏。SDK 当前 API 版本为 `0.2.0`。

## Cargo features

SDK 按用途条件编译：`declaration` 提供数据声明与宏；`runtime` 提供运行时描述符、manifest、注册 builder 和测试运行时；`editor` 提供编辑器插件声明；`editor_contribution` 提供序列化编辑器贡献；`native` 提供 C ABI 类型及 dist 宏。插件应只启用需要的 feature，尤其不要让纯 native dist 意外依赖编辑器实现。

## 声明包元数据

以下字段和枚举来自 `declaration.rs`：

- `PluginTarget`: `ClientRuntime`、`ServerRuntime`、`EditorHost`。
- `PluginPlatform`: `Windows`、`Linux`、`Macos`、`Android`、`Ios`、`WebGpu`、`Wasm`、`Headless`。
- `PluginMaturityLevel`: `Core`、`Stable`、`Beta`、`Experimental`、`Externalized`、`Stub`、`Deprecated`。
- `PluginPackaging`: `SourceTemplate`、`LibraryEmbed`、`NativeDynamic`。
- `PluginCapabilityRole`: `RuntimeRegistration`、`EditorRegistration`、`RuntimeEditorRegistration`、`RequestedOnly`。
- `PluginPackageRole`: `Production`、`DeveloperTool`、`Sample`、`TestFixture`。

典型声明使用源码中的 `declare_plugin!` 语法：

```rust
use zircon_plugin_sdk::declare_plugin;

declare_plugin! {
    pub EXAMPLE_DECLARATION {
        id: PLUGIN_ID = "example",
        display_name: "Example",
        category: runtime,
        module: MODULE_NAME = "example.runtime",
        crate_name: CRATE_NAME = "zircon_plugin_example_runtime",
        module_description: "Example runtime plugin",
        targets: [ClientRuntime, ServerRuntime, EditorHost],
        platforms: [Windows, Linux, Macos],
        capabilities: [
            RUNTIME_CAPABILITY = "runtime.plugin.example" => RuntimeRegistration
        ],
        maturity: Experimental,
        packaging: [SourceTemplate, LibraryEmbed, NativeDynamic],
    }
}
```

宏还支持 `package_role` 与 `native_projection`。后者声明 runtime/editor 原生入口、模块、系统、事件和 extension projection。不要在 `plugin.toml` 再写一份相同字段；调用声明生成 manifest。

## 构建 linked 描述符

`PluginDeclaration::runtime_declaration(crate_name)` 会校验 ID 是否为 canonical `RuntimePluginId` key，过滤出由 runtime 提供的 capability，并产生 `RuntimePluginDeclaration`。后者公开这些链式 API：

```rust
let plugin_id = zircon_runtime::builtin::RuntimePluginId::parse_key("example")
    .ok_or("invalid runtime plugin id")?;
let declaration = zircon_plugin_sdk::RuntimePluginDeclaration::new(
    "example",
    "Example",
    plugin_id,
    "zircon_plugin_example_runtime",
)
.with_category("runtime")
.with_enabled_by_default(false)
.with_required_by_default(false)
.with_target_modes([
    zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
    zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
])
.with_capability("runtime.plugin.example")
.with_system_sets(["example.main"])
.with_system_anchors(["example.tick"]);

let descriptor = declaration.descriptor();
let manifest = declaration.package_manifest();
```

可继续调用 `with_init_level`、`with_module_descriptor`、`with_module_dependency`、`with_maturity`、`with_capability_status`、`with_optional_feature`、`with_provided_interface(_id)`、`with_default_packaging` 与 `with_package_role`。`descriptor()` 保留 declaration，`into_descriptor()` 消耗它。

## 注册模块与运行时对象

`RuntimePluginRegistrationBuilder` 接受 `&mut RuntimeExtensionRegistry`。必须先取得模块 owner，再把注册项归属到该 owner；卸载时引擎据此撤销全部资源。

```rust
use zircon_plugin_sdk::registration::RuntimePluginRegistrationBuilder;
use zircon_runtime::scene::ecs::{SystemRef, SystemStage};

let mut module = RuntimePluginRegistrationBuilder::new(registry)
    .module("example.runtime")?;

module.resource(ExampleState::default)?;
module.component(example_component_descriptor())?;
module.runtime_scene_system("example.tick", SystemStage::Update, || {
    |context| {
        // 每个 scene-system 实例获得独立 FnMut 回调。
        let _ = context;
        Ok(())
    }
})
.in_set("example.main")
.after(SystemRef::System("zircon.scene.world_transform".into()))
.with_order(10)
.register()?;
```

模块 builder 还提供：

| API | 注册内容 |
|---|---|
| `resource<T>(factory)` | 每个 world 一份的新资源 |
| `component(descriptor)` | 场景组件类型 |
| `event<E>(manifest)` | 强类型 ECS 事件及其外部 schema |
| `plugin_option(manifest)` | 项目可配置选项 |
| `plugin_event_catalog(manifest)` | 一组稳定事件 ID |
| `export_interface<T>(Arc<T>)` | 导出 typed bridge 接口 |
| `import_interface<T>()` | 创建可晚绑定/失效的 `BridgeImport<T>` |
| `owner_revocation_listener(callback)` | owner 被卸载时接收通知 |

系统 builder 支持 `in_set`、`with_order`、`with_tick_policy`、`before`、`after`。约束应引用稳定的 system/set ID，而不是依靠注册顺序。

## Feature bundle

`PluginFeatureBundleBuilder::new(feature_id, display_name, owner_plugin_id)` 用于子功能。可以添加 primary/required dependency、capability、runtime/editor 模块、默认打包策略和 `enabled_by_default`。例如渲染的 15 个 feature 与网络的 HTTP/RPC/Replication 都由独立 feature crate 注册，而不是把所有依赖强制装入主插件。

```rust
let feature = zircon_plugin_sdk::PluginFeatureBundleBuilder::new(
    "example.telemetry",
    "Telemetry",
    "example",
)
.with_primary_dependency("example", "runtime.plugin.example")
.with_runtime_capability_module(
    "runtime.feature.example.telemetry",
    "example.telemetry.runtime",
    "zircon_plugin_example_telemetry_runtime",
    [
        zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
    ],
)
.enabled_by_default(false)
.build();
```

实际参数以 `feature_bundle_builder.rs` 为准；还可用 `with_dependency`、`with_required_dependency`、`with_runtime_module_from_builder` 和 `with_editor_module_from_builder` 精确描述模块。

## Manifest builder

`PluginManifestBuilder` 构建整个 `PluginPackageManifest`；`PluginModuleBuilder::{runtime, editor, native, vm}` 构建模块条目。模块可追加 description、init level、模块依赖、target modes、capabilities、system sets 和 anchors。`ImporterRuntimeManifestBuilder` 专门统一 importer 的 runtime/dist 模块、distribution、平台及目标，详见[资产导入器](asset-importers.md)。

## 编辑器插件

`EditorPluginDeclaration::new` 包装 `EditorPluginDescriptor`，支持 category、package role、description、maturity、capabilities、runtime event consumer、运行时 manifest 镜像、asset/content root。`registration_report(&dyn EditorPlugin)` 把描述符、manifest 与插件贡献合并成可安装报告。

`EditorContributionBuilder` 支持 `view`、`drawer`、`menu`、`command`、`command_with_execution_contract`、`asset_type`、`settings_page`、`localization_bundle` 和 `tool_resource_kind`，最终 `build()` 得到 `SerializedContributionBatch`。ID 应由插件命名空间前缀限定，避免不同包覆盖同名视图或命令。

## 接口与卸载安全

插件间 linked 调用通过 `PluginInterface` 和 `BridgeImport`：接口 trait 的 `INTERFACE_ID` 是版本化契约；导出值用 `Arc` 持有。导入方必须处理 `BridgeError::Absent` 以及 owner 撤销后的失败。不要把宿主生命周期之外的裸引用保存在插件静态变量中。

## SDK 测试运行时

启用 `runtime` 后，`TestRuntimeBuilder` 可装入一个或多个 `RuntimePlugin` 或注册报告，选择基础模块、固定步长、最大 fixed steps，并创建默认 level。它适合验证注册、模块激活、系统执行和 manager 解析；native ABI 仍应以 fixture 和宿主装载测试覆盖。
