---
related_code:
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/plugin_sdk/src
implementation_files:
  - zircon_plugins/plugin_sdk/src
plan_sources:
  - user: 2026-09-09 API 公开接口覆盖审计
tests:
  - zircon_plugins/plugin_sdk/src/declaration/tests.rs
doc_type: api-reference
---

# Plugin SDK API

`zircon_plugin_sdk` 将插件声明、manifest、runtime registration、editor contribution 和 native ABI 组合成可发布包。所有子模块均受 feature gate 控制；不要在未启用 feature 时引用对应 re-export。

## Feature 与模块矩阵

| feature | 模块 | 公开能力 |
| --- | --- | --- |
| `native` | `zircon_plugin_sdk::dist` | 插件 dist 模块的构建器、声明或测试支持。 |
| `editor` | `zircon_plugin_sdk::editor` | 插件 editor 模块的构建器、声明或测试支持。 |
| `editor_contribution` | `zircon_plugin_sdk::editor_contribution` | 插件 editor_contribution 模块的构建器、声明或测试支持。 |
| `runtime` | `zircon_plugin_sdk::manifest` | 插件 manifest 模块的构建器、声明或测试支持。 |
| `native` | `zircon_plugin_sdk::native` | 插件 native 模块的构建器、声明或测试支持。 |
| `runtime` | `zircon_plugin_sdk::prelude` | 插件 prelude 模块的构建器、声明或测试支持。 |
| `runtime` | `zircon_plugin_sdk::registration` | 插件 registration 模块的构建器、声明或测试支持。 |
| `runtime` | `zircon_plugin_sdk::runtime` | 插件 runtime 模块的构建器、声明或测试支持。 |
| `runtime` | `zircon_plugin_sdk::test` | 插件 test 模块的构建器、声明或测试支持。 |

## Crate-root re-export

| 符号 | 典型用途 |
| --- | --- |
| `zircon_plugin_sdk::BridgeError` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::BridgeImport` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::default_export_packaging` | 插件 SDK 的版本、平台、ABI 或角色参数。 |
| `zircon_plugin_sdk::default_supported_platforms` | 插件 SDK 的版本、平台、ABI 或角色参数。 |
| `zircon_plugin_sdk::EditorContributionBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::EditorPluginDeclaration` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::importer_runtime_supported_platforms` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::importer_runtime_supported_targets` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::ImporterRuntimeManifestBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::NATIVE_ABI_VERSION_V3` | 插件 SDK 的版本、平台、ABI 或角色参数。 |
| `zircon_plugin_sdk::NATIVE_DESCRIPTOR_SYMBOL_V3` | 插件 SDK 的版本、平台、ABI 或角色参数。 |
| `zircon_plugin_sdk::NativePluginEntryDeclaration` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginCapabilityRole` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginDeclaration` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginFeatureBundleBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginInterface` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginManifestBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginMaturityLevel` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginModuleBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginPackageRole` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginPackaging` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginPlatform` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::PluginTarget` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::RuntimePluginDeclaration` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::RuntimePluginModuleRegistration` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::RuntimePluginRegistrationBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::RuntimePluginRuntimeSceneSystemBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::SDK_API_VERSION` | 插件 SDK 的版本、平台、ABI 或角色参数。 |
| `zircon_plugin_sdk::TestRuntime` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::TestRuntimeBaseModule` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::TestRuntimeBuilder` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::TestRuntimeError` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |
| `zircon_plugin_sdk::WeakBridge` | 构建插件声明、运行时注册、编辑器贡献或测试 harness。 |

## 最小调用形状

```rust
use zircon_plugin_sdk::{PluginManifestBuilder, RuntimePluginRegistrationBuilder};

// manifest builder 的 build() 是无失败的纯值转换。
let manifest = PluginManifestBuilder::new("my.plugin", "My Plugin")
    .with_category("runtime")
    .with_description("Example runtime plugin")
    .with_capability("runtime.plugin.my_plugin")
    .build();

// registration builder 借用宿主 registry；module() 失败时返回结构化错误。
let mut registration = RuntimePluginRegistrationBuilder::new(&mut registry);
let mut module = registration.module("my_plugin.runtime")?;
```

`PluginDeclaration::new(...)` 是声明数据的 const 构造器，需要完整提供 id、显示名、分类、模块、targets、platforms、capabilities、角色、成熟度和 packaging 数组；生产插件通常通过 SDK 的 `declare_plugin!` 宏生成它。manifest 中的 ABI、target、platform 与 feature bundle 必须和宿主能力集合一致。

## 兼容性规则

- `SDK_API_VERSION`、`NATIVE_ABI_VERSION_V3` 与 descriptor symbol 是版本门；升级时不得只改一处。
- runtime/editor/native 三种产物分别由 feature 控制，发布包应检查每种产物是否生成。
- `TestRuntime` 仅用于测试和示例，不应作为生产宿主。
- `BridgeError`、`TestRuntimeError` 等错误必须在 CI 中被断言，避免静默降级。

## 审计命令

```powershell
rg --no-heading --line-number "^pub use|^pub mod|^pub trait|^pub struct|^pub enum|^pub type|^pub const|^pub fn" zircon_plugins/plugin_sdk/src
cargo rustdoc -p zircon_plugin_sdk --all-features
```
