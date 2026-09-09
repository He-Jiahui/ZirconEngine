---
related_code:
  - zircon_plugins/Cargo.toml
  - zircon_plugins/plugin_sdk/src/lib.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
implementation_files:
  - zircon_plugins/plugin_sdk/src
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_runtime/src/plugin
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/first_party_runtime_catalog/src
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_runtime/src/plugin
doc_type: module-index
title: 插件系统
status: source-audited
---

# ZirconEngine 插件系统

ZirconEngine 将可选功能从核心运行时中拆成独立的 `zircon_plugins` workspace。它既包含游戏运行时插件，也包含编辑器扩展、资产导入器、可选 feature bundle、原生动态库分发包以及 SDK 示例。插件不是简单的 Cargo feature：引擎会读取包清单、选择目标与平台、协商能力、注册模块/系统/资源/接口，并在 native 模式下通过稳定 C ABI 装载动态库。

> 本文档描述当前源码，而不是未来路线图。`maturity = "stable"` 表示包契约的成熟度；某项 `capability_status = "partial"` 仍意味着该具体能力尚未完整。两者必须同时阅读。

## 阅读路线

| 目标 | 文档 |
|---|---|
| 查全部包、成熟度和分发形式 | [插件清单](inventory.md) |
| 编写 Rust 插件或理解声明宏 | [Plugin SDK 与 Rust API](plugin-sdk.md) |
| 理解 manifest、项目选择和 linked catalog | [清单与第一方目录](catalogs-and-manifests.md) |
| 理解 linked/native/dist 与 ABI | [原生 ABI 与分发](native-abi-and-distribution.md) |
| 理解发现、协商、注册、卸载 | [生命周期与能力协商](lifecycle-and-capabilities.md) |
| 接入模型、纹理、音频、Shader、UI 导入 | [资产导入器](asset-importers.md) |
| 使用 AI、物理、网络、声音等 | [运行时插件族](runtime-families.md) |
| 使用渲染 feature、神经推理、虚拟几何 | [渲染与高级图形插件族](rendering-families.md) |
| 使用编辑器创作与 UI 工具 | [编辑器、平台与 UI 插件族](editor-and-ui-families.md) |

## Workspace 结构

一个完整插件通常由以下目录组成：

```text
zircon_plugins/<plugin>/
├── plugin.toml          # 由 Rust PluginDeclaration 投影生成的包清单
├── runtime/             # linked Rust 实现与运行时注册
├── editor/              # 编辑器扩展；只面向 editor_host
├── dist/                # cdylib，导出 native ABI v3 入口
└── features/<name>/     # 可独立选择的 feature bundle
```

`runtime` 是 Rust 行为实现的主要所有者；`editor` 声明面板、视图、命令、资产类型和创作能力；`dist` 不应再复制业务元数据，而是把同一声明投影成 native descriptor、entry report 和行为回调。少数包只有其中一部分，例如 `timeline_sequence` 是编辑器专用，`navigation/native` 是 Recast/Detour 后端而非通用 dist 外壳。

## 三种交付形式

| 形式 | 清单值 | 用途 | 边界 |
|---|---|---|---|
| 源码模板 | `source_template` | 将插件源码纳入项目构建 | 可访问完整 Rust 类型，构建耦合最高 |
| 链接库 | `library_embed` | 以 Rust 依赖静态/动态链接到宿主 | 通过 `RuntimePlugin`、注册报告和 bridge 工作 |
| 原生动态插件 | `native_dynamic` | 运行时装载 `dist` 生成的 `cdylib` | 仅使用 ABI v3/v4 数据结构、清单和函数指针 |

本文档把前两种源码内集成统称为 **linked**。`dist` 指生成动态库的包装 crate；它是 native 分发物的构建载体，不是第四种运行形态。

## 元数据单一来源

`declare_plugin!` 声明 ID、显示名、类别、模块、目标、平台、能力、成熟度与默认打包策略。SDK 从这份声明派生：

1. linked `RuntimePluginDescriptor`；
2. `PluginPackageManifest` / `plugin.toml`；
3. native descriptor 中的插件 ID、入口名和请求能力；
4. dist crate 的 registration manifest。

因此 `plugin.toml` 文件头标有 `@generated` 时不应手工修改。行为注册仍留在插件 crate 内，例如系统工厂、组件描述符、bridge 实现和 importer 回调；单一来源只约束跨形态必须一致的包元数据。

## 目标与平台

声明目标为 `client_runtime`、`server_runtime`、`editor_host`。平台枚举还支持 Windows、Linux、macOS、Android、iOS、WebGPU、Wasm 与 Headless；当前第一方 `plugin.toml` 主要发布 Windows/Linux/macOS。编辑器模块必须只声明 `editor_host`。服务端需要显式包含 `server_runtime`，不能从“运行时插件”名称推断。

## 插件选择原则

- 先以 `id` 和 capability 选择功能，不要依赖 crate 名称。
- 将 `maturity` 用于产品准入，将每项 `capability_status` 用于功能准入。
- 可选 feature bundle 必须同时满足主插件依赖、所需插件依赖和宿主授予能力。
- `native_dynamic` 只能调用 ABI 暴露的命令、事件、注册表和 bridge 方法；不能跨库传递任意 Rust trait object。
- 编辑器镜像运行时清单，但编辑器贡献与运行时行为仍分别注册。

## 当前稳定边界

源码中成熟度为 stable 的核心包包括 `rendering`、`texture` 及多种专用 importer；但许多 importer 的具体解码能力仍标为 partial。`animation`、`navigation`、`net`、`sound`、`terrain` 等是 beta 或 partial。`ai`、`physics`、`neural`、`particles`、高级渲染插件与多数 authoring 工具仍为 experimental。生产项目应固定版本、固定 capability 集合，并在启动报告中拒绝缺少必需能力的配置。
