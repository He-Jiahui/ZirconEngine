---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/root_template_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/root_shell_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/reference/workbench.png
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_template.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/retained_window/shell_window.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs
  - docs/ui-and-layout/workbench.png
  - docs/ui-and-layout/editor-workbench-design-export.md
  - tools/editor-workbench-preview/design.html
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/preview-sheet.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/export-options.mjs
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/verify-reference-negative-guard.mjs
  - tools/editor-workbench-preview/package.json
  - zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
  - zircon_editor/src/ui/workbench/layout/activity_drawer_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.ui.toml
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_editor/src/ui/retained_host/tab_drag.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_layout_paths.rs
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - dev/bevy/crates/bevy_ui/src/lib.rs
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/bevy/crates/bevy_text/src/lib.rs
  - dev/bevy/crates/bevy_ui_widgets/src/lib.rs
  - dev/bevy/crates/bevy_feathers/src/lib.rs
  - dev/bevy/crates/bevy_a11y/src/lib.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_editor/assets/ui/editor/material_meta_components.ui.toml
  - dev/material-rust-template/material-1.0/material.slint
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/layout/metrics.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_runtime_interface/src/ui/layout/linear_sizing.rs
  - zircon_runtime_interface/src/tests/ui_layout.rs
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/template/build/slot_contract.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/arranged.rs
  - zircon_runtime/src/ui/surface/diagnostics.rs
  - zircon_editor/src/ui/workbench/debug_reflector/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/debug_reflector_overlay.rs
  - zircon_runtime_interface/src/ui/surface/arranged.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime_interface/src/ui/surface/frame.rs
  - zircon_runtime_interface/src/ui/surface/diagnostics.rs
  - zircon_runtime_interface/src/ui/tree/node/visibility.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/root_template_overlay.rs
  - zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs
  - zircon_editor/src/ui/retained_host/root_shell_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/retained_host/mod.rs
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/host/pane_surface_controls.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
  - zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml
  - zircon_editor/assets/ui/editor/reference/workbench.png
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/layout_routes.rs
  - zircon_editor/tests/integration_contracts/workbench_retained_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_template.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/retained_window/shell_window.rs
  - zircon_editor/src/tests/host/retained_window/native_workbench_reference.rs
  - docs/ui-and-layout/workbench.png
  - docs/ui-and-layout/editor-workbench-design-export.md
  - tools/editor-workbench-preview/design.html
  - tools/editor-workbench-preview/design.css
  - tools/editor-workbench-preview/design.js
  - tools/editor-workbench-preview/design-manifest.mjs
  - tools/editor-workbench-preview/export-options.mjs
  - tools/editor-workbench-preview/export-designs.mjs
  - tools/editor-workbench-preview/verify-designs.mjs
  - tools/editor-workbench-preview/verify-reference-negative-guard.mjs
  - tools/editor-workbench-preview/package.json
  - zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml
  - zircon_editor/src/ui/workbench/layout/activity_drawer_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/region_frames.rs
  - zircon_editor/src/ui/workbench/autolayout/geometry/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.ui.toml
  - zircon_editor/assets/ui/editor/project_overview.ui.toml
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/build_compiled_scene_draws.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_scene/render_scene.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_editor/src/ui/retained_host/tab_drag.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/generic_host_layout_paths.rs
  - docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md
  - docs/ui-and-layout/slint-material-retained-editor-migration.md
  - zircon_editor/assets/ui/theme/editor_material.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
plan_sources:
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-20 zircon_editor UI 回迁 + 树形 TOML cutover 继续清理 root 出口与旧 UI crate 文档路径
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Bevy 对齐的 Zircon UI Text Widgets Focus A11y 里程碑计划.md
  - user: 2026-05-20 migrate Slint Material component behavior into retained Editor UI without direct Slint runtime
  - docs/superpowers/specs/2026-05-20-slint-material-retained-editor-migration-design.md
  - docs/superpowers/plans/2026-05-20-slint-material-retained-editor-migration.md
  - user: 2026-05-29 use docs/ui-and-layout/workbench.png as the exact editor workbench baseline
  - .codex/plans/Editor Workbench PNG Design Plan.md
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_editor/src/tests/ui/activity/mod.rs
  - zircon_editor/src/tests/ui/activity/window_descriptor.rs
  - zircon_editor/src/tests/ui/activity/route.rs
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short
  - cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short
  - cargo test -p zircon_editor --lib apply_presentation_prefers_drawer_derived_viewport_when_pane_surface_is_stale --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib clustered_lighting_does_not_tint_final_frame_by_tile_buffer --locked --jobs 1 -- --nocapture
  - cargo build -p zircon_app --bin zircon_editor --features target-editor-host --no-default-features --locked --jobs 1
  - visual screenshot: target/visual-layout/editor-window-20260427-after-cluster-tile-fix.png
  - visual screenshot: target/visual-layout/editor-window-20260427-layout-continue-header-fix.png
  - visual screenshot: target/visual-layout/editor-window-20260428-toolbar-no-viewport-hud.png
  - visual screenshot: target/visual-layout/editor-window-20260428-project-buttons-runtime-restart-crop.png
  - visual screenshot: target/visual-layout/editor-window-20260428-scene-toolbar-compact.png
  - visual screenshot: target/visual-layout/editor-window-20260428-layout-continue-final-large.png
  - visual screenshot: target/visual-layout/editor-window-20260428-bottom-compact-max-small.png
  - visual screenshot: target/visual-layout/editor-window-20260428-bottom-compact-max-large.png
  - visual screenshot: target/visual-layout/editor-window-20260428-side-compact-asymmetric-1024x700.png
  - visual screenshot: target/visual-layout/editor-window-20260428-side-compact-240-1616x1019.png
  - visual screenshot: target/visual-layout/editor-window-20260428-project-vertical-compact-800x620.png
  - visual screenshot: target/visual-layout/editor-window-20260428-project-vertical-compact-1024x700.png
  - visual screenshot: target/visual-layout/editor-window-20260428-catalog-actions-first-700x620.png
  - visual screenshot: target/visual-layout/editor-window-20260428-catalog-actions-first-900x500.png
  - visual screenshot: target/visual-layout/editor-window-20260428-catalog-actions-first-1280x520.png
  - cargo test -p zircon_runtime --locked
  - cargo test -p zircon_runtime_interface --lib ui_visibility_contract_separates_layout_render_and_hit_policy --locked --target-dir E:\zircon-build\targets\slate-ui-framework
  - cargo test -p zircon_runtime --lib hit_grid_respects_slate_visibility_and_clip_semantics --locked --target-dir E:\zircon-build\targets\slate-ui-framework
  - cargo test -p zircon_runtime --lib diagnostics --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo check --workspace --locked
  - docs-only validation: git diff --check docs/ui-and-layout/bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md docs/ui-and-layout/index.md
  - zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - rustfmt --edition 2021 --check zircon_editor/src/tests/ui/boundary/slint_material_retained_editor_migration.rs
  - cargo metadata --locked --no-deps --format-version 1
  - cargo test -p zircon_editor --lib build_script_tracks_editor_assets_not_deleted_ui_sources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_reference_visual_asset_matches_docs_baseline --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib runtime_component_projection_loads_workbench_reference_image_preview --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib builtin_editor_host_templates_export_layout_engine_route_reports --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib workbench_shell_window_starts_at_reference_size_and_can_resize --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib host_projection_converts_workbench_reference_image_to_root_overlay_node --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib host_window_template_bridge_projects_workbench_reference_overlay_node --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib native_host_window_snapshot_draws_workbench_reference_overlay_pixels --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib apply_presentation_projects_workbench_reference_overlay_from_host_template_bridge --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib apply_presentation_snapshot_matches_workbench_reference_from_host_template_bridge --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib full_command_stream_replays_workbench_reference_overlay_pixels --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib patch_command_stream_repaints_workbench_reference_overlay_damage_pixels --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts workbench_window_uses_reference_image_baseline --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --test integration_contracts --features integration-contracts workbench_shell_assets_replace_deleted_shell_sources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-workbench-reference-0529 --message-format short --color never -- --nocapture
  - npm --prefix tools/editor-workbench-preview run design:verify
  - npm --prefix tools/editor-workbench-preview run design:verify:reference-negative
doc_type: category-index
---

# UI And Layout

## Purpose

本目录记录运行时 UI 与 editor shell 共用的布局、树结构、命中和 surface 权威模型，重点回答三件事：

- 哪些布局/几何/命中语义已经属于 `zircon_runtime::ui`
- editor workbench 还保留哪些 editor-only 壳体职责
- Rust-owned host contract、legacy-named `retained_host` glue 和未来其他宿主如何只做适配层，而不是再次成为布局真源

## Hub Visual Design Artifacts

`hub.png` remains the Projects Dashboard pixel reference for the Hub visual system. The companion design PNGs are AI-directed, HTML/CSS-finalized reference artifacts on the same `1568x1003` canvas. `docs/ui-and-layout/hub-ai-reference-manifest.json` records the source reference, prompt family, selected draft role, page ids, AI structure-layout direction drafts, and final output filenames. `docs/ui-and-layout/hub-ai-reference-manifest.schema.json` is the manifest structure contract; `docs/ui-and-layout/hub-web-reference/validate-visuals.mjs` runs schema subset validation plus negative schema self-tests for extra fields, missing reference output, short reference inventory, malformed AI draft paths, and canvas drift. Those AI drafts are for reviewing the overall interaction structure first: navigation, page regions, toolbars, card/table grids, side panels, dropdowns, modals, and state overlays. local functional-content callouts are secondary and only clarify page intent. `docs/ui-and-layout/hub-web-reference/export-pages.mjs` captures the browser-openable web reference, so final copy, icons, spacing, menus, overlays, and state panels are owned by real HTML/CSS instead of AI text rendering. The retired `tools/generate-hub-design-assets.py` Pillow generator must not be restored as the authoritative Hub PNG source.

Supplemental design-mode sheets live beside the final references as `hub-design-structure-layout.png`, `hub-design-structure-supplement.png`, and `hub-design-functional-details.png`. They are review aids for overall structure layout and local functional-content details, not final `hub-*.png` acceptance artifacts. Their reproducible source is `docs/ui-and-layout/hub-design-board/index.html`; refresh them with `node docs/ui-and-layout/hub-design-board/export-design-board.mjs`, then validate the design-board manifest, coverage matrix, checklist, canvas size, and browser fit with `node docs/ui-and-layout/hub-design-board/validate-design-board.mjs`.

- Main pages: `hub-editor.png`, `hub-builds.png`, `hub-assets.png`, `hub-plugins.png`, `hub-cloud.png`, `hub-team.png`, `hub-learn.png`, `hub-settings.png`
- Projects flows: `hub-projects-new.png`, `hub-projects-browser.png`, `hub-projects-detail.png`
- Interactive states: `hub-projects-browser-filter-menu.png`, `hub-projects-browser-sort-menu.png`, `hub-projects-detail-delete-confirm.png`, `hub-source-engine-popup.png`, `hub-user-menu.png`
- Global states: `hub-state-empty.png`, `hub-state-loading.png`, `hub-state-error.png`

The three global-state artifacts are not standalone navigation pages. They are shared visual baselines for empty, loading, and recoverable-error surfaces across Hub routes, so their responsive evidence is documented through the shared component contracts and page-specific runtime captures rather than a separate breakpoint screenshot for each state.

The web reference can be replayed without starting Hub. Open `docs/ui-and-layout/hub-web-reference/index.html?page=<page_id>` to inspect a single finalized reference page, or run `node docs/ui-and-layout/hub-web-reference/export-pages.mjs --only=<page_id>` to refresh selected design PNGs while also updating the dashboard web capture. The exporter uses port 5198 by default, falls back to a free local port when that default is occupied, and keeps explicit `ZIRCON_HUB_WEB_REFERENCE_PORT` strict so scripted runs fail instead of silently changing a requested port. Page ids match the design PNG basename for every `hub-*.png` artifact; the fixed source screenshot `hub.png` remains the Projects Dashboard pixel reference, while the web dashboard page id is `projects-dashboard`.

After changing the static shell, run `node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs` as part of the web-reference acceptance pass. `docs/ui-and-layout/hub-web-reference/validate-interactions.mjs` opens the dashboard plus every exported page, rejects unknown `data-route` and `href="?page=..."` targets, verifies exported PNG filename replay, verifies every replay path listed in `EXPORTS.md`, and checks representative header, project, toolbar, and Quick Actions click routes.

Representative manual visual spot checks for the design set are recorded in `docs/ui-and-layout/hub-web-reference/SPOT_CHECKS.md`. That checklist covers the fixed dashboard source plus Editor, Assets, Project Detail delete confirmation, Source Engine popup, Empty, and Error states, so reviewers have a stable first-pass inspection set before checking the full 19-page export.

The current web-reference closeout evidence is recorded in `docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md`. It lists the export command, the generated 19-file final PNG inventory, the visual/interaction/design-board validation commands, the focused `cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract` result, export port fallback evidence, manifest schema and negative schema self-tests, interaction temp-profile cleanup evidence, and the known optional `cargo check` timeout under concurrent Hub library compilation.

Runtime visual evidence for this design set is captured separately under `target/hub-visual-check-final/`: Projects Dashboard/New/Browser/Detail and project menus live at the root, main navigation captures live under `main-pages/`, top-header popup captures live under `popups/`, and global-state runtime captures live under `states/`. The AI structure-layout direction drafts are not acceptance evidence; the HTML/CSS web captures are documentation baselines, and runtime screenshots remain the acceptance evidence for actual Slint rendering.

Responsive runtime evidence for the same visual system is captured under `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/`. Each breakpoint includes Projects flow, Browser menu, delete-confirm, main navigation, Source Engine popup, and user menu PNGs; the 2026-05-29 check verified the images match their breakpoint dimensions, are non-empty, and use real isolated project paths for delete-confirm validation.

Visual acceptance for Hub pages requires the matching design PNG and runtime capture to preserve the same component density, no overlapping UI, no clipped text or controls, consistent button and badge states, stable panel hierarchy, and the shared empty/loading/error styling. Any future Hub screenshot set should record these checks beside the capture path instead of treating file existence alone as acceptance.

### Hub Visual Artifact Matrix

| Design PNG | Page or state | 1568x1003 runtime evidence | Responsive runtime evidence |
| --- | --- | --- | --- |
| `hub.png` | Projects Dashboard pixel reference | `target/hub-visual-check-final/hub-projects-dashboard.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-dashboard.png` |
| `hub-editor.png` | Editor main page | `target/hub-visual-check-final/main-pages/hub-editor.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-editor.png` |
| `hub-builds.png` | Builds main page | `target/hub-visual-check-final/main-pages/hub-builds.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-builds.png` |
| `hub-assets.png` | Assets main page | `target/hub-visual-check-final/main-pages/hub-assets.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-assets.png` |
| `hub-plugins.png` | Plugins main page | `target/hub-visual-check-final/main-pages/hub-plugins.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-plugins.png` |
| `hub-cloud.png` | Packages/Cloud main page | `target/hub-visual-check-final/main-pages/hub-cloud.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-cloud.png` |
| `hub-team.png` | Team main page | `target/hub-visual-check-final/main-pages/hub-team.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-team.png` |
| `hub-learn.png` | Learn main page | `target/hub-visual-check-final/main-pages/hub-learn.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-learn.png` |
| `hub-settings.png` | Settings main page | `target/hub-visual-check-final/main-pages/hub-settings.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-settings.png` |
| `hub-projects-new.png` | Projects New Project page | `target/hub-visual-check-final/hub-projects-new-project.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-new-project.png` |
| `hub-projects-browser.png` | Project Browser page | `target/hub-visual-check-final/hub-projects-browser.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser.png` |
| `hub-projects-detail.png` | Project Detail page | `target/hub-visual-check-final/hub-projects-detail.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-detail.png` |
| `hub-projects-browser-filter-menu.png` | Project Browser filter popup | `target/hub-visual-check-final/hub-projects-browser-filter-menu.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser-filter-menu.png` |
| `hub-projects-browser-sort-menu.png` | Project Browser sort popup | `target/hub-visual-check-final/hub-projects-browser-sort-menu.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser-sort-menu.png` |
| `hub-projects-detail-delete-confirm.png` | Project Detail delete confirmation | `target/hub-visual-check-final/hub-projects-detail-delete-confirm.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-detail-delete-confirm.png` |
| `hub-source-engine-popup.png` | Header Source Engine popup | `target/hub-visual-check-final/popups/hub-source-engine-popup.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-source-engine-popup.png` |
| `hub-user-menu.png` | Header user menu popup | `target/hub-visual-check-final/popups/hub-user-menu.png` | `target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-user-menu.png` |
| `hub-state-empty.png` | Shared empty-state baseline | `target/hub-visual-check-final/states/hub-state-empty.png` | Covered by shared `EmptyStateBlock`/`EmptyStatePanel` contracts and page-specific responsive captures |
| `hub-state-loading.png` | Shared loading-state baseline | `target/hub-visual-check-final/states/hub-state-loading.png` | Covered by `TaskStatus::running_operation`, status/task contracts, and responsive Builds captures |
| `hub-state-error.png` | Shared recoverable-error baseline | `target/hub-visual-check-final/states/hub-state-error.png` | Covered by `TaskStatus::error`, `StatusBanner`, and responsive guarded/error-flow captures |

## Editor Workbench Design Artifacts

`workbench.png` remains the editor workbench density and scale reference. The generated companion design PNGs under `editor-workbench-designs/` use the same `1672x941` canvas and are regenerated by `npm --prefix tools/editor-workbench-preview run design:export`, so future editor pages and windows can be compared against one deterministic dark, dense, teal-accented UI standard using JetBrains-like main tabs, drawer-style tool windows, and flatter rounded controls.

The retained editor shell now routes both the real first-screen document `ui.host_window` and the inner `editor.window.workbench` document through the same copied reference asset, `zircon_editor/assets/ui/editor/reference/workbench.png`, which is byte-identical to `docs/ui-and-layout/workbench.png`. `UiHostWindow::new` also starts the retained host at `1672x941`, matching the reference PNG canvas for the default first frame. `BuiltinHostWindowTemplateBridge::host_projection()` now feeds `HostWindowPresentationData.root_template_nodes` through `root_template_overlay`, and the native host painter draws that root template overlay after the normal shell scene so `WorkbenchShellReferenceImage` becomes the final first-screen visual layer. `apply_presentation_projects_workbench_reference_overlay_from_host_template_bridge` covers the production presentation seam directly: it passes the real `BuiltinHostWindowTemplateBridge::host_projection()` into `apply_presentation` and asserts that `HostWindowPresentationData.root_template_nodes` receives the full-frame `Image` overlay with the `1672 x 941` preview. `apply_presentation_snapshot_matches_workbench_reference_from_host_template_bridge` then keeps the same production seam on the pixel path by taking a native `UiHostWindow` snapshot after `apply_presentation` and comparing it to `docs/ui-and-layout/workbench.png`. `workbench_reference_visual_asset_matches_docs_baseline` locks the builtin template registry so `ui.host_window` continues to load `workbench_shell.v2.ui.toml` and `editor.window.workbench` continues to load `workbench_window.v2.ui.toml`; `build_script_tracks_editor_assets_not_deleted_ui_sources` keeps the copied PNG under the recursively tracked editor `assets` tree, `workbench_shell_window_starts_at_reference_size_and_can_resize` locks the startup window size without removing resize behavior, and `native_host_window_snapshot_draws_workbench_reference_overlay_pixels` verifies the native host snapshot is pixel-identical to `docs/ui-and-layout/workbench.png` when the overlay is active. The presenter command-stream tests also cover the same root overlay: `full_command_stream_replays_workbench_reference_overlay_pixels` checks that full command-stream replay records an image payload whose RGBA bytes match the reference and renders the same pixels as the legacy painter, while `patch_command_stream_repaints_workbench_reference_overlay_damage_pixels` keeps damage-region replay aligned with the legacy regional repaint path.

`root_template_overlay` projects `WorkbenchShellReferenceImage` as a pure image overlay: it carries only the host node identity, role, frame, media source, and preview image, while leaving text/value/options/icon and interaction state empty and dropping the source template clip. This keeps the TOML image `value` from being drawn as a label over the bitmap and keeps the full-window reference image on the same painter path as the hand-authored native overlay.

- Core workbench pages: `scene-workbench.png`, `hierarchy-workbench.png`, `inspector-workbench.png`, `asset-browser-workbench.png`, `console-workbench.png`, `project-overview-workbench.png`
- Tool/window pages: `material-lab-workbench.png`, `ui-asset-editor-workbench.png`, `animation-workbench.png`, `performance-workbench.png`, `runtime-diagnostics-workbench.png`, `plugin-manager-workbench.png`, `build-export-workbench.png`, `welcome-workbench.png`
- Additional editor pages: `prefab-editor-workbench.png`, `vfx-editor-workbench.png`, `shader-editor-workbench.png`, `terrain-editor-workbench.png`, `audio-editor-workbench.png`, `behavior-tree-workbench.png`, `lighting-bake-workbench.png`, `physics-collision-workbench.png`, `level-streaming-workbench.png`, `sequencer-workbench.png`, `navmesh-ai-workbench.png`, `render-pipeline-workbench.png`, `input-mapping-workbench.png`, `data-table-workbench.png`, `network-replication-workbench.png`, `localization-workbench.png`
- Collaboration/release pages: `source-control-workbench.png`, `review-comments-workbench.png`, `build-farm-workbench.png`, `release-notes-workbench.png`, `project-settings-workbench.png`, `plugin-development-workbench.png`, `remote-device-workbench.png`, `session-sync-workbench.png`
- Cinematic/animation pages: `cutscene-editor-workbench.png`, `dialogue-editor-workbench.png`, `quest-editor-workbench.png`, `camera-rig-workbench.png`, `control-rig-workbench.png`, `motion-matching-workbench.png`, `facial-animation-workbench.png`, `blend-space-workbench.png`
- World-building/environment pages: `foliage-editor-workbench.png`, `scatter-editor-workbench.png`, `volume-editor-workbench.png`, `weather-editor-workbench.png`, `post-process-workbench.png`, `particle-library-workbench.png`, `collision-proxy-workbench.png`, `level-variant-workbench.png`
- Gameplay/runtime pages: `gameplay-ability-workbench.png`, `gameplay-effect-workbench.png`, `ai-perception-workbench.png`, `spawn-rules-workbench.png`, `gameplay-tags-workbench.png`, `save-data-workbench.png`, `world-state-workbench.png`, `telemetry-dashboard-workbench.png`
- Platform/online pages: `lobby-editor-workbench.png`, `matchmaking-editor-workbench.png`, `server-browser-workbench.png`, `replay-browser-workbench.png`, `achievements-editor-workbench.png`, `entitlements-editor-workbench.png`, `user-profile-editor-workbench.png`, `online-diagnostics-workbench.png`
- UI/UX pages: `hud-editor-workbench.png`, `menu-flow-workbench.png`, `font-atlas-workbench.png`, `icon-library-workbench.png`, `ui-binding-workbench.png`, `accessibility-audit-workbench.png`, `input-prompts-workbench.png`, `ui-motion-workbench.png`
- Rendering/GPU pages: `shader-permutations-workbench.png`, `render-target-workbench.png`, `gpu-profiler-workbench.png`, `light-probes-workbench.png`, `reflection-capture-workbench.png`, `decal-editor-workbench.png`, `virtual-texture-workbench.png`, `material-audit-workbench.png`
- Audio/voice pages: `sound-cue-workbench.png`, `audio-mixer-workbench.png`, `music-system-workbench.png`, `audio-occlusion-workbench.png`, `voice-bank-workbench.png`, `subtitle-timing-workbench.png`, `lip-sync-workbench.png`, `audio-profiler-workbench.png`
- Physics/simulation pages: `rigid-body-workbench.png`, `physics-constraints-workbench.png`, `destruction-workbench.png`, `cloth-simulation-workbench.png`, `vehicle-physics-workbench.png`, `fluid-simulation-workbench.png`, `rope-cable-workbench.png`, `physics-profiler-workbench.png`
- AI/navigation pages: `ai-director-workbench.png`, `blackboard-workbench.png`, `eqs-query-workbench.png`, `crowd-simulation-workbench.png`, `smart-objects-workbench.png`, `patrol-routes-workbench.png`, `cover-system-workbench.png`, `ai-profiler-workbench.png`
- Asset pipeline/DCC pages: `mesh-import-workbench.png`, `lod-chain-workbench.png`, `redirect-map-workbench.png`, `texture-compression-queue-workbench.png`, `source-asset-trace-workbench.png`, `dcc-live-link-workbench.png`, `metadata-editor-workbench.png`, `batch-process-queue-workbench.png`
- Engineering/production pages: `script-editor-workbench.png`, `api-browser-workbench.png`, `plugin-packaging-workbench.png`, `module-settings-workbench.png`, `automation-suite-workbench.png`, `build-config-workbench.png`, `cook-rules-workbench.png`, `runtime-commands-workbench.png`
- Project governance pages: `asset-migration-workbench.png`, `scene-diff-workbench.png`, `prefab-diff-workbench.png`, `performance-budget-workbench.png`, `memory-budget-workbench.png`, `dependency-cleanup-workbench.png`, `naming-rules-workbench.png`, `release-checklist-workbench.png`
- Runtime QA pages: `gameplay-debugger-workbench.png`, `replay-timeline-workbench.png`, `network-packet-inspector-workbench.png`, `latency-map-workbench.png`, `input-trace-workbench.png`, `save-state-diff-workbench.png`, `repro-recorder-workbench.png`, `qa-triage-workbench.png`
- Graphics deep-dive pages: `render-graph-workbench.png`, `shader-debugger-workbench.png`, `texture-streaming-workbench.png`, `shadow-map-workbench.png`, `occlusion-culling-workbench.png`, `frame-compare-workbench.png`, `material-layers-workbench.png`, `gpu-memory-workbench.png`
- Animation production pages: `retarget-workbench.png`, `ik-solver-workbench.png`, `pose-library-workbench.png`, `mocap-cleanup-workbench.png`, `animation-compression-workbench.png`, `root-motion-workbench.png`, `event-tracks-workbench.png`, `montage-debugger-workbench.png`
- UI diagnostics pages: `widget-tree-debugger-workbench.png`, `layout-constraint-solver-workbench.png`, `theme-variant-preview-workbench.png`, `localization-preview-workbench.png`, `focus-navigation-workbench.png`, `input-glyph-mapper-workbench.png`, `ui-snapshot-diff-workbench.png`, `widget-performance-workbench.png`
- World streaming pages: `world-partition-workbench.png`, `hlod-builder-workbench.png`, `level-instance-workbench.png`, `streaming-profiler-workbench.png`, `scene-bookmarks-workbench.png`, `spawn-point-editor-workbench.png`, `collision-matrix-workbench.png`, `environment-probes-workbench.png`
- LiveOps pages: `feature-flags-workbench.png`, `remote-config-workbench.png`, `telemetry-query-workbench.png`, `patch-planner-workbench.png`, `dlc-catalog-workbench.png`, `crash-symbolication-workbench.png`, `player-segment-workbench.png`, `experiment-console-workbench.png`
- AI workbench style references: `ai-workbench-style/ai-workbench-web-framework.png`, `ai-workbench-style/ai-material-editor-workbench.png`, `ai-workbench-style/ai-montage-editor-workbench.png`, `ai-workbench-style/ai-asset-browser-workbench.png`
- Layout specs: `main-tabs-layout-spec.png`, `tool-drawers-layout-spec.png`, `scene-drawer-layout-spec.png`, `material-drawer-layout-spec.png`, `montage-drawer-layout-spec.png`, `ui-asset-drawer-layout-spec.png`
- State specs: `drawer-collapsed-state-spec.png`, `drawer-expanded-state-spec.png`, `split-editor-state-spec.png`, `bottom-timeline-console-state-spec.png`, `floating-tool-window-state-spec.png`, `compact-editor-state-spec.png`
- Content specs: `prefab-drawer-content-spec.png`, `files-drawer-content-spec.png`, `hierarchy-drawer-content-spec.png`, `inspector-drawer-content-spec.png`, `animation-list-drawer-content-spec.png`, `console-drawer-content-spec.png`, `timeline-drawer-content-spec.png`, `asset-grid-drawer-content-spec.png`
- Overlay specs: `command-palette-window-spec.png`, `context-menu-window-spec.png`, `tab-overflow-window-spec.png`, `asset-picker-window-spec.png`, `import-wizard-window-spec.png`, `project-settings-window-spec.png`, `confirm-dialog-window-spec.png`, `notification-center-window-spec.png`
- Workflow specs: `prefab-placement-workflow-spec.png`, `asset-import-workflow-spec.png`, `shader-error-workflow-spec.png`, `animation-event-workflow-spec.png`, `runtime-debug-workflow-spec.png`, `build-export-workflow-spec.png`, `ui-binding-workflow-spec.png`, `lighting-bake-workflow-spec.png`
- Focused states: `scene-toolbar-focus.png`, `scene-gizmo-focus.png`, `hierarchy-selection-focus.png`, `hierarchy-context-menu-focus.png`, `inspector-transform-focus.png`, `inspector-material-focus.png`, `asset-browser-grid-focus.png`, `asset-browser-import-focus.png`, `console-log-filter-focus.png`, `console-detail-focus.png`, `project-overview-dashboard-focus.png`, `project-overview-actions-focus.png`, plus the matching tool/window focus PNGs listed in `editor-workbench-designs/STYLE-NOTES.md`

## Documents

- [Shared UI Core Foundation](./shared-ui-core-foundation.md): `zircon_runtime::ui` 新增的共享约束类型、measure/arrange pass、`HorizontalBox`/`VerticalBox`/`ScrollableBox`/`WrapBox` 容器、retained tree、dirty/invalidation、scroll state、命中索引、pointer/focus/navigation route、统一 pointer dispatcher、虚拟化窗口工具，以及 `zircon_editor` workbench autolayout 的复用边界。
  这一轮还补上了显式 `Container` / `Overlay` / `Space` 共享容器名、pointer button payload、capture 后移出命中范围仍保持派发的底层语义、host viewport pointer/scroll -> shared `UiSurface + UiPointerDispatcher` 的宿主接线，以及 editor shell drag target hit-test / dock target route 的 host-owned shared bridge。
- [Shared UI Input Events](./shared-ui-input-events.md): `zircon_runtime_interface::ui::dispatch::input` 的 M5 shared input DTO 契约，覆盖 common metadata、pointer/keyboard/text/IME/navigation/analog/drag-drop/popup/tooltip event families、transient reply/effect commands、input-method requests、dispatch diagnostics、host requests 和 component event reporting。Milestone 2 进一步把 runtime `UiSurface` 接到 shared reply/effect application 与 `dispatch_input_event(...)`，但 editor native translation、M6 caret/selection/shaping 和 M7 debug tooling 仍是后续边界。
- [Bevy UI/Text/Widgets/Focus/A11y M0 Gap Audit](./bevy-ui-text-widgets-focus-a11y-m0-gap-audit.md): 参照本地 `dev/bevy` UI、input focus、text、ui_render、standard widgets、Feathers 和 a11y 源码，建立 Zircon 当前 focus/picking/text/widgets/render/a11y 能力矩阵、缺口表、目标测试表和后续 docs 更新清单；该文档是 Bevy 对齐计划进入 M1 合同 spine 前的证据门禁，不修改 runtime/editor 行为。
- [Slate Style UI Surface Frame](./slate-style-ui-surface-frame.md): `UiVisibility`、`UiArrangedTree`、`UiHitTestGrid`、`UiHitPath` 和 `UiSurfaceFrame` 的 Slate-style 空间模型，记录 render extract 与 hit grid 共享 arranged geometry、SurfaceFrame 诊断快照、Widget Reflector 基线数据、drawcall/overdraw/material batch/hittest 统计、snapshot-derived Debug Reflector overlay，以及 editor native toolbar/template-node 命中收束到共享 hit-grid adapter 的迁移状态。
- [Zircon UI 与 Unreal Slate 布局差异审计](./zircon-ui-unreal-slate-layout-gap-audit.md): 参照 `dev/UnrealEngine` 的 Slate 布局源码，细化 prepass、`FGeometry`、arranged children、parent-owned slot policy、Overlay/Canvas/Linear/Grid/Flow/Scroll/Splitter/Scale panel、DPI、pixel snapping 和 clipping 与当前 Zircon layout pass 的差距及后续布局里程碑。
  该文档现在也记录了 `zircon_runtime_interface::ui::layout` 的 L1 起步契约、template build 将 parent-owned slot record 保留到 `UiTree.slots` 的当前落点、Linear slot size-rule、Overlay slot z-order、Canvas/Free slot placement contract 的 preservation 状态，以及 runtime layout pass 对 Linear/Overlay/Free slot padding/alignment/order 的最小消费边界。
- [Runtime UI Layout Pass Slots](../zircon_runtime/ui/layout/pass.md): `zircon_runtime::ui::layout::pass` 的 slot/panel module detail，记录 `UiSlotSchema`、`UiSlot`、`UiContainerKind`、template slot contract、runtime slot padding/alignment/order consumption、M1.3 overlay/scroll shared-frame focused tests，以及 grid/flow、overlay slot `z_order` 和 canvas placement 的剩余缺口。
- [Material UI Token And Component Audit](./material-ui-token-component-audit.md): M2.1a 的参考 Material 组件与 Zircon `.ui.toml` Material token/component 对照表，覆盖 density、spacing、radius、color roles、state layers、focus ring、shadow/elevation、typography、meta component coverage、参考导出缺口、runtime Material layout support、Material Component Lab `.zui` 原型规则、UI shader contract、MUI X 原型边界，以及 Material Lab startup/hover/click 视觉证据门禁。
- [Slint Material Retained Editor Migration](./slint-material-retained-editor-migration.md): 记录 `dev/material-rust-template/material-1.0` 到 retained Editor UI 的正式迁移边界、Slint Material export 映射、no direct Slint dependency fence、M0/M1 token landing、后续 state layer/ripple/elevation/control/surface/adoption 里程碑和 focused validation 口径。
- [Material UI Component Design Matrix](./material-ui-component-design-matrix.md): 按 MUI All Components 与 MUI X Tree View/Data Grid/Charts/Chat 建立的组件设计矩阵，记录每个组件的响应机制、外观变体、布局模式、`.zui` 映射和视觉 + 交互验证策略。
- [Zircon UI 与 Unreal Slate 渲染差异审计](../assets-and-rendering/runtime-ui-slate-rendering-gap-audit.md): 参照 `dev/UnrealEngine` 的 Slate 渲染源码，细化 paint element、brush/material payload、batch plan、cached render elements、runtime renderer/debug visualizer 与当前 shared UI render extract 的差距及后续渲染里程碑。
- [Shared UI Template Runtime](./shared-ui-template-runtime.md): `zircon_runtime::ui::template` 的 TOML 文档模型、component/slot 节点语义、模板校验规则、运行时实例展开、shared `UiTree` / `UiSurface` 桥接，以及 `.ui.toml -> UiSurface -> host projection / UiRenderExtract` 的显式映射；它记录 no-Slint generic host fence、Rust-owned `host_contract` DTO/callback seam、runtime fixture acceptance，以及 workbench pane body、HostMenuChrome business rows/popup、top menu/Page tab/Dock header/status bar/floating header/activity rail chrome 从手写 `.slint` 迁到 `.ui.toml -> TemplatePane` 的 cutover 状态。
- [Workbench Main Interface Entries](../zircon_editor/ui/layouts/windows/workbench_host_window/main_interface_entries.md): M3.1a 主界面入口收口文档，记录菜单、drawer、toolbar、document pane、floating panel 如何以 `.ui.toml + shared surface` 为真源，并把 host 限制在投影/呈现边界。
- [Runtime UI Component Showcase](./runtime-ui-component-showcase.md): Runtime UI 组件描述注册表、typed value/event/state/drop 契约、`editor.ui_component_showcase` Activity Window、Showcase `.ui.toml` 资产和 retained host generic component-row projection。
- [UI Asset Documents And Editor Protocol](./ui-asset-documents-and-editor-protocol.md): `zircon_runtime::ui::template::asset` 当前 tree-shaped `.ui.toml` authority、flat-to-tree 一次性迁移器、shared loader/compiler/surface builder，以及 editor/runtime 如何继续共用同一条资产消费链路。
- [Editor Workbench Design Export](./editor-workbench-design-export.md): `tools/editor-workbench-preview` 的 deterministic PNG design-export workflow，记录 `design.html`/`design.js`/`design.css`/`preview-sheet.js` 如何以 `workbench.png` 为密度与比例基准生成主标签页、五区工具抽屉、生产工具页面、诊断/资源关系页面、项目基础设施页面、协作/发布页面、动画叙事页面、世界构建/环境美术页面、Gameplay/运行时系统页面、平台/在线服务页面、UI/UX 工具页面、渲染/GPU 工具页面、音频/语音工具页面、物理/仿真工具页面、AI/导航工具页面、资产管线/DCC 页面、工程/生产页面、项目治理/发布准备页面、Runtime QA 页面、工作流标注、浮动操作窗体和扁平圆角控件风格的 editor 页面/焦点状态设计稿，并用 `design:verify` 做尺寸、PNG 结构、manifest 唯一性、`design.js` 渲染器注册表一致性、`preview-sheet.png` manifest 覆盖、输出目录额外 PNG、`STYLE-NOTES.md` 覆盖和暗色 workbench 视觉轮廓验收；`design:export:only -- <id>` 可定向刷新新增页面，`export-options.mjs` 统一处理逗号/空格批量 id 和 `--no-sheet` 预览总览图开关。
- [UI Module Boundary Refactor](./ui-module-boundary-refactor.md): `event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 从混合单文件重构成 folder-backed subtree 后的职责边界，并记录 `binding/model` DTO owner 已硬切到 `zircon_runtime_interface::ui::binding`。
- [Editor Host Final Cleanup](./editor-host-final-cleanup.md): `Final cleanup` 阶段对 editor host 剩余 legacy seam 的删除记录，覆盖 drawer extent root binding、menu button frame host setter/binding、floating-window drag/document-tab 生产路径里的 geometry outer-frame fallback，以及 root-shell projection / callback sizing helper 对 legacy geometry 的最后兜底。

## Related Files

- `zircon_runtime/src/ui/mod.rs`
- `zircon_runtime/src/ui/layout/constraints.rs`
- `zircon_runtime_interface/src/ui/layout/geometry.rs`
- `zircon_runtime/src/ui/layout/pass/mod.rs`
- `zircon_runtime/src/ui/layout/scroll.rs`
- `zircon_runtime/src/ui/layout/virtualization.rs`
- `zircon_runtime/src/ui/template/asset/mod.rs`
- `zircon_runtime/src/ui/dispatch/mod.rs`
- `zircon_runtime/src/ui/tree/node/mod.rs`
- `zircon_runtime/src/ui/tree/hit_test.rs`
- `zircon_runtime/src/ui/surface/mod.rs`
- `zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs`
- `zircon_editor/src/ui/workbench/autolayout/mod.rs`
- `zircon_editor/assets/ui/editor/host/workbench_shell.v2.ui.toml`
- `zircon_editor/assets/ui/editor/windows/workbench_window.v2.ui.toml`
- `zircon_editor/assets/ui/editor/reference/workbench.png`
- `zircon_editor/src/ui/retained_host/ui/root_template_overlay.rs`
- `zircon_editor/src/ui/retained_host/ui/reference_overlay_apply_tests.rs`
- `zircon_editor/src/ui/retained_host/host_contract/data/host_root.rs`
- `zircon_editor/src/ui/retained_host/host_contract/painter/workbench.rs`
- `zircon_editor/src/ui/retained_host/host_contract/presenter/command_stream/tests.rs`
- `docs/ui-and-layout/workbench.png`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs`
- `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs`
- `zircon_editor/src/ui/retained_host/host_contract/mod.rs`
- `zircon_editor/src/ui/retained_host/ui/pane_data_conversion/mod.rs`

## Current Scope

当前文档只覆盖这次共享 UI core 计划已经落地的第一段：

- 共享约束和几何类型进入 `zircon_runtime::ui`
- retained tree、dirty 标记、clip 链命中、scroll state 与 surface/render extract scaffolding 进入 `zircon_runtime::ui`
- 基础 measure/arrange pass、`Container` / `Overlay` / `Space` / `HorizontalBox` / `VerticalBox` / `ScrollableBox` / `WrapBox` 共享容器、pointer/focus/navigation route、统一 pointer dispatcher 和虚拟化窗口计算进入 `zircon_runtime::ui`
- `event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 已经按 folder-backed subtree 重新分层；`binding/model` neutral DTO declarations 已硬切到 `zircon_runtime_interface::ui::binding`，runtime `binding/mod.rs` 只保留 router behavior 与 interface DTO re-export
- `template/asset` 已经承接正式 `layout/widget/style` 资产 AST、tree-shaped `.ui.toml` authority、一次性 flat-to-tree migration adapter，以及编译到 `UiTemplateInstance` / `UiSurface` 的 shared 真源链路
- editor workbench 首批 pane body 已把 `.ui.toml -> PanePresentation` 作为结构 authority，Rust-owned host/native 需要的 DTO 被限制在 `PaneNativeBodyData` 宿主投影内
- `editor.host.workbench_shell` 和 `editor.window.workbench` 在 2026-05-29 改为以 `WorkbenchShellReferenceImage` / `WorkbenchReferenceImage` 呈现 `ui/editor/reference/workbench.png`，该资源是 `docs/ui-and-layout/workbench.png` 的 byte-identical editor 资源副本，并由测试锁定 `1672 x 941` PNG 尺寸；`UiHostWindow::new` 的默认 retained host 尺寸也同步为 `1672x941`，使首帧窗口尺寸与基准图画布一致。外层 host shell 仍保留 `UiHostWindowRoot`、menu、activity rail、document host 和 status bar 节点作为后续真实组件替换层，但当前首屏视觉由用户提供的整窗基准图覆盖。`WorkbenchShellReferenceImage` 在 shared surface 中保持 `input_policy = Ignore`，并由 route/export 测试锁定为 full-window `Image` render command、`UiVisualAssetRef::Image` 与 image brush；retained host projection 测试同时确认同一路径会加载成 `1672 x 941` preview image。`root_template_overlay` 将 `BuiltinHostWindowTemplateBridge` 的 root projection 转成 `HostWindowPresentationData.root_template_nodes`，native painter 最后绘制该 root 模板图层；`apply_presentation_projects_workbench_reference_overlay_from_host_template_bridge` 锁定真实 `apply_presentation` 接线，防止 root projection 参数或 presentation 字段以后被漏传；`apply_presentation_snapshot_matches_workbench_reference_from_host_template_bridge` 在同一生产接线后截取 `UiHostWindow` native snapshot 并与 `docs/ui-and-layout/workbench.png` 做像素级比较，防止以后只保留数据接线但遗漏 painter 输出；`native_host_window_snapshot_draws_workbench_reference_overlay_pixels` 使用真实 `UiHostWindow` 快照与 `docs/ui-and-layout/workbench.png` 做像素级比较，防止以后绕过 painter 或错误缩放参考图。`full_command_stream_replays_workbench_reference_overlay_pixels` 和 `patch_command_stream_repaints_workbench_reference_overlay_damage_pixels` 同步锁定 presenter command stream 的完整重放与 damage patch 重放路径，使命令流入口也必须输出同一参考图像素。
- `root_template_overlay` 对 `WorkbenchShellReferenceImage` 只传递 host node identity、role/frame、media source 和 preview image，保留空 `text` / `value_text` / `options_text` / `icon_name` 与默认交互状态，并丢弃源模板 clip；这样 TOML 里的图片路径 `value = "ui/editor/reference/workbench.png"` 不会被 retained painter 当作标签文字覆盖到基准图上，整窗参考图也不会因源模板裁剪走偏离 native overlay 的绘制路径。
- active `zircon_editor/ui` tree 已硬性保持无 `.slint` 源，former generated Slint DTO/callback seam 由 `zircon_editor/src/ui/retained_host/host_contract/**` 的 Rust-owned structs/globals 承接；`generic_host_layout_paths` 和 `generic_host_boundary` 同时禁止恢复 `ui/workbench.slint`、`slint_build`/`slint-build`、`slint::include_modules!()`、`temp/slint-migration/**` 或 `as slint_ui` alias
- runtime UI fixture acceptance 已证明 `HudOverlay`、`PauseMenu`、`SettingsDialog` 和 `InventoryList` 全部能从 `.ui.toml` 经 `RuntimeUiManager -> UiSurface -> UiRenderExtract -> WgpuRenderFramework::submit_runtime_frame(...)` 进入 screen-space UI pass
- editor root viewport 在 Drawer 可见时优先使用 drawer 派生出的 document frame 作为 Scene 内容区域，并在换算 Scene canvas 时扣除 document header、1px separator 和 viewport toolbar，避免 stale `PaneSurfaceRoot` 尺寸把 Scene render 拉回整窗宽高或把渲染帧算高后再纵向压缩；host scene image 使用 fill 承接 renderer 输出尺寸，不再二次 contain letterbox。该项以窗口截图 `target/visual-layout/editor-window-20260427-layout-continue-header-fix.png` 验收，截图中 Scene 标签和状态栏均显示 `1078 x 655`，可见画面底边对齐 Console 上沿。
- Scene viewport toolbar 现在由 host pane surface 在 viewport canvas 之后绘制并显式占用 `viewport_toolbar_height`，避免只预留高度但控件被下层画面或空背景吞掉；场景内旧 runtime HUD 不再输出 `Move | Persp | Shaded` 状态条，防止和正式 toolbar 重复并遮挡渲染画面。该项以窗口截图 `target/visual-layout/editor-window-20260428-toolbar-no-viewport-hud.png` 验收，截图中 Scene 工具栏位于 Scene 标签下方，视口内部不再显示重复 HUD，Scene 标签和状态栏均显示 `1078 x 655`。
- Project Overview 的 Catalog 操作区在窄 Drawer 中改为卡片内左锚点排列，并加高 Catalog panel，避免 `Open Assets` 被右锚定宽度计算推出卡片边界；Scene viewport toolbar 在小于 `900px` 时进入 compact 模式，保留主要工具、显示模式和投影切换，隐藏轴向对齐与 snap 数值按钮。该项以 `target/visual-layout/editor-window-20260428-project-buttons-runtime-restart-crop.png`、`target/visual-layout/editor-window-20260428-scene-toolbar-compact.png` 和 `target/visual-layout/editor-window-20260428-layout-continue-final-large.png` 验收，覆盖 Project Drawer、1280x760 小窗口 Scene 工具栏和 1616x1019 大窗口回归。
- Bottom drawer 默认 extent 和 Console/Runtime Diagnostics/Module Plugins 的默认约束现在收敛到 compact 底部策略；当可用高度处于小窗口区间时，自动布局与 host shared-frame root 投影都会把底部 drawer 限制到 `120px..148px`，把高度优先还给 Scene/Document 区。该项以 `target/visual-layout/editor-window-20260428-bottom-compact-max-small.png` 和 `target/visual-layout/editor-window-20260428-bottom-compact-max-large.png` 验收：1280x760 下 Scene 从 `742 x 396` 提升到 `742 x 448`，1616x1019 下仍保持 `1078 x 655`。
- Side drawer compact 现在在 1100px 以下启用非对称宽度策略，左侧保留 Project/Assets/Hierarchy tab 可读宽度，右侧 Inspector 更积极压缩，把额外横向空间交给 Scene；Project Overview Details/Catalog 垂直堆叠在小窗口下会压缩间距与 panel 高度，Catalog 按钮保持左锚定纵向排列，避免窄 Drawer 中 `Asset Browser` 被横向或纵向裁切。该项以 `target/visual-layout/editor-window-20260428-side-compact-asymmetric-1024x700.png` 验收：1024x700 下 Scene 从 `486 x 399` 提升到 `526 x 399`，左侧 tab 和 Project 按钮均未裁切；`target/visual-layout/editor-window-20260428-project-vertical-compact-800x620.png` 覆盖 800x620 极窄窗口，Project 的 `Open Assets` 与 `Asset Browser` 均可见，Scene 保持 `302 x 336`；`target/visual-layout/editor-window-20260428-side-compact-240-1616x1019.png` 和 `target/visual-layout/editor-window-20260428-project-vertical-compact-1024x700.png` 覆盖大窗口/中窗口回归。
- Ultra-compact layout 在 760px 以下进一步压缩左右 Drawer，在 420px 以下的 center+bottom 可用高度中把 bottom drawer 限制到 `80px..96px`，优先保留 Scene 和 Project 操作入口；Project Overview 的 Catalog 按钮现在在矮窗口下先于摘要显示。该项以 `target/visual-layout/editor-window-20260428-catalog-actions-first-700x620.png`、`target/visual-layout/editor-window-20260428-catalog-actions-first-900x500.png`、`target/visual-layout/editor-window-20260428-catalog-actions-first-1280x520.png` 验收：700x620 下 Scene 从 `202 x 336` 提升到 `302 x 336`，900x500 下 Scene 从 `402 x 216` 提升到 `402 x 256`，1280x520 下 Scene 从 `742 x 236` 提升到 `742 x 273`；三个截图中 Project 的 `Open Assets` 与 `Asset Browser` 操作入口均可见，矮窗口中 Catalog 摘要允许被下方裁切。
- Scene 后处理不再把 clustered-lighting tile buffer 作为默认可见颜色/强度叠加到最终 frame，避免编辑器预览区出现块状阶梯伪影；该项以窗口截图 `target/visual-layout/editor-window-20260427-after-cluster-tile-fix.png` 作为人工视觉验收样本。
- editor root generated bootstrap 已退场，当前 `UiHostWindow`、`UiHostContext`、`HostWindowSceneData`、`host_scene_data`、`HostWindowSurface*` scene contract 名称和 `host_drag_pointer_event` / `host_resize_pointer_event` 宿主事件由 Rust-owned `host_contract/**` 暴露，并用源码守卫阻止旧 workbench host bootstrap 名称或 deleted source authority 回流
- `zircon_runtime/src/ui/mod.rs` 进一步收缩为 crate 导航层，`UiConfig`、`UI_MODULE_NAME` 和 `module_descriptor` 已下沉到 `zircon_runtime/src/ui/module.rs`
- shared pointer button payload、capture 持续派发语义，以及第一条 editor host viewport pointer/scroll shared bridge 已经接到 `zircon_runtime::ui`
- Slate-style `UiVisibility` / `UiArrangedTree` / `UiSurfaceFrame` 契约已经进入 `zircon_runtime_interface::ui`；runtime arranged-tree builder 会把 legacy `state_flags.visible=false` 规范化为 effective `Hidden`，render extract、hit grid、focus/navigation 和 scroll candidate 都消费同一份 effective visibility helper
- editor shell drag target route 已经开始通过 host-owned `WorkbenchDragTargetBridge` 复用 shared `UiSurface + UiPointerDispatcher`
- editor workbench layout 数学改为直接复用共享 solver 和共享 frame/size/constraint 类型
- `zircon_editor::ui::asset_editor` 已经承接 `editor.ui_asset` 的窗口协议、session 和 source/preview authoring 入口，为后续更深的 tree-native editor refactor 保留稳定 id、mode 和 route 载荷
- `WorkbenchLayout` 继续只做 editor 拓扑与持久化，不再承担底层基础类型的定义权

后续如果继续落地更完整的容器族、grid/flow 虚拟化、editor/runtime 宿主接线、world-space UI 和 runtime ECS bridge，可以在本目录继续追加更细分的实现文档。
