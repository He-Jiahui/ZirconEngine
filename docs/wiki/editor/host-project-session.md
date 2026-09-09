---
related_code:
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/core/project/mod.rs
  - zircon_editor/src/core/document/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/core/project/authority/mod.rs
  - zircon_editor/src/core/document/lifecycle.rs
  - zircon_editor/src/core/document/scene_route.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_editor/src/ui/retained_host/app/tests
  - zircon_editor/src/core/project/tests
  - zircon_editor/src/core/document/scene_route_tests.rs
doc_type: workflow-detail
---

# 编辑器宿主、项目与文档会话

## 概览

宿主层把一次桌面编辑器运行组织为四级生命周期：Engine module、原生 retained host、项目 session、文档 activation。项目目录和场景文档不是简单的全局路径；它们都有身份、预检、激活和关闭阶段。

## 启动编辑器

最小入口是 crate root re-export 的 `run_editor`：

```rust
use zircon_editor::{run_editor, SharedEditorRuntimeGateway};
use zircon_runtime::core::CoreHandle;

fn start(
    core: CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
) -> Result<(), Box<dyn std::error::Error>> {
    run_editor(core, runtime_gateway)?;
    Ok(())
}
```

需要指定项目、启动 Scene、布局、插件或自动化行为时使用 `EditorHostRunConfig`：

```rust
use zircon_editor::{run_editor_with_config, EditorHostRunConfig};

let config = EditorHostRunConfig::new()
    .with_startup_layout_preset("default")
    .with_exit_after_first_presented_frame(false);

run_editor_with_config(core, runtime_gateway, config)?;
```

`EditorHostRunConfig` 当前支持：

- `with_startup_request`：传入 GUI 启动请求。
- `with_project_runtime_build_set`：选择项目 runtime build set。
- `with_startup_scene_uri`：项目激活后打开指定 Scene。
- `with_startup_layout_preset`：选择初始工作台布局。
- `with_exit_after_first_presented_frame`：首帧后退出，主要用于自动化和截图。
- `with_first_presented_frame_capture_path`：保存首个 presentation 证据。
- `with_editor_plugin_registrations`：注入 native 编辑器插件注册。
- `with_play_backend`：替换 Play backend。
- `with_hub_handshake`：连接 Zircon Hub 启动握手。

`run_editor_with_startup_request` 是只需覆盖启动请求时的便捷入口；`run_retained_host_automation` 用于无人工交互的 retained-host 自动化。

## Engine module 与服务解析

`ui::host::module::module_descriptor()` 注册以下服务：

| 服务 | 启动方式 | 职责 |
| --- | --- | --- |
| `EditorHostDriver` | Immediate | 标识编辑器宿主驱动 |
| `EditorManager` | Lazy | 作者态总协调器 |
| `EditorAssetManager` | Lazy | 连接 runtime project assets 与 editor jobs/logs |
| `EditorCommandRegistry` | Lazy | 暴露 manager context 中的命令注册表 |
| `EditorKeymap` | Lazy | 暴露 keymap service |

外部模块应通过 runtime manager resolver 和这些稳定服务名解析服务，不应自行构造第二个 `EditorManager`。

## 项目工作流

### 创建项目

`NewProjectDraft` 保存用户输入；`ProjectAuthority` 是文件系统创建/打开的作者态门面。创建不是“一次 mkdir”：实现会做 manifest 与目标目录预检、事务 staging、提交和失败回滚，最终返回 `CreatedProject`。

`CreatedProject`/`OpenedProject` 保留规范化的 `ResolvedProjectPath` 与 manifest summary；调用者应优先使用 `identity()` 和 `summary()`，不要重新从显示路径推导项目身份。

### 打开项目

打开链路是：

```text
输入路径
 -> ProjectProbe
 -> ProjectLaunchPreflight
 -> manifest 版本/迁移决策
 -> ProjectAuthority::open
 -> OpenedProject
 -> EditorManager 激活项目 session
```

`ProjectLaunchPreflight` 用于在真正修改宿主状态之前检查项目类型、manifest、runtime composition 和迁移条件。`ProjectManifestMigrationPlan`/`Decision` 将需要用户确认的迁移与直接打开分离。

### 最近项目

`RecentProjectEntry` 是显示和持久化项，`RecentProjectValidation` 负责判断入口当前是否仍有效。最近列表不是项目权威；点击后仍需重新 probe/preflight。

## 项目 session

`DocumentLifecycleAuthority` 为每个激活项目分配 `ProjectSessionId`。`ProjectSessionActivation` 表示新 session 已建立，而不是项目已经完整渲染。关闭项目时，宿主协调 dirty save、Play 终止、runtime endpoint 退休、文档释放和 workspace 持久化。

应按以下顺序理解 session：

1. 项目预检通过。
2. runtime/asset 依赖准备。
3. lifecycle authority 激活 `ProjectSessionId`。
4. 选择或打开 Scene 文档。
5. workbench 切换到项目模式并建立 pane/view。
6. 关闭时先解决 dirty/pending 工作，再提交 close。

`EditorHostStartupSession` 把 `EditorHostEventController` 与 `EditorStartupSessionDocument` 配对，适合测试或自定义宿主读取启动后的稳定状态。

## Scene 文档

### 打开

`SceneOpenRequest` 描述需要打开的 scene locator。`SceneDocumentRoute::open` 通过 `SceneAssetCatalog` 找到资产，并调用 `AuthoringSceneInstaller` 把 authoring scene 安装到编辑器使用的 runtime level。

返回值可能是：

- 已激活文档及 `SceneDocumentRouteActivation`。
- 需要等待异步选择或加载的结果。
- `SceneDocumentRouteError`，例如项目/资产不存在、安装失败或 lifecycle 绑定失败。

### 创建

`SceneCreateRequest` 描述新场景；route 会先准备资产创建，再通过 catalog/installer 完成注册和激活。创建成功不意味着已保存所有后续编辑；dirty 状态仍由历史保存点决定。

### 激活身份

`ActiveSceneDocumentIdentity` 把当前场景与所属 project session 绑定。异步返回必须在提交前验证 session/activation 身份，避免项目切换后把旧结果装入新项目。

### 重载

`SceneDocumentReloadCoordinator` 与 `ActiveSceneReloader` 协调外部文件变化。重载需要区分未修改文档、dirty 文档以及 activation 已变化三种情况；调用者不应直接覆盖当前 authoring world。

## 保存与恢复

编辑器通过历史 save token 和 dirty snapshot 判断文档是否需要保存。保存应遵循：

1. 捕获 `HistorySaveToken`。
2. 序列化对应 authoring 文档。
3. 原子写入目标。
4. 仅在历史未变化时 `mark_saved_if_unchanged`。

这样可以避免保存过程中又发生编辑，却错误清除 dirty 标记。Durable journal 与 recovery 模块用于崩溃恢复；它不是源资产文件的替代品。

## 状态与限制

- **已实现**：项目创建/打开/预检、Scene 创建/打开/重载、session identity、workspace 保存和关闭协调。
- **受限**：项目启动依赖 runtime build set、插件/资产可用性；预检可能要求显式迁移决策。
- **内部实现**：窗口事件循环、native presenter、启动 callback wiring 与大部分 close phase。
- 项目显示路径不可作为身份键；必须使用 `ResolvedProjectPath` 或 session identity。
- 异步票据必须在完成时重新验证当前 session，不能仅凭 Scene URI 提交。
