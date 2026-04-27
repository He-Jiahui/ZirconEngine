---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/root_shell_projection.rs
  - zircon_editor/ui/workbench/pane_surface.slint
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
plan_sources:
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-20 zircon_editor UI 回迁 + 树形 TOML cutover 继续清理 root 出口与旧 UI crate 文档路径
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_editor/src/tests/ui/activity/mod.rs
  - zircon_editor/src/tests/ui/activity/window_descriptor.rs
  - zircon_editor/src/tests/ui/activity/route.rs
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib --locked root_workbench_slint_exports_only_generic_host_bootstrap_symbols --offline
  - cargo test -p zircon_editor --lib --locked slint_host_presentation_uses_generic_scene_data_property --offline
  - cargo test -p zircon_editor --lib --locked slint_host_scene_uses_generic_surface_metrics_and_orchestration_names --offline
  - cargo test -p zircon_editor --lib --locked slint_host_drag_and_resize_callbacks_use_generic_host_event_names --offline
  - cargo test -p zircon_editor --lib --locked host_page_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked slint_window --offline
  - cargo test -p zircon_editor --lib --locked slint_host --offline
  - cargo test -p zircon_editor --lib apply_presentation_prefers_drawer_derived_viewport_when_pane_surface_is_stale --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib clustered_lighting_does_not_tint_final_frame_by_tile_buffer --locked --jobs 1 -- --nocapture
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --no-default-features --locked --jobs 1
  - visual screenshot: target/visual-layout/editor-window-20260427-after-cluster-tile-fix.png
  - visual screenshot: target/visual-layout/editor-window-20260427-layout-continue-header-fix.png
  - cargo test -p zircon_runtime --locked
  - cargo check --workspace --locked
doc_type: category-index
---

# UI And Layout

## Purpose

本目录记录运行时 UI 与 editor shell 共用的布局、树结构、命中和 surface 权威模型，重点回答三件事：

- 哪些布局/几何/命中语义已经属于 `zircon_runtime::ui`
- editor workbench 还保留哪些 editor-only 壳体职责
- Slint 和未来其他宿主如何只做适配层，而不是再次成为布局真源

## Documents

- [Shared UI Core Foundation](./shared-ui-core-foundation.md): `zircon_runtime::ui` 新增的共享约束类型、measure/arrange pass、`HorizontalBox`/`VerticalBox`/`ScrollableBox` 容器、retained tree、dirty/invalidation、scroll state、命中索引、pointer/focus/navigation route、统一 pointer dispatcher、虚拟化窗口工具，以及 `zircon_editor` workbench autolayout 的复用边界。
  这一轮还补上了显式 `Container` / `Overlay` / `Space` 共享容器名、pointer button payload、capture 后移出命中范围仍保持派发的底层语义、第一条 Slint viewport pointer/scroll -> shared `UiSurface + UiPointerDispatcher` 的宿主接线，以及 editor shell drag target hit-test / dock target route 的 host-owned shared bridge。
- [Shared UI Template Runtime](./shared-ui-template-runtime.md): `zircon_runtime::ui::template` 的 TOML 文档模型、component/slot 节点语义、模板校验规则、运行时实例展开、shared `UiTree` / `UiSurface` 桥接，以及 `attributes.layout` 到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / container config 的显式映射；它是后续 shared layout 求解和 Slint host projection 之前的模板真源层，并记录首批 workbench pane body 从 giant `PaneData` 平铺字段收束到 `PanePresentation + PaneBodyCompatData`、root Slint bootstrap / scene contract / drag-resize event / host-page pointer 从 workbench 专名收口到 `UiHostContext` / `UiHostScaffold` / `HostWindowSceneData` / `host_scene_data` / `HostWindowSurface*` / `host_*_pointer_event` / `HostPagePointer*` 的 cutover 状态。
- [Runtime UI Component Showcase](./runtime-ui-component-showcase.md): Runtime UI 组件描述注册表、typed value/event/state/drop 契约、`editor.ui_component_showcase` Activity Window、Showcase `.ui.toml` 资产和 Slint Material generic component-row projection。
- [UI Asset Documents And Editor Protocol](./ui-asset-documents-and-editor-protocol.md): `zircon_runtime::ui::template::asset` 当前 tree-shaped `.ui.toml` authority、flat-to-tree 一次性迁移器、shared loader/compiler/surface builder，以及 editor/runtime 如何继续共用同一条资产消费链路。
- [UI Module Boundary Refactor](./ui-module-boundary-refactor.md): `binding/model`、`event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 从混合单文件重构成 folder-backed subtree 后的职责边界，约束后续 shared UI runtime 继续演化时的文件级纪律。
- [Editor Host Final Cleanup](./editor-host-final-cleanup.md): `Final cleanup` 阶段对 editor host 剩余 legacy seam 的删除记录，覆盖 drawer extent root binding、menu button frame host setter/binding、floating-window drag/document-tab 生产路径里的 geometry outer-frame fallback，以及 root-shell projection / callback sizing helper 对 legacy geometry 的最后兜底。

## Related Files

- `zircon_runtime/src/ui/mod.rs`
- `zircon_runtime/src/ui/layout/constraints.rs`
- `zircon_runtime/src/ui/layout/geometry.rs`
- `zircon_runtime/src/ui/layout/pass/mod.rs`
- `zircon_runtime/src/ui/layout/scroll.rs`
- `zircon_runtime/src/ui/layout/virtualization.rs`
- `zircon_runtime/src/ui/template/asset/mod.rs`
- `zircon_runtime/src/ui/dispatch/mod.rs`
- `zircon_runtime/src/ui/tree/node/mod.rs`
- `zircon_runtime/src/ui/tree/hit_test.rs`
- `zircon_runtime/src/ui/surface/mod.rs`
- `zircon_editor/src/ui/workbench/autolayout/mod.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`
- `zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs`

## Current Scope

当前文档只覆盖这次共享 UI core 计划已经落地的第一段：

- 共享约束和几何类型进入 `zircon_runtime::ui`
- retained tree、dirty 标记、clip 链命中、scroll state 与 surface/render extract scaffolding 进入 `zircon_runtime::ui`
- 基础 measure/arrange pass、`Container` / `Overlay` / `Space` / `HorizontalBox` / `VerticalBox` / `ScrollableBox` 共享容器、pointer/focus/navigation route、统一 pointer dispatcher 和虚拟化窗口计算进入 `zircon_runtime::ui`
- `binding/model`、`event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 已经按 folder-backed subtree 重新分层，入口 `mod.rs` 保持导出层，不再承载混合实现
- `template/asset` 已经承接正式 `layout/widget/style` 资产 AST、tree-shaped `.ui.toml` authority、一次性 flat-to-tree migration adapter，以及编译到 `UiTemplateInstance` / `UiSurface` 的 shared 真源链路
- editor workbench 首批 pane body 已把 `.ui.toml -> PanePresentation` 作为结构 authority，Slint/native 需要的旧 DTO 被限制在 `PaneBodyCompatData` 兼容壳内
- editor root viewport 在 Drawer 可见时优先使用 drawer 派生出的 document frame 作为 Scene 内容区域，并在换算 Scene canvas 时扣除 document header、1px separator 和 viewport toolbar，避免 stale `PaneSurfaceRoot` 尺寸把 Scene render 拉回整窗宽高或把渲染帧算高后再纵向压缩；Slint Scene image 使用 fill 承接 renderer 输出尺寸，不再二次 contain letterbox。该项以窗口截图 `target/visual-layout/editor-window-20260427-layout-continue-header-fix.png` 验收，截图中 Scene 标签和状态栏均显示 `1078 x 655`，可见画面底边对齐 Console 上沿。
- Scene 后处理不再把 clustered-lighting tile buffer 作为默认可见颜色/强度叠加到最终 frame，避免编辑器预览区出现块状阶梯伪影；该项以窗口截图 `target/visual-layout/editor-window-20260427-after-cluster-tile-fix.png` 作为人工视觉验收样本。
- editor root Slint bootstrap 已开始 generic host boundary 第一刀，root 文件导出 `UiHostWindow`、`UiHostContext`、`UiHostScaffold`、`HostWindowSceneData`、`host_scene_data`、`HostWindowSurface*` scene contract 名称和 `host_drag_pointer_event` / `host_resize_pointer_event` 宿主事件，并用源码守卫阻止旧 workbench host bootstrap 名称回流
- `zircon_runtime/src/ui/mod.rs` 进一步收缩为 crate 导航层，`UiConfig`、`UI_MODULE_NAME` 和 `module_descriptor` 已下沉到 `zircon_runtime/src/ui/module.rs`
- shared pointer button payload、capture 持续派发语义，以及第一条 editor host viewport pointer/scroll shared bridge 已经接到 `zircon_runtime::ui`
- editor shell drag target route 已经开始通过 host-owned `WorkbenchDragTargetBridge` 复用 shared `UiSurface + UiPointerDispatcher`
- editor workbench layout 数学改为直接复用共享 solver 和共享 frame/size/constraint 类型
- `zircon_editor::ui::asset_editor` 已经承接 `editor.ui_asset` 的窗口协议、session 和 source/preview authoring 入口，为后续更深的 tree-native editor refactor 保留稳定 id、mode 和 route 载荷
- `WorkbenchLayout` 继续只做 editor 拓扑与持久化，不再承担底层基础类型的定义权

后续如果继续落地更完整的容器族、grid/flow 虚拟化、editor/runtime 宿主接线、world-space UI 和 runtime ECS bridge，可以在本目录继续追加更细分的实现文档。
