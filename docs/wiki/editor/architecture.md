---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/module.rs
implementation_files:
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/module.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_editor/src/tests
  - zircon_runtime/src/tests/extensions/tech_stack_dependency_guard.rs
doc_type: module-detail
---

# 编辑器架构与功能状态

## 系统定位

`zircon_editor` 是固定三包架构中的编辑器作者态包：

- `zircon_app` 选择运行 profile、装配模块并持有主循环。
- `zircon_runtime` 持有生命周期内核、manager/service、运行时世界、图形、资源和共享 UI 能力。
- `zircon_editor` 持有作者态会话、编辑工具、命令历史和桌面宿主。

编辑器模块以 `InitLevel::Editor` 注册，依赖 Foundation、Asset、Scene、Graphics 与 UI 模块。`EditorHostDriver` 立即启动；`EditorManager`、`EditorAssetManager`、命令注册表和 keymap 采用 lazy manager 方式解析。

## 顶层分层

| 层 | 所有权 | 允许依赖 | 禁止承担 |
| --- | --- | --- | --- |
| `core` | headless-safe 的编辑器状态、服务和协议 | runtime/interface 的稳定合同 | 桌面窗口、像素绘制、控件命中 |
| `scene` | 编辑器场景工具、选择、模式与 viewport 作者态交互 | `core` 与 runtime render/scene DTO | runtime world 的最终所有权 |
| `ui` | 宿主、workbench、绑定、面板与资产编辑器 | `core`、`scene`、共享 UI/runtime | 把按钮 callback 当业务权威 |

这三个目录是结构红线。新功能应先判断它是无界面的作者态服务、场景工具，还是 UI/宿主投影，再进入相应目录。

## `core` 功能矩阵

| 模块 | 功能 | 状态 | 主要入口 |
| --- | --- | --- | --- |
| `asset` | 编辑器资产索引、导入、删除/移动预检、资产类型注册 | 已实现 | `EditorAssetIndex`, `AssetTypeRegistry` |
| `commandlet` | headless 资产迁移、插件列表、作者自动化 | 已实现 | `parse_commandlet_args`, `run_commandlet` |
| `commands` | 命令描述、when 条件、keymap、菜单/命令面板投影、native executor | 已实现 | `EditorCommandRegistry`, `EditorCommandDescriptor` |
| `context` | `EditorContext` 及 tool scheduler 等共享服务 | 已实现 | `EditorContext`, `ToolSchedulerService` |
| `document` | 项目会话和 Scene 文档生命周期、打开/创建/重载路由 | 已实现 | `DocumentLifecycleAuthority`, `SceneDocumentRoute` |
| `editing` | 可撤销命令、事务、历史、journal、作者态 world 路由 | 已实现 | `EditorTransactionEngine`, `EditCommand` |
| `editor_event` | 规范化事件、effect、journal、listener、replay | 已实现 | `EditorEventService`, `EditorEventDispatcher` |
| `editor_extension` / `extension` | view、inspector、操作等扩展贡献 | 可扩展基础 | `EditorExtensionRegistry` 相关描述符 |
| `editor_message` | 文档/焦点/模式/工具/事务主题消息总线 | 已实现 | `SharedEditorMessageBus`, `EditorTopic` |
| `editor_operation` | 菜单、UI、远控、CLI 共用的路径命名操作 | 已实现 | `EditorOperationPath`, `EditorOperationInvocation` |
| `export` | 导出描述与作者态导出合同 | 已实现/受宿主能力限制 | export DTO 与 host pipeline |
| `gateway` | in-process/serialized runtime 抽象 | 已实现 | `EditorRuntimeGateway` |
| `hub_link` | Hub 握手与编辑器启动联动 | 已实现/按配置启用 | `EditorHostRunConfig::with_hub_handshake` |
| `i18n` | 编辑器命令与界面本地化 | 已实现 | `EditorI18nService` |
| `jobs` / `logging` / `notifications` | 后台工作、日志和通知状态 | 已实现 | `EditorContext` 服务句柄 |
| `play` | Play/Simulate 会话、编辑保护、pending edit 与预览帧 | 已实现但 backend 可替换 | `PlaySessionController`, `PlayBackend` |
| `plugin` | 编辑器插件发现、生命周期、贡献物化与 SDK | 已实现 | `EditorPluginManager`, `EditorPlugin` |
| `project` | 创建/打开/预检、manifest 迁移与最近项目 | 已实现 | `ProjectAuthority`, `ProjectLaunchPreflight` |
| `recovery` | durable journal/崩溃恢复编排 | 已实现 | recovery owner types |
| `runtime_event_consumer` | 将 runtime event 泵入编辑器状态 | 已实现 | host 事件泵 |
| `script_build` | 脚本构建任务的作者态管理 | 可扩展基础 | script build DTO/service |
| `settings` | 编辑器设置权威快照 | 已实现 | settings authority/snapshot |
| `sync` | editor/runtime 世界同步 | 已实现 | world watch/invalidation 适配 |
| `tools` | 工具资源声明、租约、公平调度和输入 capture | 已实现 | `ToolSchedulerService` |

## `scene` 功能矩阵

| 模块 | 功能 | 状态 |
| --- | --- | --- |
| `selection` | Edit/Play 域隔离的多选、主选择与 revision | 已实现 |
| `modes` | Select/Transform 基础模式及插件 overlay mode 栈 | 已实现，可扩展 |
| `viewport` | 视口状态、相机交互、renderer picking、Gizmo/handle overlay | 已实现，控制器为内部 API |

## `ui` 功能矩阵

| 模块组 | 功能 | 状态 |
| --- | --- | --- |
| `host` | `EditorManager`、项目/文档/插件/导出/Play 编排 | 已实现 |
| `retained_host` | 原生窗口、事件循环、绘制、命中、callback 适配 | 已实现，内部 API 为主 |
| `workbench` | 页面、drawer、pane、tabs、浮动窗口、布局、快照 | 已实现 |
| `binding` / `binding_dispatch` | typed UI command 到 editor event/intent | 已实现 |
| `asset_editor` | `.zui`/UI v2 的 source/design、预览、样式、undo/replay | 已实现，部分工具持续扩充 |
| `animation_editor` | sequence/graph/state-machine 会话投影 | 可扩展基础 |
| `material_editor` | 材质与 renderer data 的只读/编辑投影 | 可扩展基础 |
| `activity` | Activity view/window/drawer 描述 | 已实现 |
| `template` / `template_runtime` | editor-only 模板适配和 builtin 注册 | 已实现，部分为内部 API |
| `reflection` | workbench 与控件反射模型 | 已实现 |
| `control`, `curve`, `graph`, `timeline` | 可复用编辑控件/模型 | 基础能力，按具体编辑器接入 |
| `preview_scene` | 资产编辑器预览场景 | 可扩展基础 |
| `settings` / `preferences` | 设置窗口与偏好投影 | 已实现 |
| `layouts`, `widgets`, `v2_design_tokens` | 内置布局与视觉组件 | 内部实现 |

## 关键数据流

### 作者操作

```text
native pointer / keyboard / menu
  -> typed binding or route intent
  -> EditorEvent normalization
  -> EditorManager / EditorTransactionEngine
  -> authoring state mutation
  -> effects + dirty/invalidation
  -> workbench snapshot rebuild
  -> retained host presentation
```

### Runtime 交互

```text
EditorManager
  -> EditorRuntimeGatewayHandle lease
  -> capability check
  -> query/watch/operation/frame/pick API
  -> runtime result or invalidation
  -> editor projection, never runtime ownership transfer
```

## 扩展原则

新增编辑器功能应提供 typed descriptor、operation、event 或 command，不应直接在 UI callback 中修改业务状态。高频输入路径使用 typed enum/DTO；字符串仅作为持久化、远控或调试协议。需要改变 runtime world 的操作必须绑定完整 `GatewaySessionIdentity`，防止 gateway 替换后继续使用旧 opaque handle。

## 当前限制

- 大量宿主与 Scene viewport 控制器仍为 `pub(crate)`，第三方插件应使用 plugin SDK、命令贡献和 view descriptor，而不是直接调用宿主实现。
- UI/动画/材质编辑器成熟度不相同；存在类型和 pane 不表示所有按钮都已经连接到完整生产 pipeline。
- in-process gateway 可借用 `World`，serialized gateway 只能使用 query/operation 等 ABI 能力。
- `integration-contracts` 暴露的浮动窗口类型用于验证，不是默认生产公共面。
