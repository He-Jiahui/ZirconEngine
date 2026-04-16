---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/history.rs
  - zircon_editor/src/workbench/layout.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/host/message.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/workbench/snapshot/mod.rs
  - zircon_editor/src/workbench/view/mod.rs
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_resource/src/lib.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/locator.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world.rs
implementation_files:
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/history.rs
  - zircon_editor/src/host/manager.rs
  - zircon_editor/src/workbench/project/mod.rs
  - zircon_editor/src/workbench/snapshot/mod.rs
  - zircon_editor/src/editing/state/mod.rs
  - zircon_editor_ui/src/ui_asset_editor.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/locator.rs
  - zircon_editor/src/workbench/view/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_scene/src/world.rs
plan_sources:
  - user: 2026-04-12 扩展 editor 命令系统到删除节点、改父子层级、重命名和 inspector 字段批量提交
  - user: 2026-04-12 实现 Zircon Editor Workbench Shell V1
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/全系统重构方案.md
  - .cursor/plans/基本路线图.md
tests:
  - zircon_editor_ui/src/tests/activity.rs
  - zircon_editor/src/lib.rs
  - zircon_scene/src/lib.rs
  - cargo test -p zircon_editor_ui --lib --locked tests::activity
  - cargo test -p zircon_editor -- --nocapture
  - cargo test -p zircon_entry -- --nocapture
  - cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
doc_type: category-index
---

# Editor And Tooling

## Purpose

本目录记录编辑器宿主层、workbench 壳以及场景编辑工作流的实现细节，重点覆盖：

- `zircon_editor` 如何把 Slint 宿主组织成 workbench shell
- 布局、view registry、项目 workspace 和配置持久化如何协同
- UI 草稿态如何转换成命令，并在 `zircon_scene::LevelSystem` 所托管的 `World` 上安全执行与 undo/redo

## Documents

- [Editor Workbench Shell](./editor-workbench-shell.md): 混合固定壳 workbench、主 tabs、drawers、document workspace、native floating windows、拖放命中与布局持久化。
- [Editor Command Workflow](./editor-command-workflow.md): editor 命令层、历史栈、inspector 草稿批量提交、删除/改父子/重命名等行为约束。
- [Scene Viewport Gizmo, Handle, And Overlay Pipeline](./scene-viewport-gizmo-handle-overlays.md): Scene 视图的 typed viewport settings、scene render packet、scene gizmo provider、handle overlay、wireframe/preview/grid 分层与测试口径。
- [UI Binding And Reflection Architecture](./ui-binding-reflection-architecture.md): `zircon_ui` / `zircon_editor_ui` / `zircon_input` 边界，nativeBinding、反射树、REPL/网络操控与 headless 回放架构。
- [Editor Template Compatibility Migration](./editor-template-compatibility-migration.md): `zircon_editor_ui` 的 editor-only template catalog/registry/adapter，如何把 shared `UiBindingRef` 收口到 typed `EditorUiBinding`，以及后续把 TOML 模板实例接到 Slint host 的迁移顺序。
- [UI Asset Editor Host Session](./ui-asset-editor-host-session.md): `zircon_editor` 的 `UiAssetEditorSession`、recursive import hydration、canonical source save、Slint pane callbacks，以及 `zircon_asset` 对 `.ui.toml` 三类正式资产的注册与 catalog 接入。
- [UI And Layout / UI Asset Documents And Editor Protocol](../ui-and-layout/ui-asset-documents-and-editor-protocol.md): `zircon_ui::template::asset` 的 `layout/widget/style` 编译链、selector stylesheet、legacy adapter、slot-aware shared bridge，以及 shared asset model 如何移交给 editor asset pipeline 和 host session。
- [UI And Layout / Shared UI Core Foundation](../ui-and-layout/shared-ui-core-foundation.md): 运行时/编辑器共享的 `zircon_ui` 约束类型、retained tree、命中索引、surface/render extract，以及 editor workbench 对共享布局核心的复用边界。
- [UI And Layout / Shared UI Template Runtime](../ui-and-layout/shared-ui-template-runtime.md): shared TOML 模板文档、slot/composite 展开和稳定 binding ref 保留语义，是 editor shell compatibility migration 的共享模板真源。
- [Assets And Rendering / Directory Project Asset Rendering](../assets-and-rendering/directory-project-asset-rendering.md): 目录式项目根、`ResourceLocator`/typed handle、`AssetManager`、`ResourceManager`、`EditorAssetManager`、资源 watcher 和 viewport 自动刷新。

## Related Files

- `zircon_editor/src/editing/command.rs`
- `zircon_editor/src/editing/history.rs`
- `zircon_editor/src/editing/viewport/controller/mod.rs`
- `zircon_editor/src/editing/viewport/handles/mod.rs`
- `zircon_editor/src/workbench/layout.rs`
- `zircon_editor/src/host/slint_host/app.rs`
- `zircon_editor/src/host/slint_host/ui.rs`
- `zircon_editor/src/host/manager.rs`
- `zircon_editor/src/host/message.rs`
- `zircon_editor/src/editing/state/mod.rs`
- `zircon_editor_ui/src/binding.rs`
- `zircon_editor/src/workbench/project/mod.rs`
- `zircon_editor/src/workbench/snapshot/mod.rs`
- `zircon_editor/src/workbench/view/mod.rs`
- `zircon_scene/src/components.rs`
- `zircon_scene/src/world/render.rs`
- `zircon_graphics/src/scene/resources/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/mesh/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay.rs`
- `zircon_editor/ui/workbench.slint`
- `zircon_editor/ui/workbench/chrome.slint`
- `zircon_scene/src/world.rs`

## Current Scope

这一批文档覆盖当前可运行的最小编辑器链路：

- workbench 壳、drawer/document/exclusive page 和浮动窗口宿主
- tab drag/drop、split hotzone、exclusive page / float zone 目标解析
- 目录式项目根、默认 scene 和 `.zircon/editor-workspace.json` sidecar 持久化
- `ResourceLocator` / typed handle 驱动的 editor 导入与 `LevelSystem` runtime 绑定
- editor host 通过 `AssetManager + ResourceManager + EditorAssetManager` façade 读取项目与资源状态，并在宿主层统一解析 ready typed handle
- 项目级最近布局和用户级默认布局/preset 持久化
- `zircon_ui` 共享 layout/tree/hit-test/surface 基础，以及 editor workbench 对共享约束求解器和几何类型的复用
- 节点创建、删除、重命名、改父子层级
- inspector 的名称、父节点、平移字段草稿与批量提交
- gizmo 拖拽和普通命令共用的 undo/redo 历史栈
- Scene 视图的 typed viewport settings、scene gizmo provider、handle overlay、outline/wireframe/grid/preview packet 分层
- 命令层对最后一个 camera、层级成环、空名称等非法编辑的保护

后续如果 inspector 扩展到 rotation/scale、多选批量编辑、拖拽层级树重排，继续在本目录追加细化文档。

