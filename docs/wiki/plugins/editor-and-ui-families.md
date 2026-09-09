---
related_code:
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_plugins/first_party_editor_catalog/src/lib.rs
  - zircon_plugins/material_editor/editor/src
  - zircon_plugins/animation_graph/editor/src
  - zircon_plugins/timeline_sequence/editor/src
  - zircon_plugins/ui_asset_authoring/editor/src
  - zircon_plugins/native_window_hosting/editor/src
  - zircon_plugins/editor_build_export_desktop/editor/src
implementation_files:
  - zircon_plugins/plugin_sdk/src/editor.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_plugins/material_editor/editor/src
  - zircon_plugins/ui_asset_authoring/editor/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_plugins/01
tests:
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_plugins/editor_contribution_fixture
  - zircon_editor/src
doc_type: module-detail
title: 编辑器、平台与 UI 插件族
status: source-audited
---

# 编辑器、平台与 UI 插件族

编辑器插件面向 `editor_host`，通过 `EditorPluginDeclaration` 生成描述符与清单，通过 `EditorContributionBuilder` 注册 view、drawer、menu、command、asset type、settings page、localization bundle 和 tool resource kind。很多包的 runtime 能力已经较丰富，而 authoring UI 仍为 experimental；两者需要分别判断。

## Editor contribution 模型

贡献不是直接修改编辑器全局状态，而是构建序列化 batch。每项贡献由 package ID 拥有，卸载 package 时可以统一撤销。`command_with_execution_contract` 还可声明命令的执行边界；view/menu/asset type 的 ID 应使用插件命名空间。

编辑器插件可用 `mirrors_runtime(&RuntimePluginDeclaration)` 或 `mirrors_runtime_manifest(manifest)` 引入同包运行时信息。镜像用于展示能力、依赖和目标，不会把 runtime 注册回调搬进 editor。通过 `with_runtime_event_consumer_registration` 可声明对运行时事件的消费。

## 通用 Runtime + Editor 包

`ai` editor 提供行为树与 perception debug 相关 UI、overlay 和 runtime mirror。`navigation` editor 提供 bake panel、viewport overlay、资产/组件/operation/template 注册，并消费运行时 overlay frame。`physics` editor 提供物理组件与调试/创作贡献。`sound` editor 面向声音资产、组件、bus/event 等创作。`particles` editor 面向 emitter/module authoring。`terrain`、`tilemap_2d`、`texture` 和 `virtual_geometry` 均有对应 editor crate。

这些编辑器模块的存在不自动意味着所有创作面板完整。应结合包成熟度与 contribution 内容判断；terrain/tilemap 等 beta 包当前更接近已建立插件边界和基础 authoring 流程。

## 专用 authoring 包

| 包 | 依赖 | 功能与状态 |
|---|---|---|
| `material_editor` | 渲染/材质上下文 | experimental，材质资产编辑器贡献 |
| `animation_graph` | 必需 `runtime.plugin.animation` | experimental，动画图与状态机 editor-only authoring |
| `timeline_sequence` | 必需 timeline event track capability | experimental，序列时间线 editor-only authoring |
| `prefab_tools` | runtime + editor | beta/partial，Prefab 工作流与工具 |
| `ui_asset_authoring` | UI 文档/编辑器上下文 | experimental，retained UI 资产文档创作 |

`animation_graph` 和 `timeline_sequence` 没有 runtime crate；运行结果由 animation runtime 消费。它们的 dist 只导出 editor entry。把 editor-only 包塞进客户端导出既无效，也会扩大制品。

## UI 资产链

UI 功能分为两个责任：

1. `ui_asset_authoring` 在编辑器中创建和修改 retained UI 文档；
2. `ui_document_importer` 把 `.zui` 导入为 `UiWidget`，并可产生 `UiLayout` 与 `UiStyle` 附属资产。

authoring 包为 experimental；importer 包成熟度 stable，但具体 UI document capability 为 partial。编辑器保存成功不应被视为运行时必然可导入，保存后仍需执行 importer 并展示 schema/验证诊断。

## Runtime Diagnostics

`runtime_diagnostics` 是 experimental editor-only 包，用于把 runtime diagnostics 暴露为编辑器扩展。它通过 editor capability 安装，不向游戏运行时提供 manager。诊断采集本身由各 runtime 插件完成，例如 net 记录连接/字节/事件路径，physics 记录 step duration。

## Native Window Hosting

`native_window_hosting` 是 experimental platform/editor 包，负责编辑器内原生窗口承载能力。它只面向 editor host；窗口句柄与消息循环所有权必须由宿主定义，插件卸载前应移除子窗口和回调。该包不是游戏 runtime 的通用窗口模块。

## Desktop Build Export

`editor_build_export_desktop` 是 experimental 平台插件，向编辑器贡献桌面构建/导出工作流。它应根据目标平台、插件 package manifest 和 default packaging 策略组装制品。选择 native dynamic 时还必须携带 dist 动态库与清单；选择 library embed/source template 时则进入项目构建图。

## 示例与 fixture

`plugin_sdk_examples` 是 sample/developer documentation 性质的 editor/dist 包，用于演示 SDK 贡献，不应进入生产默认 catalog。`editor_contribution_fixture` 和 `native_dynamic_fixture` 是 test fixture，分别验证 native editor contribution 和 runtime/editor ABI；它们即便能够加载，也不代表真实产品功能。

## 插件作者的编辑器入口

典型流程如下：

```rust
use zircon_plugin_sdk::{EditorContributionBuilder, EditorPluginDeclaration};

let declaration = EditorPluginDeclaration::new(
    "example_tools",
    "Example Tools",
    "zircon_plugin_example_tools_editor",
)
.with_category("authoring")
.with_capability("editor.extension.example_tools")
.with_content_root("editor");

let contributions = EditorContributionBuilder::new("example_tools")
    .drawer("example_tools.inspector", "Example Inspector")
    .command("example_tools.rebuild", "Rebuild", "example.rebuild")
    .build()?;
```

构造器的精确参数以 `editor.rs` 和 `editor_contribution.rs` 为准。插件还需实现编辑器要求的 `EditorPlugin` 行为并用 declaration 生成 registration report；只构建 batch 不会自动安装贡献。

## 与运行时的通信

编辑器应通过 runtime event consumer、bridge interface 或受控命令与运行时通信。不要共享可变全局对象。debug overlay 适合消费不可变 snapshot；authoring command 应产生可撤销的编辑器 operation；需要运行时重建的资产通过导入/编译管线提交，而不是从 UI 线程直接修改运行时内部缓存。

## 分发和准入

editor-only dist 使用 `native_dist_editor_plugin_v3!`，descriptor 的 runtime entry 为空。宿主仍会做平台、API、能力和 schema 校验。生产编辑器 catalog 应排除 sample/test fixture，按 package role 和 maturity 过滤，并在缺少 runtime dependency 时禁用相关面板而不是让面板在操作时崩溃。
