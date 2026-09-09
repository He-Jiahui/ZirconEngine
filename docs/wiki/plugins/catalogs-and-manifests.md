---
related_code:
  - zircon_plugins/plugin_sdk/src/declaration.rs
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
implementation_files:
  - zircon_plugins/plugin_sdk/src/manifest
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_runtime/src/plugin
doc_type: architecture-reference
title: 清单与第一方插件目录
status: source-audited
---

# 清单与第一方插件目录

Zircon 的 manifest 回答“包声明了什么”，project selection 回答“项目要求什么”，catalog 回答“当前二进制链接了哪个实现”。三者缺一不可：清单里存在一个插件，不代表当前应用编译进了它；编译进 catalog，也不代表当前项目选择或目标允许激活它。

## Package manifest

`PluginPackageManifest` 的核心字段包含 ID、显示名、描述、类别、SDK API、成熟度、package role、目标、平台、能力、默认打包、根目录、依赖、模块、能力状态、可选 feature、接口、选项、事件目录、组件、importer 与 distribution。生成的 `plugin.toml` 是这份 Rust 数据的序列化投影。

典型模块条目包含 `name`、`kind`（runtime/editor/native/vm）、`crate_name`、target modes、capabilities、init level、模块依赖、system sets 和 anchors。包级支持目标不能替代模块级目标：一个 runtime+editor 包可以支持 editor host，但其 editor 模块仍不应进入 client runtime。

## Project selection

项目清单用插件 ID 选择包和 feature。`resolve_plugin_selections(target_mode, manifest, provider)` 负责把选择解析成 registration report，同时产生未解析/不适用项的报告。调用方必须使用 resolution report，而不是丢弃诊断后只取成功项。

选择发生在 provider 调用之前，因此流程是：

```text
ProjectPluginManifest
  -> target_mode 过滤
  -> plugin / feature 依赖与能力解析
  -> catalog provider 查找 linked 实现
  -> RuntimePluginRegistrationReport 或 EditorPluginRegistrationReport
  -> 扩展注册表与模块激活
```

## Runtime catalog

`zircon_first_party_runtime_catalog` 是 linked 第一方实现的集中 fan-out。入口为：

```rust
first_party_runtime_plugin_registrations_for_manifest(target_mode, manifest)
first_party_registration_for_runtime_plugin(runtime_id)
```

它没有默认 features。构建应用必须显式选择集合：

| Cargo feature | 当前映射 |
|---|---|
| `base-runtime-plugins` | AI、Animation、glTF/OBJ/WGSL importer、Net、Particles、Rendering、Sound、Texture |
| `advanced-render-runtime-plugins` | Hybrid GI、Neural、Solari、Virtual Geometry |
| `navigation-runtime-plugin` | Navigation |
| `ui-document-importer` | UI Document Importer |
| `zr-vm-language-runtime-plugin` | ZrVM Language |
| `backend-zr-vm` | ZrVM 插件并启用其真实 backend feature |

`base-runtime-plugins` 是编译分组，不是成熟度保证：其中包含 experimental/partial 的 AI 与 Particles。产品 profile 仍须用 project manifest 选择实际插件。

## Editor catalog

`zircon_first_party_editor_catalog` 只在 `EditorHost` 目标解析 provider；其他目标直接返回空报告。当前显式 Cargo features 是 `navigation-editor-plugin` 与 `neural-editor-plugin`，分别映射对应 editor registration。入口为：

```rust
first_party_editor_plugin_registrations_for_manifest(target_mode, manifest)
first_party_registration_for_editor_plugin(plugin_id)
```

其范围比 `zircon_plugins` 中全部 editor crate 更窄。这表示其他 editor 包虽有源码/manifest/dist，也未必被该 linked catalog feature 集合直接提供；应用可通过其他组合层或 native 分发接入。

## Native catalog 与 distribution

native 动态包不需要 linked provider 函数，但需要 `[distribution]`：forms、default packaging、ABI、engine compatibility、dist crate、descriptor symbol，以及 runtime/editor entry。宿主发现动态库后把 descriptor 内 manifest 与外部 catalog 信息交叉校验，不能只信文件名。

## 更新规则

新增或重命名插件时应从 Rust `PluginDeclaration` 修改单一来源，重新投影 `plugin.toml`，再按需要更新 runtime/editor catalog feature 与 provider 映射。`@cargo-zircon:*registration-begin/end` 标记的区段是 catalog 映射边界；遗漏映射会导致“清单可见但 linked provider 不存在”。删除插件时反向检查项目 profile、feature dependency、catalog 映射和 dist 制品，避免留下幽灵选择项。

## 冲突与诊断

同一插件 ID 只能有一个最终 provider。同一个 importer 后缀可能有多个插件候选，但要由 priority 和能力选择明确解决。linked 与 native 同时提供同一 ID 时，组合层必须定义优先级并拒绝重复注册；不要让后注册者覆盖先注册者，因为 owner 撤销和接口导出会变得不确定。
