---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/core/editor_event/host_adapter.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/ui/workbench/view/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_resource/src/lib.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/locator.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_scene/src/lib.rs
  - zircon_scene/src/world/mod.rs
implementation_files:
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editing/history.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/locator.rs
  - zircon_editor/src/ui/workbench/view/mod.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_scene/src/world/mod.rs
plan_sources:
  - user: 2026-04-12 扩展 editor 命令系统到删除节点、改父子层级、重命名和 inspector 字段批量提交
  - user: 2026-04-12 实现 Zircon Editor Workbench Shell V1
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - user: 2026-04-17 Viewport 交互边界重构计划
  - user: 2026-04-20 继续，把 runtime 层仍然存在的 editor only 实现迁回 editor
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/全系统重构方案.md
  - .cursor/plans/基本路线图.md
tests:
  - zircon_editor/src/tests/ui/activity/mod.rs
  - zircon_editor/src/tests/ui/activity/window_descriptor.rs
  - zircon_editor/src/tests/ui/activity/route.rs
  - zircon_editor/src/lib.rs
  - zircon_scene/src/lib.rs
  - cargo test -p zircon_editor --lib --locked tests::ui::activity
  - cargo test -p zircon_editor -- --nocapture
  - cargo test -p zircon_app -- --nocapture
  - cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo test -p zircon_app --locked
  - cargo test -p zircon_graphics --locked
  - cargo test -p zircon_editor editor_viewport_interaction_boundary_lives_in_editor_crate --locked
  - cargo test -p zircon_editor editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings --locked
  - cargo check --workspace --locked
  - cargo check -p zircon_editor --lib --message-format short
  - cargo check -p zircon_runtime --lib --message-format short --target-dir target/codex-runtime-config-check
doc_type: category-index
---

# Editor And Tooling

## Purpose

本目录记录编辑器宿主层、workbench 壳以及场景编辑工作流的实现细节，重点覆盖：

- `zircon_editor` 如何把 Slint 宿主组织成 workbench shell
- 布局、view registry、项目 workspace 和配置持久化如何协同
- UI 草稿态如何转换成命令，并在 `zircon_runtime::scene::LevelSystem` 所托管的 `zircon_scene::Scene` 上安全执行与 undo/redo

## Documents

- [Editor Workbench Shell](./editor-workbench-shell.md): 混合固定壳 workbench、主 tabs、drawers、document workspace、native floating windows、拖放命中与布局持久化。
- [Editor Command Workflow](./editor-command-workflow.md): editor 命令层、历史栈、inspector 草稿批量提交、删除/改父子/重命名等行为约束。
- [Scene Viewport Gizmo, Handle, And Overlay Pipeline](./scene-viewport-gizmo-handle-overlays.md): Scene 视图的 typed viewport settings、scene render packet、scene gizmo provider、handle overlay、wireframe/preview/grid 分层与测试口径。
- [Viewport Interaction Boundary Split](./viewport-interaction-boundary-split.md): `zircon_editor` / `zircon_graphics` / `zircon_app` 的 viewport ownership 重分配，editor-owned interaction types、runtime-private camera controller，以及 graphics 仅保留 render framework/overlay 职责。
- [Runtime/Editor Boundary Cleanup](./runtime-editor-boundary-cleanup.md): 继续把 runtime 内残留的 editor-only 默认假设剪掉，包括 viewport authoring contract 回迁到 `zircon_editor`，以及 foundation config path 的 editor 命名中性化。
- [Crate Boundary Audit Round 2](./crate-boundary-audit-round-2.md): 第二轮更严格的错包审计规则、`zircon_graphics` 红测根因、已通过的边界项，以及下一批最强迁移候选。
- [UI Binding And Reflection Architecture](./ui-binding-reflection-architecture.md): `zircon_ui` / `zircon_editor::ui` / `zircon_runtime::input` 边界，nativeBinding、反射树、REPL/网络操控与 headless 回放架构。
- [Animation Binding Command Surface](./animation-binding-command-surface.md): `AnimationCommand` 如何统一轨道创建/删除、重绑定、关键帧、scrub 和 playback 的 editor authoring binding 面，并进入正式 `EditorEventRuntime` 事件链与动画资产 view 路由。
- [Animation Editor Pane Session](./animation-editor-pane-session.md): `ui::animation_editor` 与 `ui::host::animation_editor_sessions` 如何维护 sequence/graph/state-machine 的最小真实 session model，并把 animation 资产页签投影到正式 workbench pane。
- [Engine Architecture / Runtime Diagnostics Facade](../engine-architecture/runtime-diagnostics-facade.md): `EditorManager::runtime_diagnostics()`、`editor.runtime_diagnostics` activity pane、`RuntimeDiagnosticsV1` pane payload 和 `pane.runtime.diagnostics.body` TOML 模板的 editor-facing inspection 边界。
- [Editor Template Compatibility Migration](./editor-template-compatibility-migration.md): `zircon_editor::ui` 的 editor-only template catalog/registry/adapter，如何把 shared `UiBindingRef` 收口到 typed `EditorUiBinding`，以及后续把 TOML 模板实例接到 Slint host 的迁移顺序。
- [UI Asset Editor Host Session](./ui-asset-editor-host-session.md): `zircon_editor::ui::host` 与 `zircon_editor::ui::asset_editor` 的当前 owner 边界，说明 `EditorManager`、layout/window/session orchestration 已回迁到 `src/ui/`，且不再通过 `core` 兼容 re-export 持有实现。
- [Editor Structure Hard Cutover Rules](./editor-structure-hard-cutover-rules.md): `core/scene/ui` 顶层分工、`ui/host` vs `ui/slint_host` vs `ui/asset_editor` vs `ui/workbench` 的 owner 红线，以及 `zircon_editor` crate root/public surface 收口规则。
- [UI And Layout / UI Asset Documents And Editor Protocol](../ui-and-layout/ui-asset-documents-and-editor-protocol.md): `zircon_ui::template::asset` 的 `layout/widget/style` 编译链、selector stylesheet、legacy adapter、slot-aware shared bridge，以及 shared asset model 如何移交给 editor asset pipeline 和 host session。
- [UI And Layout / Shared UI Core Foundation](../ui-and-layout/shared-ui-core-foundation.md): 运行时/编辑器共享的 `zircon_ui` 约束类型、retained tree、命中索引、surface/render extract，以及 editor workbench 对共享布局核心的复用边界。
- [UI And Layout / Shared UI Template Runtime](../ui-and-layout/shared-ui-template-runtime.md): shared TOML 模板文档、slot/composite 展开和稳定 binding ref 保留语义，是 editor shell compatibility migration 的共享模板真源。
- [Assets And Rendering / Directory Project Asset Rendering](../assets-and-rendering/directory-project-asset-rendering.md): 目录式项目根、`ResourceLocator`/typed handle、`AssetManager`、`ResourceManager`、`EditorAssetManager`、资源 watcher 和 viewport 自动刷新。

## Related Files

- `zircon_editor/src/core/editing/command.rs`
- `zircon_editor/src/core/editing/history.rs`
- `zircon_editor/src/scene/viewport/controller/mod.rs`
- `zircon_editor/src/scene/viewport/handles/mod.rs`
- `zircon_editor/src/ui/workbench/layout/mod.rs`
- `zircon_editor/src/ui/slint_host/app.rs`
- `zircon_editor/src/ui/slint_host/ui.rs`
- `zircon_editor/src/ui/host/mod.rs`
- `zircon_editor/src/ui/host/module.rs`
- `zircon_editor/src/core/editor_event/host_adapter.rs`
- `zircon_editor/src/core/editing/state/mod.rs`
- `zircon_editor/src/ui/binding/mod.rs`
- `zircon_editor/src/ui/asset_editor/mod.rs`
- `zircon_editor/src/ui/workbench/project/mod.rs`
- `zircon_editor/src/ui/workbench/snapshot/mod.rs`
- `zircon_editor/src/ui/workbench/view/mod.rs`
- `zircon_scene/src/components.rs`
- `zircon_scene/src/world/render.rs`
- `zircon_graphics/src/scene/resources/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/mesh/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/mod.rs`
- `zircon_editor/ui/workbench.slint`
- `zircon_editor/ui/workbench/chrome.slint`
- `zircon_scene/src/world/mod.rs`

## Current Scope

这一批文档覆盖当前可运行的最小编辑器链路：

- workbench 壳、drawer/document/exclusive page 和浮动窗口宿主
- tab drag/drop、split hotzone、exclusive page / float zone 目标解析
- 目录式项目根、默认 scene 和 `.zircon/editor-workspace.json` sidecar 持久化
- `ResourceLocator` / typed handle 驱动的 editor 导入与 `LevelSystem` runtime 绑定
- editor host 通过 asset/resource-owned `AssetManager + ResourceManager + EditorAssetManager` 合同读取项目与资源状态，并在宿主层统一解析 ready typed handle
- `EditorManager`、`EditorModule` wiring、view/layout/window/workspace orchestration 与 UI asset session host bookkeeping 现在统一归 `zircon_editor::ui::host`，`core::host` 已删除
- `zircon_editor` 的长期结构标准：crate root 窄导出、`core/scene/ui` 固定分层、asset editor/workbench specialist path 直达 owner module
- 项目级最近布局和用户级默认布局/preset 持久化
- `zircon_ui` 共享 layout/tree/hit-test/surface 基础，以及 editor workbench 对共享约束求解器和几何类型的复用
- 节点创建、删除、重命名、改父子层级
- inspector 的名称、父节点、平移字段草稿与批量提交
- gizmo 拖拽和普通命令共用的 undo/redo 历史栈
- Scene 视图的 typed viewport settings、scene gizmo provider、handle overlay、outline/wireframe/grid/preview packet 分层
- 命令层对最后一个 camera、层级成环、空名称等非法编辑的保护

后续如果 inspector 扩展到 rotation/scale、多选批量编辑、拖拽层级树重排，继续在本目录追加细化文档。



