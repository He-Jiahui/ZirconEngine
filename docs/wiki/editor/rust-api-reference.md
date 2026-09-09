---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
implementation_files:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
tests:
  - zircon_editor/src/tests
  - zircon_editor/tests
doc_type: module-detail
---

# Rust API 索引

本页按使用任务列出当前公开入口。它是导航表，不替代 rustdoc；方法签名以源码为准。优先从 `zircon_editor` crate root 导入，只有 root 未导出时才使用 owner module 路径。

## Crate root

| API | 用途 | 稳定性 |
| --- | --- | --- |
| `run_editor` | 使用默认配置和指定 runtime gateway 启动 retained editor | 公共入口 |
| `run_editor_with_startup_request` | 指定 GUI startup request | 公共入口 |
| `run_editor_with_config` | 完整宿主配置 | 公共入口 |
| `run_retained_host_automation` | retained host 自动化 | 公共入口 |
| `EditorHostRunConfig` | 启动、布局、插件、Play backend、截图配置 | 公共入口 |
| `EditorModule`, `module_descriptor` | runtime module 装配 | 公共入口 |
| `EditorManager` | 编辑器作者态协调服务 | 公共入口，建议 resolver 获取 |
| `EditorRuntimeGateway*` | runtime 连接与替换 | 公共入口 |
| `EditorCommand*`, `EditorKeymap*`, `WhenClause` | 命令系统 | 公共入口 |
| `EditorPlugin`, `EditorPluginDescriptor` | native 编辑器插件合同 | 公共入口 |
| `EditorIntent` | 内部行为语义的公开 intent 类型 | 公共但演进中 |

## 启动与模块

```rust
use zircon_editor::{
    module_descriptor, run_editor_with_config, EditorHostRunConfig,
    EDITOR_MANAGER_NAME, EDITOR_MODULE_NAME,
};
```

Service name 常量用于 runtime resolver；不要手写等价字符串。`EditorModule` 实现 runtime `EngineModule`。

## 命令

```rust
use zircon_editor::{
    CommandEvalCtx, EditorCommandAction, EditorCommandDescriptor,
    EditorCommandExecutorRegistry, EditorCommandRegistry,
    EditorKeyChord, EditorKeymap, WhenClause,
};
use zircon_editor::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath,
};
```

常用方法：

| 类型 | 方法 |
| --- | --- |
| `EditorOperationPath` | `parse`, `as_str` |
| `EditorOperationInvocation` | `new`, `parse`, `with_arguments`, `with_operation_group` |
| `EditorCommandDescriptor` | `operation`, `native`, `localized_operation`, `with_*`, `is_enabled` |
| `EditorCommandRegistry` | `new`, `default_workbench`, `register`, `register_operation` |
| `EditorKeyChord` | parse/格式化快捷键 |
| `EditorKeymap` | 注册/解析 binding、冲突报告 |

## 事务与历史

路径：`zircon_editor::core::editing::engine`。

| 类型 | 用途 |
| --- | --- |
| `EditCommand`, `EditContext` | 可撤销命令与执行上下文合同 |
| `EditorTransactionEngine` | begin/undo/redo/history/save token |
| `TransactionScope` | push、merge、participant、commit/cancel |
| `HistoryContextId` | Global/Document/PlaySession 历史域 |
| `HistoryStatus`, `HistoryDetailPage` | 历史和 dirty UI 投影 |
| `EditCommandCodecRegistry` | journal command codec |
| `DurableJournal`, `JournalWriter` | 崩溃恢复日志 |

`TransactionScope` 不是 `Send`，应在创建它的同步作者态流程中完成。

## 项目与文档

```rust
use zircon_editor::core::project::{
    NewProjectDraft, ProjectAuthority, ProjectLaunchPreflight,
    SceneCreateRequest, SceneOpenRequest,
};
use zircon_editor::core::document::{
    DocumentLifecycleAuthority, SceneDocumentRoute,
    SceneDocumentReloadCoordinator,
};
```

| 类型 | 职责 |
| --- | --- |
| `ProjectAuthority` | 文件系统项目创建/打开权威 |
| `ProjectProbe` / `ProjectLaunchPreflight` | 打开前检查 |
| `OpenedProject` / `CreatedProject` | 规范化项目结果 |
| `DocumentLifecycleAuthority` | project session 与 scene activation |
| `SceneDocumentRoute` | Scene 创建/打开并安装 authoring world |
| `SceneDocumentReloadCoordinator` | 外部变化重载 |

## Scene 与选择

```rust
use zircon_editor::scene::selection::{
    SelectionModel, SelectionMutation, WorldDomain,
};
use zircon_editor::scene::modes::{
    EditorSceneMode, SceneModeActivation, SceneModeRegistry, SceneModeStack,
};
use zircon_editor::scene::viewport::{
    GridMode, SceneViewportSettings, TransformHandleKind,
    TransformSpace, ViewOrientation, ViewportInput, ViewportState,
};
```

`SceneViewportController` 和 handle registry 当前是 crate-private；扩展通过 Scene mode、typed command 和 plugin contribution 接入。

## Workbench 与视图

```rust
use zircon_editor::ui::workbench::layout::{
    LayoutCommand, LayoutManager, WorkbenchLayout,
};
use zircon_editor::ui::workbench::view::{
    ViewDescriptor, ViewRegistry, ViewInstance,
};
use zircon_editor::ui::workbench::window_registry::EditorWindowRegistry;
```

| 类型 | 职责 |
| --- | --- |
| `ViewDescriptor` | view 类型、模板、payload、dock 策略 |
| `ViewRegistry` | descriptor/instance 单一注册表 |
| `WorkbenchLayout` | 页面、drawer、document tree、floating window |
| `LayoutCommand` / `LayoutManager` | 结构化布局变更 |
| `EditorWindowRegistry` | window/drawer view/drawer window |
| `LayoutPreset*` | 用户/项目 preset 持久化 |

## UI Binding

```rust
use zircon_editor::ui::binding::{
    AnimationCommand, AssetCommand, DockCommand, DraftCommand,
    EditorUiBinding, EditorUiRouter, SelectionCommand,
    ViewportCommand, WelcomeCommand,
};
use zircon_editor::ui::binding_dispatch::{
    dispatch_asset_binding, dispatch_docking_binding,
    dispatch_selection_binding, dispatch_viewport_binding,
};
```

外部模板/插件应生成 typed binding payload，不应调用 `retained_host::callback_dispatch` 内部函数。

## UI Asset Editor

路径：`zircon_editor::ui::asset_editor`。

| 类型 | 用途 |
| --- | --- |
| `UiAssetEditorSession` | UI 资产作者态权威 |
| `UiAssetEditorRoute` | session/document 路由 |
| `UiAssetEditorMode` | source/design 模式 |
| `UiAssetEditorCommand` | 结构化作者命令 |
| `UiAssetEditorUndoStack` | session-local undo/redo |
| `UiAssetPreviewHost` | 编译并承载预览 surface |
| `UiAssetEditorPanePresentation` | Workbench pane 投影 |
| `UiAssetEditorCommandJournal` | replay 记录 |
| `UiAssetEditorBugReportReplayArtifact` | 可移植问题复现数据 |

## Animation 与 Material

```rust
use zircon_editor::ui::animation_editor::{
    AnimationEditorSession, AnimationEditorCapabilityDescriptor,
};
use zircon_editor::ui::material_editor::{
    MaterialEditorProjection, RendererDataEditorProjection,
};
```

这些模块公开的是会话/能力/投影基础。调用具体编辑动作前检查 capability，不要把 projection 类型视为完整 editor toolkit。

## Gateway

```rust
use zircon_editor::{
    DetachedEditorRuntimeGateway, EditorRuntimeGateway,
    EditorRuntimeGatewayHandle, InProcessGateway,
    RuntimeCapabilities, SessionGateway, SharedEditorRuntimeGateway,
};
use zircon_editor::core::gateway::{
    EditorRuntimeOperationRoute, EditorRuntimeViewportPickRoute,
};
```

| 方法组 | 说明 |
| --- | --- |
| `capabilities`, `session_identity` | 能力与完整 endpoint 身份 |
| `with_world(_mut)` | 仅 in-process 借用 |
| `query/watch/unwatch/drain_world_invalidations` | serialized-safe 世界同步 |
| `tick_frame`, `handle_event` | runtime 帧循环 |
| `bind/unbind/present/capture_frame` | viewport presentation |
| `submit_highlight_set` | 编辑器选择高亮 |
| `request/poll/cancel_viewport_pick` | renderer picking |
| `submit/poll/harvest_operation` | runtime 长操作 |
| `profile_control`, plugin event API | 诊断与插件事件 |

## 插件 SDK

路径：`zircon_editor::core::plugin::sdk`。SDK 重导出资产类型、命令、操作、view/inspector 描述符和插件生命周期类型。典型扩展顺序：

1. 实现 `EditorPlugin` 并声明 `EditorPluginDescriptor`。
2. 在注册阶段贡献 commands、views、asset types、inspector customization。
3. 让 manager 物化贡献并返回 registration report。
4. 卸载时按 contribution ticket 退休所有注册项。

插件不得长期保存 manager 内部锁、`&World`、native window 指针或 crate-private controller。

## 可见性与 Feature

- `pub(crate)`：仅 `zircon_editor` 内部，不是集成 API。
- `#[cfg(test)]`：只用于单元测试。
- `integration-contracts`：为跨 crate 验证开放的结构，不保证普通构建存在。
- Root re-export：优先公共面，但仍应查看错误类型和 capability。
- Owner module 的 `pub`：仓库级可见，稳定性低于 root，硬切迁移时路径可能改变。

## 错误处理惯例

编辑器 API 大量使用 domain error enum。上层应保留错误类别并投影 diagnostics，不要把所有错误降为字符串。尤其需要分别处理：命令 disabled、事务 rollback failure、stale gateway generation、session lost、capability missing、protocol violation 和 dirty save 冲突。
