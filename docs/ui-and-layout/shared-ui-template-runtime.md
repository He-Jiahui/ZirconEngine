---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/FiraMono-subset.ttf
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/ui/surface/render/typography.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/workbench_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/mod.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
implementation_files:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/FiraMono-subset.ttf
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/ui/surface/render/typography.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/template/document.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/tests/host/template_runtime/workbench_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml
  - zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_context.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_scene.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/pane_content.slint
  - zircon_editor/src/ui/slint_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/document_tab_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/drawer_header_pointer/mod.rs
  - zircon_editor/src/ui/slint_host/shell_pointer.rs
  - zircon_editor/src/ui/slint_host/drawer_resize.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/shared_pointer/activity_rail.rs
  - zircon_editor/src/ui/slint_host/callback_dispatch/template_bridge/workbench/root_shell_frames.rs
plan_sources:
  - user: 2026-04-15 按自定义 TOML 描述文件运行时构建 Slint 树并严格服从 Shared Layout 契约
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-18 继续下一步，推进 Runtime visual contract
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要 re-export，直接清理 core 里 ui 部分
  - user: 2026-04-21 M1 主链收口与文本底座计划，模板样式补齐 typography 字段并驱动 runtime 文本底座
  - user: 2026-04-21 继续推进 M1，默认字体入口必须成为 runtime 自有资产
  - user: 2026-04-21 继续推进 M1，把 .font.toml 正式纳入 asset/resource 主链并让 UI loader 复用公共 FontAsset
  - user: 2026-04-21 继续推进 M1，让项目内 res:// 字体资产通过 ProjectAssetManager 进入 runtime UI 文本链路
  - user: 2026-04-24 继续编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - docs/superpowers/plans/2026-04-24-ui-toml-pane-template-implementation.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_editor/src/tests/host/template_runtime/workbench_document.rs
  - zircon_editor/src/tests/host/template_runtime/shared_surface.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - zircon_editor/src/tests/host/slint_hierarchy_template_body.rs
  - zircon_editor/src/tests/host/slint_animation_template_body.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - cargo test -p zircon_runtime render_extract_carries_visual_contract_fields_for_visible_nodes
  - cargo test -p zircon_runtime default_runtime_font_manifest_stays_inside_runtime_assets --locked
  - cargo test -p zircon_runtime font_asset_ --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract project_font_manifest_resolves_through_project_asset_manager --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract --locked
  - cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked
  - cargo check -p zircon_runtime --locked --lib
  - cargo test -p zircon_runtime template_tree_builder_projects_template_instance_into_shared_ui_tree_with_metadata --locked
  - cargo test -p zircon_editor --lib --locked tab_drag_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked drawer_resize_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked root_shell_frames_use_generic_host_type_names --offline
  - cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked
  - cargo test -p zircon_runtime --locked
  - cargo check -p zircon_editor
  - cargo test -p zircon_editor boundary
  - cargo test -p zircon_editor template
  - cargo test -p zircon_editor --lib --locked template_runtime --offline
  - cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline
  - cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline
  - cargo test -p zircon_editor --lib --locked root_workbench_slint_exports_only_generic_host_bootstrap_symbols --offline
  - cargo test -p zircon_editor --lib --locked slint_host_presentation_uses_generic_scene_data_property --offline
  - cargo test -p zircon_editor --lib --locked slint_host_scene_uses_generic_surface_metrics_and_orchestration_names --offline
  - cargo test -p zircon_editor --lib --locked slint_host_drag_and_resize_callbacks_use_generic_host_event_names --offline
  - cargo test -p zircon_editor --lib --locked host_page_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked activity_rail_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked document_tab_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked drawer_header_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked shell_pointer_module_uses_generic_host_type_names --offline
  - cargo test -p zircon_editor --lib --locked slint_window --offline
  - cargo test -p zircon_editor --lib --locked slint_host --offline
  - cargo fmt --all
  - cargo check -p zircon_editor --locked --offline
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput
doc_type: module-detail
---

# Shared UI Template Runtime

## Purpose

这一层现在承接的是“tree `.ui.toml` 资产 -> shared UI 权威模型”的正式入口，不再把 legacy `UiTemplateDocument` 当成 editor/runtime 生产链路的主文档类型。

在这轮 cutover 后，正式 authority 已经固定成：

1. tree `.ui.toml` 解析成 `UiAssetDocument`
2. `UiAssetLoader` 校验 tree authority 与稳定 `node_id`
3. `UiDocumentCompiler` 产出 `UiCompiledDocument`
4. `UiCompiledDocument` 实例化成 shared `UiTemplateInstance`
5. `UiTemplateSurfaceBuilder` 把实例树投影到 shared `UiSurface`
6. `UiSurface::compute_layout(...)` 按 shared measure/arrange 契约求 frame / clip / scroll window
7. editor/runtime adapter 再继续把 shared surface、projection 或宿主节点模型投影到宿主层

editor host 这一侧也已经同步收口：

- [`EditorTemplateRegistry`](../../zircon_editor/src/ui/template/registry.rs) 只存 `UiCompiledDocument`
- [`runtime_host.rs`](../../zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 的生产态只接受 `UiAssetDocument`
- builtin host 文档 [`zircon_editor/assets/ui/editor/host/*.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml) 已改写成 tree asset authority，并继续留在 crate `src/` 之外
- `UiTemplateDocument` / `UiTemplateLoader` / `UiLegacyTemplateAdapter` 仅剩 legacy compat adapter 或测试路径，不再是 editor production runtime 的 fallback authority

这意味着这一层当前负责的是“资产语义真源 + shared tree 首段落点 + 显式 layout 合同落点”，但仍然不负责 editor docking 业务、Slint callback ABI 或宿主专属状态机。真正的布局、命中、焦点和 route 权威仍然在 `UiTree` / `UiSurface` / shared layout contract。

## Current Asset Authority

这一轮 editor host 的最新收口点是“移除生产态 legacy loader fallback，而不是继续让 builtin host 模板在 runtime 里双解析”：

- [`EditorTemplateRegistry`](../../zircon_editor/src/ui/template/registry.rs) 删除 `UiTemplateDocument` 存储分支，只保留 `UiCompiledDocument`
- [`EditorUiHostRuntime::register_document_source(...)`](../../zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs) 只走 `parse_ui_asset_document_source(...)`，生产态不再回退到 `UiTemplateLoader`
- builtin host 文件 [`workbench_shell.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)、[`workbench_drawer_source.ui.toml`](../../zircon_editor/assets/ui/editor/host/workbench_drawer_source.ui.toml)、[`floating_window_source.ui.toml`](../../zircon_editor/assets/ui/editor/host/floating_window_source.ui.toml)、[`scene_viewport_toolbar.ui.toml`](../../zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml)、[`asset_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/asset_surface_controls.ui.toml)、[`pane_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml)、[`startup_welcome_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/startup_welcome_controls.ui.toml)、[`inspector_surface_controls.ui.toml`](../../zircon_editor/assets/ui/editor/host/inspector_surface_controls.ui.toml) 都已经改成 tree-shaped `UiAssetDocument`
- 对 legacy 兼容的需求现在只允许留在 [`UiLegacyTemplateAdapter`](../../zircon_runtime/src/ui/template/asset/legacy.rs) 与 editor test support，不再允许回流进 production runtime

当前本地验证直接锁住了这条边界：

- `cargo check -p zircon_editor`
- `cargo test -p zircon_editor boundary`
- `cargo test -p zircon_editor template`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput`

## Builtin Pane Body Projection

2026-04-24 的 pane body cutover 把首批 workbench pane 的 body authority 又从 Slint/Rust 手写 DTO 往 `.ui.toml -> PanePresentation -> Slint host projection` 推进了一层。`Console` 和 `Inspector` 已经走 template-only body；本轮补齐了 `Hierarchy`、`AnimationSequenceEditor` 和 `AnimationGraphEditor` 的 hybrid body 消费路径。

当前合同如下：

- [`hierarchy_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/hierarchy_body.ui.toml) 保留 `SelectionCommand.SelectSceneNode` route，并把树主体声明成 `hierarchy_tree_slot`；Slint 宿主只继续承载已有 tree native slot 与 pointer bridge。
- [`animation_sequence_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/animation_sequence_body.ui.toml) 保留 `AnimationCommand.ScrubTimeline` route，并把 timeline 主体声明成根级 `animation_timeline_slot`。
- [`animation_graph_body.ui.toml`](../../zircon_editor/assets/ui/editor/host/animation_graph_body.ui.toml) 保留 `AnimationCommand.AddGraphNode` route，并把 graph canvas 主体声明成根级 `animation_graph_canvas_slot`。
- [`pane_data_conversion.rs`](../../zircon_editor/src/ui/slint_host/ui/pane_data_conversion.rs) 现在对 hierarchy/animation 优先读取 `PanePresentation.body`，调用 `EditorUiHostRuntime::project_pane_body(...)` 注入 payload 与 hybrid slot anchor，再把 payload 还原成 Slint 现有 native view 所需的 rows、track、parameter、node、state 和 transition 数据。
- [`apply_presentation.rs`](../../zircon_editor/src/ui/slint_host/ui/apply_presentation.rs) 会把每个 dock/floating pane 的可见内容尺寸传给转换层，让 template body projection 至少拥有宿主 content bounds；dock/window 生命周期和 native pointer bridge 不在这一层重写。
- [`pane_content.slint`](../../zircon_editor/ui/workbench/pane_content.slint) 只消费 template projection 输出的 stable control id / anchor id，并继续把 hierarchy tree、timeline 和 graph canvas 的细交互交给原 native slot。

Task 10 的 host-side cleanup 把 [`PaneData`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs) 中原本平铺的 `Console` / `Inspector` / `Hierarchy` / `Animation` / asset body DTO 收进 `PaneBodyCompatData`。`PaneData` 现在的正式 body authority 是 `presentation: PanePresentation`；`body_compat` 只作为 Slint ABI、native slot 和未完全退场 pane 的兼容壳存在。对应地，[`pane_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs) 只在生成 `PanePresentation` 后填充兼容壳，[`scene_projection.rs`](../../zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs) 读取 compat 数据时也显式标注为宿主桥接，而不是继续把 giant union 当成结构真源。

验证覆盖：

- `cargo test -p zircon_editor --lib --locked template_runtime --offline` 锁住 builtin body document 注册、hybrid slot 根级声明、route namespace 和 payload projection。
- `cargo test -p zircon_editor --lib --locked slint_hierarchy_template_body --offline` 锁住 hierarchy body 从 payload 和 template anchor 投影到 Slint view data。
- `cargo test -p zircon_editor --lib --locked slint_animation_template_body --offline` 锁住 sequence timeline / graph canvas hybrid anchors 与 animation payload 投影。
- `cargo test -p zircon_editor --lib --locked slint_host --offline` 确认现有真实宿主桥仍然保持可运行。
- `cargo check -p zircon_editor --locked --offline` 确认 `PaneBodyCompatData` 收束后 production crate 仍然编译通过。

## Runtime Typography Metadata

M1 之后，template metadata 已经开始承担 runtime 文本底座所需的最小 typography 真源，而不再只有 `background/foreground/border/text/icon/opacity` 这类视觉占位字段。

`resolve.rs` 当前会把下面这些键直接解析进 `UiResolvedStyle`：

- `font`
- `font_family`
- `font_size`
- `line_height`
- `text_align`
- `wrap`
- `text_render_mode`

除了直写属性，还支持 `[font]` table 的同义字段，例如：

```toml
font = "res://fonts/default.font.toml"
font_family = "Fira Mono"
font_size = 18.0
line_height = 24.0
text_align = "center"
wrap = "word"
text_render_mode = "auto"
```

如果需要显式覆盖字体资产默认值，也可以写成：

```toml
text_render_mode = "sdf"
```

这组 metadata 现在已经有 capture 级证据，而不只是 shared tree / planner 级证据：[runtime_ui_text_render_contract.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_ui_text_render_contract.rs) 直接证明 `native` / `sdf` 两条文本路径都能把 template typography metadata 落到最终 glyph 输出，并且 `clip_frame`、`wrap` 和 `opacity` 都会继续进入最终 glyph 采样结果，而不是在 shared render contract 里中途丢失。

这份回归现在已经分成两类最终证据：

- 手写 `UiRenderCommand` 路径，证明 runtime text backend 本身不会把字形重新退化成占位矩形带
- 正式模板资产路径，证明 `.ui.toml -> UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder -> UiSurface.render_extract -> RenderFramework capture_frame(...)` 这条主链也能把 `wrap` 和 `opacity` 保到最终 glyph 像素差异上

这让 shared template runtime 的 typography 合同不再只是“字段已解析进 `UiResolvedStyle`”，而是已经证明正式资产链路能把这些字段保真送到最终 renderer。

editor viewport 那条 runtime-style HUD 现在也加入了这条 capture 证据链：[render_frame_submission_hud_text_renders_through_runtime_glyph_capture](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 直接把 `EditorState::render_frame_submission()` 产出的 `UiRenderExtract` 交给 runtime render framework capture，并用“有字 HUD / 去字 HUD”的像素差异证明 shared template/runtime 写出的 typography 字段没有在 editor 宿主路径里被旁路掉。

或者：

```toml
[font]
asset = "res://fonts/default.font.toml"
family = "Fira Mono"
size = 18.0
line_height = 24.0
align = "center"
wrap = "word"
render_mode = "native"
```

这里的默认语义已经固定下来：

- `UiResolvedStyle::text_render_mode` 默认值是 `Auto`
- `text_render_mode = "auto"` 表示把具体 native/sdf 选择延后到 runtime text backend
- text backend 会优先读取字体资产 manifest 的 `render_mode`
- 如果字体资产没有声明默认值，则稳定回落到 `Native`
- 如果模板样式显式写 `native` 或 `sdf`，显式样式优先于字体资产默认值

作为 M1 的默认可用闭环，`res://fonts/default.font.toml` 现在还带有一条更硬的资源归属规则：

- manifest 内部的 `source` 已收口到 `zircon_runtime/assets/fonts/FiraMono-subset.ttf`
- shared template runtime 继续只暴露字体资产引用，不把 dev tree 相对路径泄露进样式合同
- `default_runtime_font_manifest_stays_inside_runtime_assets` 会把这条默认资产归属锁成测试

在这之上，字体 manifest 解析边界也被收紧成正式合同：

- `source` 必须是相对路径，不接受绝对文件路径
- `res://` 字体 manifest 的 `source` 解析后必须仍落在 runtime `assets/` 根内
- 非 `res://` 的外部 manifest 也只能解析到 manifest 自己目录作用域内，不允许继续越级逃逸

这轮又把这份合同从“renderer 私有 loader”继续推成公共资产语义：

- `.font.toml` 现在有正式的 [`FontAsset`](../../zircon_runtime/src/asset/assets/font.rs) 模型，而不是只在 `graphics::...::font_asset` 内部保留匿名 TOML 结构
- runtime asset pipeline 已经把 `.font.toml` 识别成 `ImportedAsset::Font` / `AssetKind::Font`，所以 project scan、artifact store、runtime resource registry 都能看见字体资产
- [`font_asset.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs) 现在直接复用 `FontAsset::from_toml_str(...)`，并向 text backend 返回强类型 `UiTextRenderMode`
- [`ScreenSpaceUiTextSystem`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs) 现在会持有 `ProjectAssetManager`，因此 shared template/runtime 写出的 `font = "res://fonts/project.font.toml"` 不再只会回到 runtime crate 自带 `assets/`，而是能优先命中当前项目里正式导入的字体资产
- 项目里的原始字体二进制 `.ttf/.otf/.woff/.woff2` 现在被 [`collect_files.rs`](../../zircon_runtime/src/asset/project/manager/collect_files.rs) 视为 manifest source auxiliary，而不是独立 asset；这保证 shared template runtime 可以安全引用项目字体而不会把 `scan_and_import()` 直接炸掉

这样 shared template/runtime 样式仍然只持有 `font = "res://..."` 这类中性引用，但 runtime text backend 已经不再信任 manifest 内部的任意文件系统跳转。

这样 shared template runtime 现在已经能把“字体资源引用、字号、行高、对齐、换行、auto/native/sdf 选择”一路投影到 shared render contract，而不需要 editor/runtime 再各自做一套文本样式解释。

## Legacy Compat Model

### `UiTemplateDocument`

- `version`
- `components: BTreeMap<String, zircon_runtime::ui::template::UiComponentTemplate>`
- `root: zircon_runtime::ui::template::UiTemplateNode`

文档拥有一个真正的入口 root，以及一组可被重复装配的命名 component template。

### `zircon_runtime::ui::template::UiComponentTemplate`

- `root: zircon_runtime::ui::template::UiTemplateNode`
- `slots: BTreeMap<String, zircon_runtime::ui::template::UiSlotTemplate>`

component template 是“复合组件装配层”的最小权威单元。它不直接描述最终像素 frame，只描述宿主树和 shared tree 应该如何拼装。

### `zircon_runtime::ui::template::UiTemplateNode`

当前节点固定只有三种互斥形态：

- `component`
  - 表示一个真实宿主/共享组件节点
- `template`
  - 表示对命名 `zircon_runtime::ui::template::UiComponentTemplate` 的调用
- `slot`
  - 表示 template 内部的插槽占位点

额外携带的通用字段包括：

- `control_id`
- `bindings`
- `children`
- `slots`
- `attributes`
- `style_tokens`

这里的 `bindings` 目前不是宿主 callback 名称，而是稳定的 `zircon_runtime::ui::template::UiBindingRef`：

- `id`
- `event`
- `route`

`id` 用来承载诸如 `WorkbenchMenuBar/SaveProject` 这类稳定命名空间；`route` 只是稳定 route key，不是桌面宿主私有函数名。

`UiComponentTemplate` / `UiSlotTemplate` / `UiBindingRef` / `UiActionRef` 现在都统一经 `zircon_runtime::ui::template::*` 暴露，`zircon_runtime::ui` root 不再继续平铺这组 template document model。

## Legacy Template TOML Shape

当前实现支持的最小 TOML 形态如下：

```toml
version = 1

[root]
template = "WorkbenchShell"
slots = { menu_bar = [{ template = "MenuBar" }] }

[components.WorkbenchShell]
slots = { menu_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }] }

[components.MenuBar]
root = { component = "UiHostToolbar", children = [
  { component = "UiHostIconButton", control_id = "SaveProject", bindings = [
    { id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }
  ] }
] }
```

这个结构已经满足第一阶段目标：

- component template 可以嵌套 component template
- slot 内容由调用点提供
- binding 引用保留稳定命名空间
- 运行时实例展开后不会丢掉这些 binding ref

仓库里这组 builtin host 模板资产现在已经放在 [workbench_shell.ui.toml](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)。它先覆盖 workbench shell 的复合装配骨架：

- `WorkbenchShell`
- `MenuBar`
- `ActivityRail`
- `DocumentHost`
- `StatusBar`

## Layout Contract By Attribute

当前模板文档不会额外引入第二套 layout 节点类型，而是固定通过 `attributes.layout` 把 shared contract 写进模板节点。

已落地的字段包括：

- `width` / `height`
  - 对应 shared `AxisConstraint { min, max, preferred, priority, weight, stretch }`
- `anchor` / `pivot` / `position`
  - 直接映射到 shared `Anchor` / `Pivot` / `Position`
- `boundary`
  - 对应 `LayoutBoundary::{ContentDriven, ParentDirected, Fixed}`
- `clip` / `clip_to_bounds`
  - 控制节点 clip 链入口
- `z_index`
  - 控制 shared draw order 的层级偏置
- `input_policy`
  - 支持 `Inherit` / `Receive` / `Ignore`
- `container`
  - 显式声明 shared 容器语义，而不是强依赖 component 名字

`container` 目前支持：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox { gap }`
- `VerticalBox { gap }`
- `ScrollableBox { axis, gap, scrollbar_visibility, virtualization }`

这一步的关键点是：editor-only component 名字不再需要和 shared primitive 名字一一重合。像 `WorkbenchShell`、`DocumentHost`、`ActivityRail` 这样的 composite，可以继续保留自己的宿主身份，但它们的 shared layout 行为已经由 `attributes.layout.container` 显式给出。

## Validation Rules

`UiTemplateValidator` 当前已经把以下约束钉死：

- 每个节点必须且只能声明 `component` / `template` / `slot` 其中一种
- `template` 调用必须引用已注册的 component template
- required slot 必须由调用点提供
- 不允许给单值 slot 塞多个子节点
- template 内部出现的 slot placeholder 必须先在 `slots` 中声明
- slot placeholder 不能再额外携带 bindings、children、slot fills 或 control id
- template 调用不允许直接再挂 `children`，slot 才是唯一的复合内容注入口

这一步的意义是避免 editor host 或后续 Slint projection 再去容忍一堆“能跑但不清晰”的隐式模板结构。

## Instance Expansion

`zircon_runtime::ui::template::UiTemplateInstance::from_document(...)` 当前会：

- 先跑完整 `UiTemplateValidator`
- 再把 `template` 调用展开成真实 component 子树
- 再把 slot placeholder 替换成调用点提供的内容
- 最终得到一个已经没有 `template`/`slot` 占位歧义的运行时模板实例树

目前实例层还提供 `binding_refs()`，按树遍历顺序收集稳定 binding 引用。这正是后续 editor/runtime adapter 把模板树映射成 typed command/binding、再投影给 Slint host 的入口。

## Shared Tree Bridge

这一轮新增了 shared-core 桥接器：

- `UiTemplateTreeBuilder`
- `UiTemplateSurfaceBuilder`
- `UiTemplateBuildError`
- `zircon_runtime::ui::tree::UiTemplateNodeMetadata`

### `zircon_runtime::ui::tree::UiTemplateNodeMetadata`

`UiTreeNode` 现在可以携带模板元数据快照，用来保留后续 shared core 和宿主投影都会需要的稳定信息：

- `component`
- `control_id`
- `attributes`
- `style_tokens`
- `bindings`

这一步很关键，因为之前如果直接把模板实例铺进 `UiTree`，会丢掉：

- 稳定 binding id
- 宿主 icon / label 等属性
- style token
- component/control identity

那样后续再想从 shared tree 做 route 或宿主投影，就必须重新回头读模板实例，等于 shared tree 不是真正的中继层。

### `UiTemplateTreeBuilder`

当前 builder 会把 `zircon_runtime::ui::template::UiTemplateInstance` 转成 `UiTree`，并做两类 shared-core 推断：

- 节点按 preorder 分配稳定 `UiNodeId`
- `UiNodePath` 使用 control/component 名称加顺序索引生成可读路径
- 模板元数据挂入每个 `UiTreeNode`
- 显式 `attributes.layout` 映射到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / `LayoutBoundary` / `UiInputPolicy` / `z_index`
- `attributes.layout.container` 优先映射到 `UiContainerKind`
- 当模板没有显式 `container` 时，再退回到已知共享容器名映射
- 可交互节点根据 bindings / 已知交互 primitive 推断 `clickable` / `hoverable` / `focusable`
- 带 bindings 的节点默认设置 `UiInputPolicy::Receive`
- `ScrollableBox` 自动初始化 `UiScrollState::default()` 并开启 `clip_to_bounds`

当前 layout contract 采用“显式字段优先、组件名仅作回退”的规则。也就是说：

- 如果模板写了 `attributes.layout.container.kind = "VerticalBox"`，shared tree 就直接按 `VerticalBox` 处理
- 如果模板没写 layout 容器，但 component 名字本身就是 `HorizontalBox` / `ScrollableBox` 这类 shared primitive，builder 仍然会做兼容映射
- 如果两者都没有，节点保持 `UiContainerKind::Free`

对于 layout 字段值，builder 现在会在 bridge 阶段做基本结构校验；不支持的 enum 值或错误的 table 形态会直接返回 `UiTemplateBuildError::InvalidLayoutContract`，避免把畸形模板延后到 layout pass 或宿主投影时才暴露。

当前已知容器映射只覆盖 shared primitive 名称：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox`
- `VerticalBox`
- `ScrollableBox`

未知 component 目前不会被强行解释成布局容器，而是保留 `UiContainerKind::Free`。

### `UiTemplateSurfaceBuilder`

`UiTemplateSurfaceBuilder` 只是 `UiTemplateTreeBuilder` 的轻封装：

- 先构建 `UiTree`
- 再放入 `UiSurface`
- 最后调用 `rebuild()` 生成 hit-test index 和初始 `UiRenderExtract`

这让 shared template runtime 现在已经具备了“模板实例 -> shared retained surface -> shared layout 求 frame -> shared visual draw list”的最低闭环，而不是停留在纯文档/纯实例层。

当前 `rebuild()` 输出的 `UiRenderExtract` 会直接把模板属性里已经 resolved 的视觉字段带出来，而不再只保留几何：

- `background` / `foreground` / `border` 会落到 `UiResolvedStyle`
- `text` / `label` 会落到 render command 的 `text`
- `icon` / `image` 会落到 `UiVisualAssetRef`
- `opacity` 会落到 render command 的 `opacity`

这意味着 style asset 和 inline override 在 template compiler 里完成归并以后，shared surface 已经能把这些视觉结果继续传给 preview/runtime consumer；后续还没做的部分是文本测量、字体 atlas、图片资源装载和真实 GPU pass，而不是再回头重建另一套 visual payload 模型。

## Current Scope And Deliberate Gaps

这一轮刻意没有把以下能力塞进 `zircon_runtime::ui::template`：

- Slint host tree 自动投影
- repeat/tree data projection
- 样式 token 继承/覆盖求值
- 模板参数求值和表达式系统
- 文本/图片测量服务
- 基于样式 token 或表达式的动态 layout 合同求值
- runtime widget 级 visual primitive 的完整模板化

原因很直接：这里先锁住模板装配契约，并把显式 layout 合同落进 shared tree，但不在 token、表达式、测量服务都还没定型之前就发明第二层隐式布局公式。

## Why This Boundary Matters

如果没有这层共享模板语义，后续 editor 迁移很容易退回两条错误路线：

- 让 Slint `.slint` 业务树继续做真正的模板权威
- 或者在 `zircon_editor` 里直接把 WorkbenchLayout/ViewModel 拼成另一套 host-only 树

现在 `zircon_runtime::ui::template` 已经先把文档、slot、binding 命名、运行时实例展开，以及 shared tree 的第一段桥接统一下来。后续无论是 runtime UI 还是 editor shell，都必须从同一份模板真源继续向 shared layout 求解和宿主投影层推进。

## Builtin Root Document Identity

`zircon_editor` 这一轮又把 shared template runtime 的 builtin root 文档身份往 generic host 边界推进了一步：

- builtin root host 模板现在以 `ui.host_window` 作为首选 `document_id`
- 旧的 `workbench.shell` 仍作为兼容 alias 同时注册到同一份 [`workbench_shell.ui.toml`](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml)
- `UiHostWindow` 相关 component descriptor 也同步改成指向 `ui.host_window`
- `EditorUiHostRuntime` 新增 generic `load_builtin_host_templates()`，把“加载一组 builtin host template”与“加载 workbench shell”两个概念拆开
- [`zircon_editor/ui/workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的导出 root 现在也已经跟着这个 identity 收口：`UiHostWindow` 只剩 window/bootstrap wrapper，bootstrap 符号统一为 `UiHostContext`、`UiHostScaffold` 和 `HostWindowSceneData`
- host presentation / scene contract 也同步去掉首批 workbench 专名：`workbench_scene_data` 已改为 `host_scene_data`，`HostWorkbenchSurfaceMetricsData` / `HostWorkbenchSurfaceOrchestrationData` 已改为 `HostWindowSurfaceMetricsData` / `HostWindowSurfaceOrchestrationData`
- host drag / resize pointer event 也已经从 `workbench_drag_pointer_event` / `workbench_resize_pointer_event` 收口为 `host_drag_pointer_event` / `host_resize_pointer_event`
- Rust host-page pointer helper 也已经从 `WorkbenchHostPagePointer*` / `build_workbench_host_page_pointer_layout` 收口为 `HostPagePointer*` / `build_host_page_pointer_layout`
- Rust menu pointer helper 也已经从 `WorkbenchMenuPointer*` / `build_workbench_menu_pointer_layout` 收口为 `HostMenuPointer*` / `build_host_menu_pointer_layout`
- Rust activity rail pointer helper 也已经从 `WorkbenchActivityRailPointer*` / `build_workbench_activity_rail_pointer_layout` 收口为 `HostActivityRailPointer*` / `build_host_activity_rail_pointer_layout`
- Rust document tab pointer helper 也已经从 `WorkbenchDocumentTabPointer*` / `build_workbench_document_tab_pointer_layout` 收口为 `HostDocumentTabPointer*` / `build_host_document_tab_pointer_layout`
- Rust drawer header pointer helper 也已经从 `WorkbenchDrawerHeaderPointer*` / `build_workbench_drawer_header_pointer_layout` 收口为 `HostDrawerHeaderPointer*` / `build_host_drawer_header_pointer_layout`
- Rust shell pointer helper 也已经从 `WorkbenchShellPointer*` / `workbench_shell_pointer_*` 收口为 `HostShellPointer*` / `host_shell_pointer_*`
- Rust tab drag helper 也已经从 `WorkbenchDragTarget*` / `ResolvedWorkbenchTabDrop*` / `resolve_workbench_*` 收口为 `HostDragTarget*` / `ResolvedHostTabDrop*` / `resolve_host_*`
- Rust resize target helper 也已经从 `WorkbenchResizeTargetGroup` / `resolve_workbench_resize_target_group` 收口为 `HostResizeTargetGroup` / `resolve_host_resize_target_group`
- Rust root shell frames DTO 也已经从 `BuiltinWorkbenchRootShellFrames` / `workbench_body_frame` 收口为 `BuiltinHostRootShellFrames` / `host_body_frame`
- Rust builtin host projection builder 也已经从 `build_builtin_workbench_host_projection` 收口为 `build_builtin_host_window_projection`
- Rust drawer source bridge 也已经从 `BuiltinWorkbenchDrawerSource*` / `build_builtin_workbench_drawer_source_surface` 收口为 `BuiltinHostDrawerSource*` / `build_builtin_host_drawer_source_surface`
- 内置 template runtime 现在会递归解析 `res://...` widget imports，并允许 layout root 通过 `#RootControlId` 作为嵌入组件使用
- `root_workbench_slint_exports_only_generic_host_bootstrap_symbols` 会守住 root 文件不再回到 `WorkbenchHostContext`、`WorkbenchHostScaffold` 或 `HostWorkbenchWindowSceneData`
- `slint_host_presentation_uses_generic_scene_data_property` 与 `slint_host_scene_uses_generic_surface_metrics_and_orchestration_names` 会守住 host scene DTO 不再回到 workbench 专名
- `slint_host_drag_and_resize_callbacks_use_generic_host_event_names` 会守住通用拖拽/resize 宿主事件不再回到 workbench 专名
- `host_page_pointer_module_uses_generic_host_type_names` 会守住 host-page pointer helper 不再回到 workbench 专名
- `menu_pointer_module_uses_generic_host_type_names` 会守住 menu pointer helper 不再回到 workbench 专名
- `activity_rail_pointer_module_uses_generic_host_type_names` 会守住 activity rail pointer helper 不再回到 workbench 专名
- `document_tab_pointer_module_uses_generic_host_type_names` 会守住 document tab pointer helper 不再回到 workbench 专名
- `drawer_header_pointer_module_uses_generic_host_type_names` 会守住 drawer header pointer helper 不再回到 workbench 专名
- `shell_pointer_module_uses_generic_host_type_names` 会守住 shell pointer helper 不再回到 workbench 专名
- `tab_drag_module_uses_generic_host_type_names` 会守住 tab drag helper 不再回到 workbench 专名
- `drawer_resize_module_uses_generic_host_type_names` 会守住 resize target helper 不再回到 workbench 专名
- `root_shell_frames_use_generic_host_type_names` 会守住 root shell frames DTO 不再回到 workbench 专名
- `drawer_source_bridge_uses_generic_host_type_names` 会守住 drawer source bridge 不再回到 workbench 专名
- 这层 wrapper 目前仍通过属性别名、typed event 名称和 callback forwarding 暂时保留部分 workbench 业务 ABI，因此 shared template/runtime 的 generic root identity 可以先稳定下来，而不需要一次性重写所有 host/slint 业务接线

这样 shared template runtime 对外暴露的默认 root 入口已经不再是 workbench 业务名；workbench 只剩兼容标签，而不是 builtin host root 的唯一 canonical identity。后续继续做 `Generic host boundary` 时，就可以在不改 shared runtime 主入口命名的前提下，逐步削掉 `workbench.slint` 和 builtin projection 里的业务壳结构。
