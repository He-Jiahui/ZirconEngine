# Editor And Graphics High-Risk Module Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** 把 `zircon_editor` 与 `zircon_graphics` 中剩余的高风险聚合文件拆成 folder-backed subtree，并让 `callback_dispatch.rs`、`layout.rs`、`overlay.rs` 只保留结构入口或单一职责核心。

**Architecture:** 先按“声明层 / 行为层 / domain 层”划分三个目标文件的内部责任，再把原文件降级为结构边界。每一轮只拆一个行为族，保持公共导出不变，先靠现有测试守行为，再在拆分完成后同步拆测试文件和架构文档。

**Tech Stack:** Rust 2024, Slint host adapters, serde, wgpu, image, cargo fmt, cargo test, repo-local docs

---

## Target File Structure

### `zircon_editor/src/host/slint_host/callback_dispatch/mod.rs`

目标形态：

```text
zircon_editor/src/host/slint_host/callback_dispatch/mod.rs
zircon_editor/src/host/slint_host/callback_dispatch/
  constants.rs
  common/
    mod.rs
    dispatch.rs
    effects.rs
    slot_parse.rs
  template_bridge/
    mod.rs
    workbench.rs
    viewport_toolbar.rs
    asset_surface.rs
    welcome_surface.rs
    inspector_surface.rs
    pane_surface.rs
  shared_pointer/
    mod.rs
    menu.rs
    activity_rail.rs
    host_page.rs
    document_tab.rs
    drawer_header.rs
    viewport_toolbar.rs
    welcome_recent.rs
    hierarchy.rs
    asset_tree.rs
    asset_content.rs
    asset_reference.rs
  viewport/
    mod.rs
    bridge.rs
    pointer_dispatch.rs
    route_mapping.rs
    toolbar_control.rs
    command_dispatch.rs
    snap_cycle.rs
  layout/
    mod.rs
    command.rs
    tab_drop.rs
    drawer_toggle.rs
    main_page.rs
  asset/
    mod.rs
    surface_control.rs
    selection.rs
    search.rs
  inspector/
    mod.rs
    surface_control.rs
    apply.rs
    draft_field.rs
    delete_selected.rs
    mesh_import_path.rs
  welcome/
    mod.rs
    surface_control.rs
```

职责约束：

- `callback_dispatch.rs` 只做 `mod` 和 `pub use`
- 每个 builtin template bridge 一个文件
- 每种 pointer click dispatch 一个文件
- viewport command / pointer route / snap cycle 分开
- inspector/asset/welcome/layout 行为不得再互相混放

### `zircon_editor/src/workbench/layout/mod.rs`

目标形态：

```text
zircon_editor/src/workbench/layout/mod.rs
zircon_editor/src/workbench/layout/
  main_page_id.rs
  activity_drawer_slot.rs
  activity_drawer_mode.rs
  split_axis.rs
  split_placement.rs
  dock_edge.rs
  workspace_target.rs
  tab_insertion_side.rs
  tab_insertion_anchor.rs
  restore_policy.rs
  tab_stack_layout.rs
  document_node.rs
  activity_drawer_layout.rs
  main_host_page_layout.rs
  floating_window_layout.rs
  workbench_layout.rs
  layout_diff.rs
  layout_normalization_report.rs
  drag_payload.rs
  hit_target.rs
  drop_target.rs
  layout_command.rs
  layout_manager.rs
  manager/
    mod.rs
    defaults.rs
    persistence.rs
    restore.rs
    drop_resolution.rs
    normalize.rs
    apply.rs
    attach.rs
    detach.rs
    focus.rs
    workspace_access.rs
    edge_mapping.rs
```

职责约束：

- 每个顶层声明独立成文件
- `LayoutManager` 只留 façade，实际行为放进 `manager/`
- `DocumentNode` / `TabStackLayout` 的树操作不能和持久化/恢复逻辑混放
- layout schema 与 mutation pipeline 明确分层

### `zircon_graphics/src/scene/scene_renderer/overlay/mod.rs`

目标形态：

```text
zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
zircon_graphics/src/scene/scene_renderer/overlay/
  viewport_icon_source.rs
  empty_viewport_icon_source.rs
  viewport_overlay_renderer.rs
  prepared_overlay_buffers.rs
  prepared_scene_gizmo_pass.rs
  prepared_icon_draw.rs
  pass_order.rs
  line_pass.rs
  passes/
    mod.rs
    preview_sky_pass.rs
    base_scene_pass.rs
    selection_outline_pass.rs
    wireframe_pass.rs
    grid_pass.rs
    handle_pass.rs
    scene_gizmo_pass.rs
  icons/
    mod.rs
    viewport_icon_atlas/mod.rs
    icon_entry.rs
    viewport_icon_sprite.rs
    icon_slot.rs
  shaders/
    line.wgsl
    icon.wgsl
    sky.wgsl
```

职责约束：

- `overlay.rs` 只保留 `mod` 和 `pub use`
- 每个 render pass 一个文件
- icon atlas 和 icon draw 准备链独立
- shader 源不再嵌在 Rust 大文件底部

## Validation Baseline

在开始拆分前记录基线：

- `cargo test -p zircon_editor --offline slint_callback_dispatch`
- `cargo test -p zircon_editor --offline layout_manager`
- `cargo test -p zircon_editor --offline workbench`
- `cargo test -p zircon_graphics --offline scene_overlay`
- `cargo test -p zircon_graphics --offline project_render`

预期：全部通过；如果有既有失败，先记录失败名，不在本轮顺手修 unrelated 行为。

### Task 1: Callback Dispatch Skeleton

**Files:**
- Modify: `zircon_editor/src/host/slint_host/callback_dispatch/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/constants.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/common/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/common/dispatch.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/common/effects.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/common/slot_parse.rs`
- Test: `zircon_editor/src/tests/host/slint_callback_dispatch.rs`

- [x] 建立 `callback_dispatch/` 目录树，让根文件只保留 `mod` 与 `pub use`
- [x] 把 builtin document id、control id、viewport surface node id 移到 `constants.rs`
- [x] 把 `dispatch_envelope`、`dispatch_editor_binding`、`merge_effects`、`parse_activity_drawer_slot` 下沉到 `common/`
- [x] 运行 `cargo fmt -p zircon_editor`
- [x] 运行 `cargo test -p zircon_editor --offline slint_callback_dispatch`

### Task 2: Callback Dispatch Template Bridges

**Files:**
- Modify: `zircon_editor/src/host/slint_host/callback_dispatch/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/workbench.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/viewport_toolbar.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/asset_surface.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/welcome_surface.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/inspector_surface.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/template_bridge/pane_surface.rs`
- Test: `zircon_editor/src/tests/host/slint_callback_dispatch.rs`

- [x] 为六个 `Builtin*TemplateBridge` 各建一个文件，桥接错误类型与 bridge struct 同文件持有
- [x] 把 bridge 共享的 `binding_for_control` / `binding_for_control_with_arguments` 模式收束到 `template_bridge/mod.rs` 的共享 helper
- [x] 让 `build_builtin_workbench_host_projection(...)` 与 workbench bridge 同域，不再放在根文件中部
- [x] 保持外部导出名不变，避免 host/slint 其它模块大面积改 import
- [x] 运行 `cargo test -p zircon_editor --offline builtin_workbench`
- [x] 运行 `cargo test -p zircon_editor --offline builtin_viewport_toolbar`

### Task 3: Callback Dispatch Shared Pointer Modules

**Files:**
- Modify: `zircon_editor/src/host/slint_host/callback_dispatch/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/menu.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/activity_rail.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/host_page.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/document_tab.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/drawer_header.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/viewport_toolbar.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/welcome_recent.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/hierarchy.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/asset_tree.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/asset_content.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/shared_pointer/asset_reference.rs`
- Test: `zircon_editor/src/tests/host/slint_menu_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_activity_rail_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_document_tab_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_drawer_header_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_host_page_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_list_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_asset_pointer.rs`
- Test: `zircon_editor/src/tests/host/slint_viewport_toolbar_pointer.rs`

- [x] 把每种 `Shared*PointerClickDispatch` struct 与对应 `dispatch_shared_*` 函数放进各自文件
- [x] 每个文件只保留一种 route -> typed event 的翻译逻辑
- [x] 公共的 `host_adapter::dispatch_event` 调用保留在 shared helper，不要复制分支
- [x] 按域跑现有测试：
  - `cargo test -p zircon_editor --offline slint_menu_pointer`
  - `cargo test -p zircon_editor --offline slint_activity_rail_pointer`
  - `cargo test -p zircon_editor --offline slint_document_tab_pointer`
  - `cargo test -p zircon_editor --offline slint_drawer_header_pointer`
  - `cargo test -p zircon_editor --offline slint_host_page_pointer`
  - `cargo test -p zircon_editor --offline slint_list_pointer`
  - `cargo test -p zircon_editor --offline slint_asset_pointer`
  - `cargo test -p zircon_editor --offline slint_viewport_toolbar_pointer`

### Task 4: Callback Dispatch Builtin Controls And Viewport

**Files:**
- Modify: `zircon_editor/src/host/slint_host/callback_dispatch/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/bridge.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/pointer_dispatch.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/route_mapping.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/toolbar_control.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/command_dispatch.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/viewport/snap_cycle.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/layout/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/layout/command.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/layout/tab_drop.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/layout/drawer_toggle.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/layout/main_page.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/asset/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/asset/surface_control.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/asset/selection.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/asset/search.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/surface_control.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/apply.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/draft_field.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/delete_selected.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/inspector/mesh_import_path.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/welcome/mod.rs`
- Create: `zircon_editor/src/host/slint_host/callback_dispatch/welcome/surface_control.rs`
- Test: `zircon_editor/src/tests/host/slint_callback_dispatch.rs`
- Test: `zircon_editor/src/tests/host/slint_event_bridge.rs`

- [x] 把 `SharedViewportPointerBridge` 和 viewport pointer dispatcher 相关逻辑下沉到 `viewport/`
- [x] 把 `next_display_mode_name`、`next_grid_mode_name`、`next_translate_snap`、`next_rotate_snap_degrees`、`next_scale_snap` 归入 `viewport/snap_cycle.rs`
- [x] 把 builtin asset / welcome / inspector / pane / workbench control dispatch 分域归位
- [x] 把 `dispatch_layout_command`、`dispatch_tab_drop` 从混合文件中抽到 `layout/`
- [x] 跑关键测试：
  - `cargo test -p zircon_editor --offline slint_callback_dispatch`
  - `cargo test -p zircon_editor --offline slint_event_bridge`

### Task 5: Callback Dispatch Tests And Docs

**Files:**
- Modify: `zircon_editor/src/tests/host/mod.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/template_bridge.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/viewport.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/layout.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/asset.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/inspector.rs`
- Create: `zircon_editor/src/tests/host/slint_callback_dispatch/welcome.rs`
- Modify: `docs/editor-and-tooling/editor-template-compatibility-migration.md`
- Modify: `docs/editor-and-tooling/editor-workbench-shell.md`
- Modify: `docs/editor-and-tooling/ui-binding-reflection-architecture.md`

- [x] 把 `slint_callback_dispatch.rs` 镜像拆成目录树，测试文件名与生产模块名对齐
- [x] 更新三份 editor 文档中的 `related_code` / `implementation_files` 头部
- [x] 在正文里说明 `callback_dispatch.rs` 已降级为结构入口，列出新的子树域划分
- [x] 运行 `cargo test -p zircon_editor --offline slint_callback_dispatch`
- [x] 运行 `cargo test -p zircon_editor --offline`

### Task 6: Workbench Layout Declarations

**Files:**
- Modify: `zircon_editor/src/workbench/layout/mod.rs`
- Create: `zircon_editor/src/workbench/layout/main_page_id.rs`
- Create: `zircon_editor/src/workbench/layout/activity_drawer_slot.rs`
- Create: `zircon_editor/src/workbench/layout/activity_drawer_mode.rs`
- Create: `zircon_editor/src/workbench/layout/split_axis.rs`
- Create: `zircon_editor/src/workbench/layout/split_placement.rs`
- Create: `zircon_editor/src/workbench/layout/dock_edge.rs`
- Create: `zircon_editor/src/workbench/layout/workspace_target.rs`
- Create: `zircon_editor/src/workbench/layout/tab_insertion_side.rs`
- Create: `zircon_editor/src/workbench/layout/tab_insertion_anchor.rs`
- Create: `zircon_editor/src/workbench/layout/restore_policy.rs`
- Create: `zircon_editor/src/workbench/layout/layout_diff.rs`
- Create: `zircon_editor/src/workbench/layout/layout_normalization_report.rs`
- Create: `zircon_editor/src/workbench/layout/drag_payload.rs`
- Create: `zircon_editor/src/workbench/layout/hit_target.rs`
- Create: `zircon_editor/src/workbench/layout/drop_target.rs`
- Create: `zircon_editor/src/workbench/layout/layout_command.rs`
- Test: `zircon_editor/src/tests/workbench/layout.rs`

- [x] 先抽纯声明，不动 `LayoutManager` 行为
- [x] 每个顶层 enum/struct 一文件，保留 derive 与最小 constructor
- [x] 根 `layout.rs` 改为结构导出层
- [x] 运行 `cargo test -p zircon_editor --offline layout_manager`

### Task 7: Workbench Layout Tree Model

**Files:**
- Create: `zircon_editor/src/workbench/layout/tab_stack_layout.rs`
- Create: `zircon_editor/src/workbench/layout/document_node.rs`
- Create: `zircon_editor/src/workbench/layout/activity_drawer_layout.rs`
- Create: `zircon_editor/src/workbench/layout/main_host_page_layout.rs`
- Create: `zircon_editor/src/workbench/layout/floating_window_layout.rs`
- Create: `zircon_editor/src/workbench/layout/workbench_layout.rs`
- Test: `zircon_editor/src/tests/workbench/layout.rs`
- Test: `zircon_editor/src/tests/workbench/chrome_snapshot.rs`
- Test: `zircon_editor/src/tests/workbench/view_model.rs`

- [x] 把 layout schema 与树操作从 `LayoutManager` 分离
- [x] `TabStackLayout::insert/remove` 留在 `tab_stack_layout.rs`
- [x] `DocumentNode::node_at_path_mut/remove_instance/contains` 留在 `document_node.rs`
- [x] `WorkbenchLayout::default()` 单独归到 `workbench_layout.rs`
- [x] 运行：
  - `cargo test -p zircon_editor --offline layout_manager_moves_views`
  - `cargo test -p zircon_editor --offline chrome_snapshot`
  - `cargo test -p zircon_editor --offline view_model`

### Task 8: Workbench Layout Manager Behavior

**Files:**
- Create: `zircon_editor/src/workbench/layout/layout_manager.rs`
- Create: `zircon_editor/src/workbench/layout/manager/mod.rs`
- Create: `zircon_editor/src/workbench/layout/manager/defaults.rs`
- Create: `zircon_editor/src/workbench/layout/manager/persistence.rs`
- Create: `zircon_editor/src/workbench/layout/manager/restore.rs`
- Create: `zircon_editor/src/workbench/layout/manager/drop_resolution.rs`
- Create: `zircon_editor/src/workbench/layout/manager/normalize.rs`
- Create: `zircon_editor/src/workbench/layout/manager/apply.rs`
- Create: `zircon_editor/src/workbench/layout/manager/attach.rs`
- Create: `zircon_editor/src/workbench/layout/manager/detach.rs`
- Create: `zircon_editor/src/workbench/layout/manager/focus.rs`
- Create: `zircon_editor/src/workbench/layout/manager/workspace_access.rs`
- Create: `zircon_editor/src/workbench/layout/manager/edge_mapping.rs`
- Test: `zircon_editor/src/tests/workbench/layout.rs`
- Test: `zircon_editor/src/tests/host/manager.rs`
- Test: `zircon_editor/src/tests/editor_event/runtime.rs`

- [x] 让 `LayoutManager` 只保留 public façade
- [x] 把 `restore_workspace` / `load_*` / `save_*` 放到 `manager/persistence.rs` 与 `manager/restore.rs`
- [x] 把 `resolve_drop` 放到 `manager/drop_resolution.rs`
- [x] 把 `normalize` 与 normalization helpers 放到 `manager/normalize.rs`
- [x] 把 `apply` 主分发器与 `attach_instance` / `detach_instance` / `focus_instance` 分开放到单独文件
- [x] 运行：
  - `cargo test -p zircon_editor --offline layout_manager`
  - `cargo test -p zircon_editor --offline host::manager`
  - `cargo test -p zircon_editor --offline editor_event`

### Task 9: Workbench Layout Docs And Test Cleanup

**Files:**
- Modify: `docs/editor-and-tooling/editor-template-compatibility-migration.md`
- Modify: `docs/editor-and-tooling/editor-workbench-shell.md`
- Modify: `docs/editor-and-tooling/index.md`
- Modify: `zircon_editor/src/tests/workbench/layout.rs`

- [x] 在文档中把 `layout.rs` 从“单文件 schema + mutation”改成“结构入口 + layout subtree”
- [x] 补充新的 `related_code` 条目，尤其是 `layout/manager/*.rs`
- [x] 复查 `zircon_editor/src/tests/workbench/layout.rs` 是否仍需再拆；如果超过单一职责，按 `schema.rs` / `mutation.rs` / `restore.rs` 拆
- [x] 运行 `cargo test -p zircon_editor --offline workbench`

### Task 10: Overlay Skeleton And Public Types

**Files:**
- Modify: `zircon_graphics/src/scene/scene_renderer/overlay/mod.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icon_source/viewport_icon_source.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icon_source/empty_viewport_icon_source.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/prepared/prepared_overlay_buffers.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/prepared/prepared_scene_gizmo_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/prepared/prepared_icon_draw.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/pass_order.rs`
- Test: `zircon_graphics/src/tests/scene_overlay.rs`

- [x] 先抽 public trait/type，不动 render pass 细节
- [x] `ViewportOverlayRenderer::pass_order()` 改为读取 `pass_order.rs`
- [x] `overlay.rs` 降级为结构入口
- [x] 运行 `cargo test -p zircon_graphics --offline scene_overlay`

### Task 11: Overlay Pass Extraction

**Files:**
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/line_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/preview_sky_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/base_scene_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/selection_outline_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/wireframe_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/grid_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/handle_pass.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/passes/scene_gizmo_pass/mod.rs`
- Test: `zircon_graphics/src/tests/scene_overlay.rs`
- Test: `zircon_graphics/src/tests/project_render.rs`

- [x] 每个 pass 的 `record(...)` 迁到单文件
- [x] `SceneGizmoPass::new/prepare/record` 保持同域，不要和 atlas 混放
- [x] `begin_line_pass(...)` 作为公共 helper 抽到 `line_pass.rs`
- [x] 运行：
  - `cargo test -p zircon_graphics --offline scene_overlay`
  - `cargo test -p zircon_graphics --offline project_render`

### Task 12: Overlay Icon Atlas And Shader Extraction

**Files:**
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icons/mod.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas/mod.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icons/icon_entry.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_sprite.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/icons/icon_slot.rs`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/shaders/line.wgsl`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/shaders/icon.wgsl`
- Create: `zircon_graphics/src/scene/scene_renderer/overlay/shaders/sky.wgsl`
- Modify: `zircon_graphics/src/scene/scene_renderer/overlay/passes/scene_gizmo_pass/mod.rs`
- Modify: `zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer.rs`
- Test: `zircon_graphics/src/tests/scene_overlay.rs`
- Test: `zircon_graphics/src/tests/project_render.rs`
- Test: `zircon_graphics/src/tests/pipeline_compile.rs`

- [x] 把 `ViewportIconAtlas` 及其 supporting enum/struct 全部抽离
- [x] 用 `include_str!` 或同等显式加载方式接入三个 WGSL 文件
- [x] 验证 shader 名称与 pipeline entry point 没有改变
- [x] 运行：
  - `cargo test -p zircon_graphics --offline scene_overlay`
  - `cargo test -p zircon_graphics --offline project_render`
  - `cargo test -p zircon_graphics --offline pipeline_compile`

### Task 13: Graphics Docs And Final Validation

**Files:**
- Modify: `docs/editor-and-tooling/scene-viewport-gizmo-handle-overlays.md`
- Modify: `docs/assets-and-rendering/directory-project-asset-rendering.md`
- Modify: `docs/assets-and-rendering/index.md`
- Modify: `docs/engine-architecture/index.md`

- [x] 更新 `overlay.rs` 相关文档头部，把 `overlay/passes/*`、`overlay/icons/*`、`overlay/shaders/*` 纳入 `related_code`
- [x] 在正文里明确 overlay pass pipeline 已拆成 pass 子树和 icon atlas 子树
- [x] 最终运行：
  - `cargo fmt -p zircon_editor -p zircon_graphics`
  - `cargo test -p zircon_editor --offline`
  - `cargo test -p zircon_graphics --offline`

## Sequencing Rules

- 先做 `callback_dispatch`，因为它的测试最细，拆分反馈最快
- 再做 `layout`，因为 `callback_dispatch` 仍然大量依赖 `LayoutCommand` 和 `WorkbenchLayout`
- 最后做 `overlay`，避免 editor 与 graphics 同时引入宽范围 import churn
- 每完成一个主文件，先跑该主文件最窄测试，再跑对应 crate 的离线测试
- 不要在同一批次里同时改 production subtree 和 unrelated docs；production 绿了再补 docs

## Completion Checklist

- `callback_dispatch.rs`、`layout.rs`、`overlay.rs` 都缩成结构入口或单一 façade
- 每个顶层 enum/struct/error 各自独立文件
- pointer / builtin control / viewport / layout / render pass / icon atlas 不再混放
- 关联测试文件与生产目录结构镜像对齐
- 相关 `docs/` 头部和正文同步到新路径
- `cargo test -p zircon_editor --offline` 与 `cargo test -p zircon_graphics --offline` 通过
