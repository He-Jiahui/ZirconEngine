---
related_code:
  - zircon_runtime/assets/fonts/default.font.toml
  - zircon_runtime/assets/fonts/FiraMono-subset.ttf
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/importer/ingest/import_from_source.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/asset/project/manager/asset_kind.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/assets/ui/runtime/fixtures/hud_overlay.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/pause_menu.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/settings_dialog.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/inventory_list.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/support.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_host.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager_error.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/surface_builder.rs
  - zircon_runtime/src/ui/v2/surface_tree
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
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
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/node_pool.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/paint.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_runtime_interface/src/ui/surface/render/debug.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/surface/render/typography.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/ui.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_core2d.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/scene_passes/render_scene_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/construct/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager_error.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/surface_builder.rs
  - zircon_runtime/src/ui/v2/surface_tree
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/text.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/painter/primitives.rs
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/text_layout.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/assets/ui/runtime/fixtures/hud_overlay.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/pause_menu.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/settings_dialog.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/inventory_list.v2.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/quest_log_dialog.v2.ui.toml
  - zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs
  - zircon_editor/src/tests/ui/boundary/global_material_surface_assets.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/support.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_host.rs
  - zircon_runtime/tests/font_asset_manifest_contract.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
plan_sources:
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 是指加载入口资源文件
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-21 M1 主链收口与文本底座计划，runtime UI 文本改为 glyphon + wgpu，并保留 SDF/native 共存接口
  - user: 2026-04-21 继续推进 M1，补齐默认字体资产归属与默认可用闭环
  - user: 2026-04-21 继续推进 M1，把 .font.toml 接进正式 asset/resource/importer 主链，并让 UI loader 复用公共 FontAsset
  - user: 2026-04-21 继续推进 M1，让项目内 res:// 字体资产通过 ProjectAssetManager 进入 runtime UI 文本链路
  - user: 2026-04-28 继续文本的 SDF 渲染和排版能力任务
  - user: 2026-05-05 SVG/Image components, SVG icons, Material UI, and top-right debug refresh-rate overlay must stay on the .ui.toml chain
  - user: 2026-05-12 runtime UI v2 化、全局 dirty-domain 增量刷新、组件交互完整度、旧 schema fallback 更大范围删除
  - user: 2026-05-18 M7A runtime UI placement in product render pipeline
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - .codex/plans/Zircon UI 增量布局、增量重绘与控件池优化计划.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
  - .codex/plans/Zircon UI 与 Unreal Slate 差异审计及后续里程碑.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Material UI + .ui.toml 全链路 UI 系统推进计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/font.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/graphics/tests/render_product_ui.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
  - cargo test -p zircon_runtime render_extract_carries_visual_contract_fields_for_visible_nodes
  - cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_empty --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::text_layout --locked --jobs 1
  - cargo test -p zircon_runtime screen_space_ui_plan_keeps_text_batches_for_quad_commands
  - cargo test -p zircon_runtime screen_space_ui_plan_routes_sdf_text_to_a_separate_batch
  - cargo test -p zircon_runtime screen_space_ui_plan_keeps_auto_text_in_a_separate_batch
  - cargo test -p zircon_runtime --lib screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches --locked --jobs 1
  - cargo test -p zircon_runtime --lib sdf_atlas --locked --jobs 1
  - cargo test -p zircon_runtime --lib sdf_font_bake --locked --jobs 1
  - cargo test -p zircon_runtime --lib sdf_draw_plan --locked --jobs 1
  - cargo test -p zircon_runtime --lib text_backend_routing --locked --jobs 1
  - cargo test -p zircon_runtime auto_text_mode_uses_font_asset_default_when_present
  - cargo test -p zircon_runtime font_asset_ --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract project_font_manifest_resolves_through_project_asset_manager --locked
  - cargo test -p zircon_runtime render_framework_tracks_text_payloads_submitted_with_shared_ui_extracts --locked
  - cargo test -p zircon_runtime runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces --locked
  - cargo test -p zircon_editor --lib runtime_ui_golden --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib runtime_ui_manager --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib ui_boundary::assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib render_framework_ --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first
  - cargo check -p zircon_runtime --lib
  - cargo check -p zircon_editor --lib
  - cargo test -p zircon_runtime --lib runtime_ui_manager_builds_all_builtin_fixtures_into_shared_surfaces --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_ui_asset_root_contains_only_v2_ui_toml_entries --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib importer_registry_routes_v2_ui_toml_to_v2_document_backend --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib ui_v2_asset_wrappers_parse_and_validate_kind --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib ui_v2_asset_direct_references_include_imports_and_resources --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib importer_decodes_ui_v2_view_component_and_style_assets_from_v2_ui_toml --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib project_manager_scans_ui_v2_assets_and_restores_v2_payloads --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib project_asset_manager_load_accepts_v2_ui_payload_under_ui_layout_kind --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib asset::tests::assets::ui --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib runtime_ui_manager_loads_fixture_documents_from_asset_files --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --jobs 1 -- --nocapture --test-threads=1
  - cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-b
  - cargo test -p zircon_runtime --lib ui_v2 --locked --target-dir target\codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --lib runtime_ui --locked --target-dir target\codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --locked --target-dir target\codex-shared-b -- --nocapture
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir target\codex-shared-b
  - cargo test -p zircon_editor --lib all_runtime_v2_fixtures_share_template_semantic_golden --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib quest_log_runtime_v2_asset_preserves_runtime_semantic_golden --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib runtime_ui_golden_is_hard_cut_to_v2_fixtures --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib ui_asset_editor_runtime --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib runtime_fixture_host_tests_are_hard_cut_to_v2_paths --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib runtime_v2_fixture_buttons_project_interactive_metadata --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --lib runtime_v2_fixture_assets_parse_from_runtime_crate_assets --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics
  - cargo test -p zircon_runtime production_ui_entry_assets_live_under_crate_assets_not_src --locked
  - cargo test -p zircon_runtime default_runtime_font_manifest_stays_inside_runtime_assets --locked
  - cargo test -p zircon_runtime --test font_asset_manifest_contract --locked
  - cargo check -p zircon_runtime --locked --lib
  - cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib rust_owned_host_painter_resolves_runtime_svg_image_assets --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\svg-adaptive-check --message-format short --color never
  - cargo test -p zircon_editor --lib svg_icon_pixels_follow_requested_target_size --locked --jobs 1 --target-dir E:\zircon-build\targets\svg-adaptive-check --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --color never
  - cargo test -p zircon_runtime --lib screen_space_ui_plan --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib text_attrs --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_editor --lib native_runtime_text_painter --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m6 --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib surface_node_pool --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-incremental-layout-render --message-format short --color never
  - $env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - $env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo check -p zircon_editor --lib --locked --jobs 1 --color never
  - $env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --lib builtin_registry_covers_runtime_ui_executor_id --locked --jobs 1 --message-format short --color never
  - $env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never
  - $env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Runtime UI Graphics Integration

## Purpose

这份文档记录 runtime UI 在本轮 cutover 后的正式加载边界：

- 运行时 builtin fixture 的入口资源已经迁出 `src/`
- runtime 只从 crate `assets/` 读取生产 `.v2.ui.toml` 入口
- runtime builtin fixture 现在走 UI v2 flat arena asset 和 heap-resident prototype file cache

本篇强调的是“运行时入口资源位置”和“加载路径”。旧 tree `.ui.toml` 与新 v2 `.v2.ui.toml` 的本体协议见 [`UI Asset Documents And Editor Protocol`](../ui-and-layout/ui-asset-documents-and-editor-protocol.md)。

## Production Entry Assets Must Live Under `assets/`

这轮新增了一条明确约束：正式加载入口资源文件不得放在任何 crate 的 `src/` 目录下。

对 runtime UI 来说，直接变化是：

- 旧位置：`zircon_runtime/src/ui/runtime_ui/fixtures/*.ui.toml`
- 新位置：`zircon_runtime/assets/ui/runtime/fixtures/*.v2.ui.toml`

目前 builtin fixture 包括：

- `hud_overlay.v2.ui.toml`
- `pause_menu.v2.ui.toml`
- `settings_dialog.v2.ui.toml`
- `inventory_list.v2.ui.toml`
- `quest_log_dialog.v2.ui.toml`

editor 侧曾经保留的 `zircon_editor/assets/ui/runtime/*.ui.toml` 旧 runtime preview 资产已经删除。runtime UI 的真源统一收口到 `zircon_runtime/assets/ui/runtime/fixtures/*.v2.ui.toml`，editor 预览和 host metadata 测试也从这些 v2 fixture 构建。

## Runtime Fixture Contract

[`RuntimeUiFixture`](../../zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs) 现在只保留和资源定位直接相关的三个接口：

- `asset_id()`
- `relative_asset_path()`
- `asset_path()`

旧的 `source()` 入口已经删除。runtime fixture 不再通过 `include_str!` 或 `src/fixtures` 的源码字符串进入系统。

这意味着 fixture 枚举现在只负责：

- 把逻辑枚举值映射到稳定 asset id
- 把逻辑枚举值映射到 crate `assets/` 下的相对路径

真正的内容读取、解析和编译都回到 shared UI 资产链路。

## Runtime V2 Semantic Golden

M4.3 的同源验收已经收束为 runtime v2 资产自身的语义 golden，不再把 editor legacy runtime preview 资产当作主链配对输入。验收的共同真源是 `UiV2PrototypeStoreFileCache`、`UiV2SurfaceBuilder`、`UiSurface.compute_layout`、`UiRenderExtract`、semantic control id、text payload 与 binding route，而不是旧递归 schema。

[`runtime_ui_golden.rs`](../../zircon_editor/src/tests/ui/boundary/runtime_ui_golden.rs) 覆盖五个 runtime v2 fixture：

- `hud_overlay.v2.ui.toml`
- `pause_menu.v2.ui.toml`
- `settings_dialog.v2.ui.toml`
- `inventory_list.v2.ui.toml`
- `quest_log_dialog.v2.ui.toml`

每组 golden 都检查 semantic control ids、可见文字 payload、按钮数量、runtime quad/text render payload。Quest Log 还检查 v2 fixture 保留 `QuestLog/Track`、`QuestLog/Close` click binding id 与 `RuntimeAction.*` route。这样可以防止 runtime fixture 只编译出空树，也避免测试继续把旧 `.ui.toml` 资产当成 runtime UI 的长期 fallback。

这个 gate 之前暴露过旧 runtime preview 资产的真实资源缺陷：旧 `.ui.toml` 本地 stylesheet 曾用 `text = "$material_text"` 表示文字颜色，覆盖 authored `props.text`。当前 v2 fixture 不再依赖这些 editor-owned 旧文件；颜色和内容仍通过 v2 props/style 分离表达。

## Shared Load Path

[`RuntimeUiManager`](../../zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs) 现在的正式加载路径是：

1. `UiV2PrototypeStoreFileCache::load_store(std::iter::once(fixture.asset_path()))`
2. `UiV2SurfaceBuilder::build_surface_from_compiled_document(...)`
3. `ui::v2::surface_tree` inserts the compiled arena directly into `UiTree`
4. `surface.compute_layout(...)`

这里没有 runtime-only parser，也没有为 fixture 保留一条旧 recursive schema fallback。`RuntimeUiManager` 持有 `UiV2PrototypeStoreFileCache`，因此同一 fixture 重复加载会复用堆上的 document/prototype/compiled store，而不是每次重新反序列化旧 UI 树。`UiV2SurfaceBuilder` 不再把 v2 arena 转回 `UiTemplateNode`，也不调用 `UiTemplateTreeBuilder` 或 `UiTemplateSurfaceBuilder`；`ui_v2_surface_projection_does_not_call_template_tree_builder` 把这个硬切边界固定成源码守卫。

runtime fixture 和 editor bootstrap 资产现在在最终 `UiSurface`/render extract 边界保持共享。runtime builtin fixtures 从 v2 asset/cache 构建并把 `UiSurface.render_extract` 放进 runtime frame；UI Asset Editor 的 runtime preview 测试也调用 `UiAssetEditorSession::from_v2_source(...)`，再由 `UiAssetPreviewHost::new_v2(...)` 直接构建 v2 shared surface。该路径只把 v2 文档临时投影成轻量 legacy outline，供现有编辑器层级/标签面板显示，不再把旧 `.ui.toml` 当作 runtime fallback。

同一 shared load path 也负责 component schema default 与 authored visual props 的合流。2026-04-29 的修正把 render extract 文本解析锁成非空 `text` 优先、非空 `label` 兜底；空字符串 schema default 不再遮蔽 authored label。因此 runtime fixture、editor asset browser 和 viewport HUD 这类共用 `UiRenderExtract` 的入口，都不需要在宿主侧重新解释 button label。

## Tree TOML Is Also The Runtime Fixture Authority

运行时 fixture 已经全部迁成 UI v2 flat arena `.v2.ui.toml`。因此 runtime UI 现在同时满足两条规则：

- 资源位置规则：入口文件在 crate `assets/`
- 资产格式规则：入口文件是 v2 `root + nodes + component/classes/props/state/layout/slots/events/children` graph，而不是旧递归 `UiTemplateNode`

旧 recursive/flat asset 迁移逻辑只存在于 shared UI 的 test support 和 editor legacy preview support；它不属于 runtime fixture 的正式读取，也不是 `RuntimeUiManager` 加载失败时的 fallback。

Asset importer matching now has the same boundary: `.v2.ui.toml` is registered as its own v2 UI document suffix before the legacy `.ui.toml` suffix. The default runtime importer parses that suffix through `UiV2AssetLoader` and emits first-class `UiV2ViewAsset`, `UiV2ComponentAsset`, or `UiV2StyleAsset` payloads; the first-wave fixture importer selects the same suffix at higher priority for plugin-boundary tests. This prevents v2 files from being parsed through the legacy recursive `UiAssetDocument` loader and lets project scan/artifact restore preserve the v2 payload variant.

## Runtime Frame Boundary

资源目录 cutover 并没有改变 runtime UI 向 graphics 提交的公共语义。

当前 runtime 侧仍然是：

- `RuntimeUiManager` 持有当前 `UiSurface`
- `dispatch_pointer_event(...)` 与 `dispatch_navigation_event(...)` 只是把输入事件转交给 `UiSurface` 的 shared dispatcher 路径，不在 manager 内重建第二套 routing/focus 规则
- `build_frame()` 把 `surface.render_extract` 塞进 `PublicRuntimeFrame.ui`
- render framework / scene renderer 继续消费这份 shared draw extract

R1-R7 render contract work adds a derived paint/batch/cache/text-shape/debug-visualizer layer on top of this same extract instead of adding a second frame boundary. `UiRenderCommand` can now derive `UiPaintElement` records with typed brush, text, resource, clip, and effect payloads, `UiBatchPlan` can explain stable merge/split decisions from those paint elements, `UiRenderCachePlan` can report paint/batch cache reuse or rebuild reasons, `UiShapedText` can carry glyph ids, advances, font/atlas resources, atlas UVs, ellipsis ranges, and edit decorations, and `UiRenderVisualizerSnapshot` can export paint rows, batch rows, overlays, overdraw regions, resource bindings, and text/backend stats for Widget Reflector style panels. Resource-bearing brush payloads now preserve revision, atlas page, UV rect, pixel size, fallback resource, and material variant state in `UiRenderResourceKey` / `UiRenderResourceState`, so future runtime atlas/cache/debug work can split, invalidate, and visualize by shared DTO fields instead of renderer-local guesses. Runtime surfaces now also retain a first per-node `UiSurfaceRenderCache` over `UiRenderCommand` records. Dirty rebuilds reuse unchanged commands, rebuild changed commands, remove missing-node commands, and report damage as old/new frame union rectangles through `UiSurfaceRebuildReport`. `UiPaintElement.cache_generation` is populated from a stable command hash, so downstream debug/cache DTOs can observe reuse without the renderer guessing at local state. Runtime surfaces also retain a first `UiSurfaceNodePool` for template-backed controls: removed controls detach from the tree into a surface-owned pool, compatible future insertions reuse the retained node shell, and ordinary resize/property changes continue through dirty rebuilds without reloading `.ui.toml` descriptions or rebuilding the whole surface. The runtime frame still submits `UiRenderExtract` as the authoritative UI payload.

The 2026-05-08 retained render-cache slice was validated at the runtime surface layer with `cargo test -p zircon_runtime --lib surface_dirty_domains --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-incremental-layout-render" --message-format short --color never`, which passed all 5 focused dirty-domain tests. The render-specific regression proves an unchanged render-dirty command is reused and reports zero damage rectangles. `cargo check -p zircon_runtime_interface --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-ui-incremental-layout-render" --message-format short --color never` also passed for the shared render/cache DTO additions. The control-pool follow-up reached clean `rustfmt --edition 2021 --check` for the touched UI files, but its focused `cargo test -p zircon_runtime --lib surface_node_pool` and a fresh `cargo check -p zircon_runtime --lib` were blocked by unrelated asset-importer API drift before the UI tests could run. Broader editor/runtime validation remains unclaimed in this dirty checkout because unrelated editor host, runtime asset facade, native plugin ABI, and current asset-importer compile errors block those commands before the retained render/editor preview path can be exercised.

`runtime-ui-integration-tests` feature 下的 all-fixture 验收现在会遍历 `HudOverlay`、`PauseMenu`、`SettingsDialog`、`InventoryList`、`QuestLogDialog`，逐个通过 `RuntimeUiManager::load_builtin_fixture(...) -> build_frame() -> WgpuRenderFramework::submit_runtime_frame(...)` 提交，并检查 `RenderStats` 中的 UI command 与 quad/text payload 计数。这条测试只证明所有 builtin fixture 都进入同一 screen-space UI pass，不为某个 fixture 增加专用 renderer 分支。

所以这轮变更的重点不是另起一套 runtime UI renderer，而是确保“进入 renderer 的 UI 数据”来自 crate `assets/` 下的正式 v2 `.v2.ui.toml` 文件，同时把文本子层从占位矩形升级到真正的字形绘制。

## Product Graph Placement

M7A keeps the same `UiRenderExtract` frame boundary, but the concrete placement is now graph-owned instead of being only a late renderer side call. `BuiltinRenderFeature::Ui` declares a `runtime-ui` pass in `RenderPassStage::Ui` with executor id `ui.screen-space`. The pass reads the post-process `final-color` graph resource and writes the external `viewport-output` target, so the product graph can prove that screen-space runtime UI runs after postprocess and before overlay/final presentation.

The default product pipelines all include this pass. `RenderPipelineAsset::default_core2d()`, `default_forward_plus()`, and `default_deferred()` compile the UI feature beside their existing postprocess and debug/overlay stages. Forward+ and Deferred also include the base `PostProcess` feature even when optional bloom, color grading, and history are disabled; that keeps a concrete `post-process` pass in the executed order so UI placement evidence does not disappear in lightweight profiles.

Graph execution now passes the renderer-owned `ScreenSpaceUiRenderer` through `RenderPassGpuExecutionContext`. `RenderPassExecutorRegistry` registers `ui.screen-space`, and that executor calls `ScreenSpaceUiRenderer::record(...)` against the final-color target inside the graph execution slice. The old manual late `ScreenSpaceUiRenderer::record(...)` call after overlay is no longer the authority path for product placement.

`RenderStats` keeps the legacy payload counters from `UiSubmissionStats` and adds graph placement evidence: `last_ui_graph_executed_pass_count`, `last_ui_target_size`, and `last_ui_graph_pass_order`. `update_base_stats(...)` derives those fields from the real `RenderGraphExecutionRecord`, and `last_ui_graph_pass_order = "postprocess-ui-overlay"` is set only when the executed pass list contains `post-process`, `runtime-ui`, and `overlay-gizmo` in that order. This keeps clipped command, image payload, and text payload stats tied to the shared extract while making placement observable from the render framework facade.

## Editor Native Visual Asset Rasterization

The shared runtime render contract already carries visual references through `UiVisualAssetRef`. The Rust-owned editor host now consumes that contract instead of treating every runtime image command as a deterministic placeholder. `host_contract/painter/visual_assets.rs` resolves `UiVisualAssetRef::Image` through the same runtime asset path helper with the editor `assets/` root as a development fallback and resolves `UiVisualAssetRef::Icon` through the editor icon and ionicons folders. Bitmap sources decode through the retained host `Image::load_from_path(...)` / `Image::to_rgba8()` primitives, while SVG sources render through `resvg` against the final host paint rectangle.

The 2026-05-06 icon pass hardens both editor preview projection and runtime visual command resolution against the path variants authored by templates and host DTOs. `preview_images.rs` and `visual_assets.rs` now normalize `res://`, `asset://`, `assets/`, rooted paths, short icon names, `ionicons/name.svg`, and extensionless SVG icon names before probing the editor asset tree. This makes `source = "ionicons/options-outline.svg"`, `icon = "options-outline"`, `UiVisualAssetRef::Image("res://icons/ionicons/options-outline.svg")`, and `UiVisualAssetRef::Icon("ionicons/options-outline.svg")` converge on the same loaded SVG pixels instead of falling back to placeholders.

The painter now enters that path through `UiPaintElement` / `UiPaintPayload` derived from each shared render command. This keeps image, brush, border, and text handling aligned with the new shared DTOs while preserving the previous fallback behavior for missing assets and host-only RGBA painting.

`render_commands.rs` keeps the placeholder path only as the missing-asset fallback. When decode succeeds, runtime `UiPaintPayload` image/vector brushes and template-node preview images emit host image-pixel commands and `primitives.rs::draw_rgba_image_clipped(...)` clips and alpha-blends those pixels into the retained native host frame. SVG commands now ask `visual_assets.rs` for pixels at the target frame size before issuing the command, so resizing a toolbar, tab, menu, or runtime vector brush causes a fresh vector rasterization instead of stretching a cached intrinsic bitmap. Icon references are tinted in the painter-local decoded-pixel cache, while ordinary image references preserve source colors. Template-node icons use the same decoded pixel path, but their tint can now reflect Material interaction state: default, active/selected/pressed, and disabled icon colors are resolved before alpha blending. This keeps the `.ui.toml -> UiSurface.render_extract -> UiRenderCommand` path as the renderer authority; the native host does not add a generated Slint UI or a second image schema.

The cache stores successful image pixels for the editor process. SVG cache keys include asset path, tint, and requested raster size, while bitmap cache keys remain intrinsic-size oriented. That preserves SVG's scale-without-quality-loss contract during pointer damage, pane resizing, and viewport-image region redraws. There is no hot-reload invalidation for this cache yet; file edits are picked up on process restart or a future explicit cache-busting path.

## Typography Contract

`zircon_runtime::ui::surface::render::UiResolvedStyle` 现在不再只有背景和边框字段，它已经补齐 runtime 文本底座要用到的最小 typography 合同：

- `font`
- `font_family`
- `font_size`
- `line_height`
- `text_align`
- `wrap`
- `text_render_mode`

这些字段由 `resolve.rs` 直接从模板 metadata 解析，允许 runtime fixture 和 editor-owned runtime-style overlay 走同一套样式入口。现阶段支持的直写键包括：

- `font = "res://fonts/default.font.toml"`
- `font_family = "Fira Mono"`
- `font_size = 18.0`
- `line_height = 24.0`
- `text_align = "center"`
- `wrap = "word"`
- `text_direction = "auto"` / `"ltr"` / `"rtl"` / `"mixed"`
- `text_overflow = "clip"` / `"ellipsis"`
- `rich_text = true`
- `text_render_mode = "auto"` / `"native"` / `"sdf"`

`[font]` table 也能承载相同语义，便于后续把字体资产、family、尺寸与 render mode 收到一处。

## Text Layout Extract Contract

`UiRenderExtract` 现在不再只把文本当成 `kind = Text` 加一段裸字符串交给 renderer。每条带 `text` 的 `UiRenderCommand` 同时携带 `text_layout: Option<UiResolvedTextLayout>`，由 [`layout_engine.rs`](../../zircon_runtime/src/ui/text/layout_engine.rs) 在 surface extract 阶段生成。少量 editor-side 手写 overlay 或测试命令仍可以显式写入 `text_layout: None`，表示它们只提供 text/style/frame 合同，不绕过 renderer 的通用 text batch planning。

当前 layout DTO 固定承载：

- `font_size` / `line_height`
- `text_align`
- `wrap`
- `direction` / `overflow`
- `source_range`、每行 `visual_range`、`measured_width`、`baseline`
- 已分行的 `UiResolvedTextLine { text, frame, runs }`
- mixed direction 行的低保真 visual order string 与 source/visual byte range 映射
- rich text run kind：plain、strong、emphasis、code、link
- editable text state DTO：caret、selection、composition 和 text edit action 合同
- `overflow_clipped`

这条 extract 层布局是 runtime/editor 共享的中性数据，不依赖生成式 UI 宿主，也不把 editor authoring 状态写进 runtime。它现在按 Unreal Slate 的职责拆分靠拢：`zircon_runtime::ui::text` 承担类似 `FTextLayout` 的 range、run、wrap、overflow 和 editable 状态合同，`zircon_runtime_interface::ui::surface::render` 承担跨层 DTO，glyphon/SDF backend 继续承担最终 shaping、font fallback、atlas/cache 和提交。word wrap 在断行边界会移除分隔空格，ellipsis 会保留被截断前已有的 rich run kind 并追加 plain ellipsis run，editable composition update 会把 preedit 文本写入可见 text range 并覆盖 replacement footprint，commit 只完成该 composition 状态而不二次插入。2026-05-06 的 M6 visual-order slice 在 wrapping/ellipsis 后加入 shared helper：它按 strong LTR/RTL 字符把 run 切成视觉片段，Mixed/LTR 行保持 LTR 段顺序并反转 RTL 段字符，显式 RTL 行反转段顺序，同时保留每个 visual run 的原始 source byte range 和 visual byte range。后续 neutral-separator slice 又让没有强方向的标点/空白继承周围同向 strong run；因此 `שלום-עולם` 这类 RTL 短语内部的连字符会随 RTL span 一起进入视觉顺序，而 LTR/RTL 边界空格仍保持在 LTR 侧以维持已有 mixed-line spacing。这对应 Unreal `FSlateTextShaper::ShapeBidirectionalText` 先分 direction run 再 shaping、`FShapedGlyphSequence::EnumerateVisualGlyphsInSourceRange` 保留 source-to-visual 枚举，以及文本后端按 byte offset 计算 selection/caret geometry 的职责边界。当前 helper 仍是低保真 visual-order scaffold，不做 HarfBuzz cluster shaping、glyph mirroring、combining mark 重排或真实 font fallback；这些继续留给 glyphon/cosmic-text、SDF backend 和后续 HarfBuzz/ICU 接入。

screen-space UI batch planner 会优先消费 `text_layout.lines`：每个 resolved line 会变成独立 `ScreenSpaceUiTextBatch`，并保留该行自己的 frame。只有手写 overlay 或测试命令显式给出 `text_layout: None` 时，planner 才回退到旧的整段 `text + command.frame` 批次。这让 extract 层的分行、对齐和裁剪结果真正进入 glyphon/native/SDF backend，而不是在 renderer 内重新退回节点级整块排版。

M6 文本收敛切片进一步把 `UiTextPaint` 的 editable decoration 事实接入 runtime screen-space planner。planner 会从 `UiRenderCommand::to_paint_elements(...)` 读取 shared text payload：selection decoration 进入普通 UI quad draw，作为文本下方的局部高亮；caret 和 composition underline 进入 `post_text_draws`，在 glyphon/SDF text pass 之后重新绑定 UI quad pipeline 画在文本上方。这样 editor native painter 和 runtime WGPU path 都消费同一组 selection/caret/composition underline frame，不再各自用字符数或节点 frame 重新估算。

同一 M6 链路现在也让 rich runs 成为 shared paint fact。`UiTextPaint.runs` 会把 `UiShapedTextCluster` 转成 `UiTextPaintRun`，保留 run text、source/visual range、frame、font/color 继承和 `UiTextRunPaintStyle`。runtime screen-space planner 优先按这些 runs 生成 text batches；glyphon native backend 把 Strong/Emphasis/Code 映射为 bold、italic、monospace attrs；editor native painter 也按同一 run DTO 进行软件 fallback 样式绘制。没有 `text_layout` 的手写 overlay 才继续使用旧整段 text fallback。

新增回归 [`text_layout.rs`](../../zircon_runtime/src/ui/tests/text_layout.rs) 锁住两类行为：

- `render_extract_outputs_aligned_wrapped_text_layout` 证明 word wrap 和 center align 会在 `UiRenderExtract` 中产出稳定行 frame
- `render_extract_clips_text_layout_to_clip_frame` 证明 `clip_frame` 会裁掉不可见文本行并设置 `overflow_clipped`
- `render_extract_outputs_rich_directional_ellipsis_layout` 证明 rich run、direction marker 和 ellipsis policy 同时进入 resolved layout
- `render_extract_outputs_visual_order_ranges_for_mixed_direction_text` 证明 mixed LTR/RTL 文本会输出 visual-order line string，同时保留每段 source/visual range
- `render_extract_keeps_neutral_separator_inside_rtl_visual_span` 证明 RTL 短语内部的 neutral separator 会随 RTL visual span 移动，同时保留原始 source byte range
- `editable_text_state_applies_selection_and_composition_actions` 证明 selection replacement、composition visible update 和 composition commit 走同一 editable text state helper
- `screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches` 证明 graphics planner 会按 resolved line 生成 text batches，而不是吞掉 extract 阶段的排版结果
- `screen_space_ui_plan_uses_shared_text_decorations_as_pre_and_post_text_draws` 证明 runtime WGPU planner 使用 shared text decoration frames，并把 selection 与 caret/composition 分到正确的 text 前/后绘制阶段
- `screen_space_ui_plan_splits_rich_text_runs_from_shared_paint` 证明 graphics planner 按 shared rich paint runs 拆分 batch，并保留 Strong/Code 样式标记
- `text_attrs_maps_shared_rich_run_style_to_glyphon_attrs` 证明 glyphon native path 从 shared run style 得到 bold、italic、monospace attrs

## Glyphon Runtime Text Path

M1 之后，screen-space UI renderer 不再把 `Text` 节点画成一条占位矩形带。当前链路已经变成：

1. `UiSurface` 生成 shared `UiRenderExtract`
2. `ScreenSpaceUiRenderer` 先做 screen-space batch plan
3. 背景 / 边框 / image 继续走现有 quad 路径
4. 文本命令拆进独立 text batch
5. text batch 交给 glyphon + wgpu 路径准备 atlas / glyph buffer 并在同一 UI pass 里提交

这样按钮、标签这类“同一个节点既有背景又有文本”的情况不会再丢文本；quad 和 text 已经是并行层，而不是互斥 kind。

当前实现还额外把 glyphon/cosmic-text 的 API 对齐固定在 renderer 内部边界上：

- [`ScreenSpaceUiTextBatch`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs) 继续只是 screen-space renderer 子树里的 DTO，但字段已经收口成 `pub(super)`，只允许同一 `ui/` renderer 子系统在 batch planner 和 text backend 之间共享
- [`text.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs) 通过 `glyphon::cosmic_text::Align` 和 `Buffer::set_text(..., alignment)` 把 shared `UiTextAlign` 直接传进 glyphon，而不是再靠手动改写内部行布局状态

这能把本轮锁文件升级后暴露的 glyphon API 漂移控制在 renderer 局部，不需要改 shared `UiResolvedStyle`、`UiRenderExtract` 或 runtime fixture 资产格式。

## Native / SDF Coexistence

M1 的完成线不是一次性做完整 SDF 文本系统，而是先把共存合同和运行时批次拆干净。

当前实现里：

- `UiTextRenderMode::Auto` 先进入独立 auto batch，再由 text backend 按字体资产默认值解析到 native 或 sdf
- `UiTextRenderMode::Native` 直接进入 native text backend
- `UiTextRenderMode::Sdf` 直接进入 renderer-local SDF atlas / GPU renderer
- native backend 维护 glyphon text atlas / renderer，SDF backend 维护自己的 SDF atlas texture / bind group / shader pipeline

这让 runtime/editor overlay 现在既能显式声明“这段文本属于 native 还是 sdf”，也能把默认策略下沉到字体资产，而不需要继续把两类策略混在同一条占位路径里。`sdf` backend 现在不再复用 glyphon 可见输出，而是由 [`sdf_atlas.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs) 生成 atlas slot/run plan，再由 [`sdf_render.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs) 上传 SDF atlas texture 并通过 [`sdf_text.wgsl`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl) 绘制 screen-space glyph quads。

兼容普通文本渲染的边界也已经显式化：[`text.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs) 会先把输入批次收进 `ResolvedScreenSpaceUiTextBatches`，再分别交给 native glyphon backend、SDF atlas owner 和 GPU SDF renderer。`sdf_atlas_texts()` 只返回 resolved SDF 批次，因此显式 `Native` 文本和解析为 `Native` 的 `Auto` 文本不会进入 SDF atlas/cache；它们继续走原有 normal glyphon backend。显式 `Sdf` 文本和解析为 `Sdf` 的 `Auto` 文本才会进入 SDF atlas planning 与 GPU SDF draw submission。

## SDF Atlas Boundary

[`ScreenSpaceUiSdfAtlas`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs) 是 `scene_renderer::ui` 内部的 SDF atlas/cache owner。它不向 shared `UiRenderExtract` 暴露 GPU 细节，也不改变 `UiTextRenderMode::Sdf` 的公共样式合同；它只接收已经由 `ScreenSpaceUiTextSystem` 解析后的 SDF `ScreenSpaceUiTextBatch`，生成当前帧的 `SdfAtlasPlan`。

这份 plan 固定做三件事：

- 以 glyph + font asset + font family + quantized font size 作为 `SdfAtlasGlyphKey`，避免不同字体或字号的同一字符错误共用一个 atlas slot
- 普通 stateless planner 仍按 key-sorted glyph set 生成 deterministic atlas slot；runtime owner 则会在非空 SDF 帧之间保留已见 glyph slot，复用旧 slot index，超过 cache 上限时按 last-seen generation 淘汰 inactive slot
- 为每个 atlas slot 分配稳定 `SdfAtlasRect`，小批次从 256x256 texture 起步，超过默认网格后按 power-of-two grid 扩容
- 为每个 SDF text batch 生成 glyph slot index run；空白字符保留 advance 但不分配 atlas slot，让 GPU SDF renderer 可以从同一个 plan 生成 textured glyph quads 而不会画出可见空格

2026-05-23 的 M7 cache 切片把 `ScreenSpaceUiSdfAtlas` 从“每帧非空输入替换整份 plan”推进到持久 slot cache：非空帧会保留上一帧仍有价值的 cached slots，新增 glyph 只追加新 slot，空 SDF 帧会释放 cache，超过 `SDF_ATLAS_MAX_CACHED_SLOT_COUNT` 时才淘汰最久未使用的 inactive slot。`SdfAtlasCacheReport` 同步记录 previous/current slot count、retained/stable/relocated/added/evicted slot count 和 atlas resize；其中 relocated 表示 glyph key 保留但 `SdfAtlasRect` 变了，后续 partial atlas writes 必须把这类槽也当作 dirty slot，而不能只看 key 是否命中。M7 quality-parameter slice 又把 atlas slot size、最小 grid side 和 cache 上限收进 `SdfAtlasQuality`，默认值保持 64px slot、8x8 起步 grid、256 cached slots；planner 内部会 normalize 这些参数，避免 0 值配置破坏 atlas 尺寸或 eviction。

[`ScreenSpaceUiSdfRenderer`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs) 是当前 SDF 可见输出路径。`ScreenSpaceUiTextSystem::prepare` 会先把 `Auto` batch 解析到 native/sdf，再把 resolved SDF batches 同步交给 `ScreenSpaceUiSdfAtlas::prepare(...)`，随后由 SDF renderer 调用 renderer-local [`SdfFontBakeCache`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs) 生成字体轮廓 atlas、上传 `R8Unorm` atlas texture、生成 screen-space glyph quads，并在 UI render pass 中绑定 SDF pipeline 绘制。SDF quad planning 会同时受真实 glyph metrics、advance、bearing、text batch frame、`text_align`、显式 `clip_frame` 和 viewport 约束；native glyphon backend 不再接收 SDF batch，因此替换 shader path 不会污染普通文本 atlas。

真实字体 bake 被局部封装在 `scene_renderer::ui` 内：`SdfFontBakeCache` 通过既有 `.font.toml` manifest 解析字体源，缓存 `fontsdf::Font`，按 `SdfAtlasGlyphKey` 为非空白 glyph bake 单通道 SDF alpha，并把 bitmap 尺寸、bearing、ascent 与 advance 交回 draw planner。whitespace 不写 atlas slot，只通过字体 metrics 保留 advance；missing glyph 使用稳定空可见输出和保守 advance，避免把未知字符退回旧的整块占位 mask。2026-05-23 的 M7 bake-report slice 还让 `SdfAtlasBake` 携带 `SdfAtlasBakeReport`，记录 slot 数、可见/空 glyph 数、atlas byte 数、非零像素数和已加载字体数，后续 quality 参数、局部 atlas upload 和 debug 面板可以直接消费这份报告。这保持了 shared template metadata、`UiRenderExtract` DTO、RHI、render graph 和 render plugin 边界不变。

同日的 M7 text prepare report slice 把 atlas/cache/bake 事实向上汇聚到 runtime text system，而不是让 debug 面板或后续 renderer 统计从底层对象反推。`ScreenSpaceUiSdfRenderer` 在 `prepare(...)` 后保存 `ScreenSpaceUiSdfPrepareReport`，记录 SDF text batch 数、atlas slot 数、atlas size、atlas resize、bake report、当前 atlas upload byte 数、是否全量 texture upload，以及最终 SDF vertex 数；`ScreenSpaceUiTextSystem` 再保存 `ScreenSpaceUiTextPrepareReport`，记录输入 auto/native/sdf batch 数、解析后的 native/sdf batch 数，以及对应的 `SdfAtlasCacheReport` 与 `ScreenSpaceUiSdfPrepareReport`。2026-05-24 的 upload-report slice 进一步把 upload 计算抽到 [`sdf_upload.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs)，并把 `ScreenSpaceUiSdfPrepareReport.atlas_upload` 固定为内部 DTO：当前 GPU path 仍以 `FullTexture` 写入保证正确性，但 report 会同步给出 dirty slot count/dirty byte count；未 resize 且全部 stable retained 时 dirty 为 0，新增或 relocated slot 会计入 dirty，为后续把 full texture upload 收束成局部 texture writes 提供可验证边界。`ScreenSpaceUiRenderer` 会在每次 `record(...)` 后缓存最新 text prepare report，并在没有可提交 UI 时清空这份 report；后续 render stats 或 debug reflector 可以从 screen-space renderer 边界读取这份快照，不需要直接穿透进 glyphon/SDF backend。该层仍然是 `scene_renderer::ui` 内部的 renderer-local 可观测层，不把 GPU atlas 或 glyphon 类型泄漏到 shared `UiRenderExtract`、editor widget 合同或 runtime interface。

这一轮还补了一条 capture 级回归：[runtime_ui_text_render_contract.rs](/E:/Git/ZirconEngine/zircon_runtime/tests/runtime_ui_text_render_contract.rs)。它不再只看 planner/batch 统计，而是直接通过 `RenderFramework::submit_frame_extract_with_ui(...) -> capture_frame(...)` 证明：

- `UiTextRenderMode::Native` 会产出真实 glyph footprint，而不是整块文本占位带
- `UiTextRenderMode::Sdf` 也会沿同一条 runtime UI 提交链产出真实字体轮廓像素；`AIO` 这类测试文本会相对 background-only frame 留下稀疏 glyph delta，而不是退回整块占位带
- `clip_frame` 会继续约束文本采样区域，不会沿整条文本带泄漏
- `wrap = "word"` 会把 glyph footprint 实际分配到多行，而不是仍然挤成单条占位带
- `opacity` 会继续进入 glyph 颜色/采样链路，capture frame 上能看到稳定的可见变暗，而不是只停留在 shared command 元数据里
- 同一个回归文件现在还额外覆盖正式模板资产链；runtime builtin path 已收口到 `.v2.ui.toml -> UiV2PrototypeStoreFileCache -> UiV2SurfaceBuilder -> UiSurface.render_extract -> RenderFramework capture_frame(...)`
- 这意味着 template/surface 驱动的 runtime 文本也已经有最终像素证据，而不再只有手写 `UiRenderCommand` 和 editor HUD 提交路径的 capture 证明

## Font Asset Entry

为了不给 runtime UI 文本继续绑定系统字体或源码常量，这轮新增了最小字体资产入口：

- `zircon_runtime/assets/fonts/default.font.toml`

该 manifest 负责声明默认字体来源与 family。runtime renderer 会把 `res://fonts/*.font.toml` 解析成具体文件路径并在首次使用时加载，未显式指定时则回落到 `res://fonts/default.font.toml`。这条链路已经足够支撑：

- runtime fixture 默认字体可用
- 模板样式显式引用字体资产
- editor-owned runtime overlay 与 runtime UI 共用同一套字体入口

当前默认入口已经进一步收口成 runtime 自有资源：

- `zircon_runtime/assets/fonts/default.font.toml` 现在直接引用同目录下的 `FiraMono-subset.ttf`
- 默认字体不再依赖 `dev/bevy/...` 这类开发树相对路径
- `default_runtime_font_manifest_stays_inside_runtime_assets` 会校验 manifest 解析后的真实源文件仍位于 `zircon_runtime/assets/` 内部

这轮又把 manifest 解析本身收得更硬了一层：

- [`font_asset.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs) 现在负责独立解析 `.font.toml`，不再把路径拼接逻辑散落在 text backend 内部
- manifest 的 `source` 必须是相对路径，绝对路径会被直接拒绝
- 对 `res://fonts/*.font.toml` 来说，`source` 解析后的真实文件必须仍位于 `zircon_runtime/assets/` 根内，不能用 `../` 逃逸到 crate 根或 `dev/` 外部树
- 对外部绝对 manifest 路径来说，`source` 也只能落在 manifest 所在目录作用域内，不能借 manifest 继续跳到旁路目录

M1 这里再补了一条最小默认策略：

- 字体 manifest 可选声明 `render_mode = "native"` 或 `"sdf"`
- `UiTextRenderMode::Auto` 会优先采用字体 manifest 的默认值
- 如果字体 manifest 没写 `render_mode`，则稳定回落到 `Native`
- 如果样式显式写了 `text_render_mode = "native"` / `"sdf"`，显式样式仍然覆盖字体默认值

这条入口现在也不再只是 runtime UI renderer 的私有 TOML 约定，而是已经补进 runtime asset 主链：

- [`FontAsset`](../../zircon_runtime/src/asset/assets/font.rs) 成为正式的最小字体资产语义模型，字段固定为 `source`、`family`、`render_mode`
- `AssetImporter` 已经把 `.font.toml` 接到 [`ImportedAsset::Font`](../../zircon_runtime/src/asset/assets/imported.rs) 和 [`AssetKind::Font`](../../zircon_runtime/src/asset/project/manager/asset_kind.rs)
- [`ArtifactStore`](../../zircon_runtime/src/asset/artifact/store.rs) 会把这类资产写入 `lib://fonts/*.json`，因此 project scan / artifact load / runtime resource registry 已经能稳定识别字体资产
- [`font_asset.rs`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs) 现在直接复用 `FontAsset::from_toml_str(...)`，并把 `render_mode` 以强类型 `UiTextRenderMode` 暴露给 text backend，不再保留一层裸字符串中转
- [`ScreenSpaceUiTextSystem`](../../zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs) 和 renderer 构造链现在会接收 [`ProjectAssetManager`](../../zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_new/new_with_icon_source.rs)，因此 `res://fonts/*.font.toml` 会优先从当前打开项目的正式资产注册表里解析，再回退到 runtime crate 自带默认字体
- [`collect_files.rs`](../../zircon_runtime/src/asset/project/manager/collect_files.rs) 现在会把 `.ttf`、`.otf`、`.woff`、`.woff2` 视为字体 manifest 的 source auxiliary，而不是 standalone project asset；这样项目把原始字体文件放进 `assets/fonts/` 时，不会再在 `scan_and_import()` 阶段直接炸成 unsupported format

这让“允许显式引用字体资产”终于从“只对 runtime crate 自带默认字体成立”推进到了“项目自己的 `res://` 字体资产也能进入同一条 runtime text backend”。

## Editor Viewport HUD Uses The Same Text Backend

这轮 M1 发生在 retained host 硬切换之前；现在 editor viewport 的 authoring overlay 继续接入 runtime 文本底座：

- `EditorState::render_frame_submission()` 现在除了 scene `RenderFrameExtract`，还会带一份可选 shared `UiRenderExtract`
- `SceneViewportController::build_runtime_overlay_ui()` 生成右上角状态 HUD，文本内容来自 editor-owned viewport 状态，而不是 runtime world
- `RenderFramework::submit_frame_extract_with_ui(...)` 把 scene extract 和 HUD 的 `UiRenderExtract` 一起交给 graphics
- `ScreenSpaceUiRenderer` 因此会把这条 HUD 文本和 runtime fixture 文本一样送进 glyphon/native/sdf 批次分流，不需要再为 editor viewport 另起一条文本实现

这就是 M1 里“runtime UI 与 editor viewport/runtime-style overlay 至少共享同一套 runtime 文本底座”的当前落地形态。editor shell 已硬切到 retained host；viewport 内的 runtime-style HUD 仍走 shared text backend。

这条 editor 侧共用路径现在也已经有 capture 级证据，而不再只停留在“提交到了 render framework”：

- [`render_frame_submission_hud_text_renders_through_runtime_glyph_capture`](/E:/Git/ZirconEngine/zircon_editor/src/tests/editing/state.rs) 直接拿 `EditorState::render_frame_submission()` 产出的 scene extract + HUD `UiRenderExtract` 走 `WgpuRenderFramework::submit_frame_extract_with_ui(...) -> capture_frame(...)`
- 测试同时对比了“有字 HUD”和“去字但保留同背景/边框的 HUD”，因此能证明 capture 中新增的像素差异来自 glyph，而不是 HUD 背景 quad 本身
- 这让 runtime fixture 文本和 editor viewport HUD 文本都落在同一条最终 glyph capture 证据链上

## Guard Rails

本轮额外补了源码守卫，避免后续又把生产入口资源偷偷放回 `src/`：

- runtime fixture 必须能从 crate `assets/` 成功枚举和加载
- `zircon_editor/src` 和 `zircon_runtime/src` 下都不允许继续出现生产 `.ui.toml` 入口资源

这条守卫的意义是把目录规范变成测试约束，而不是只靠文档约定。

## Acceptance Evidence

直接覆盖这次 runtime cutover 的验证包括：

- `cargo test -p zircon_runtime --lib runtime_ui --locked --target-dir target\codex-shared-b -- --nocapture`
  - 2026-05-12 runtime-v2 fixture rerun passed; it proves all five builtin runtime fixtures load from crate `assets/`, build shared `UiSurface` values, and keep frame UI payloads present on the v2 path.
- `cargo test -p zircon_runtime --lib ui_v2 --locked --target-dir target\codex-shared-b -- --nocapture`
  - 2026-05-12 runtime-v2 direct-surface rerun passed with 12 tests; it covers direct arena-to-`UiTree` projection, deep v2 surface construction, style pseudo states, component slot validation, and v2 file-cache behavior.
- `cargo check -p zircon_runtime --lib --locked --target-dir target\codex-shared-b`
  - 2026-05-12 runtime-v2 direct-surface rerun passed, proving the runtime v2 fixture and surface projection owners type-check together.
- `cargo test -p zircon_runtime --lib production_ui_entry_assets_live_under_crate_assets_not_src --locked --target-dir target\codex-shared-b -- --nocapture`
  - 2026-05-12 runtime-v2 fixture rerun passed with 1 test; it proves production UI entry assets stay under crate `assets/` and do not return to `src/`.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir target\codex-shared-b`
  - 2026-05-12 runtime package validator passed `cargo build -p zircon_runtime --locked --target-dir target\codex-shared-b` and `cargo test -p zircon_runtime --locked --target-dir target\codex-shared-b` after confirming `62.9 GB` free on the target drive.
- `cargo test -p zircon_runtime render_extract_carries_visual_contract_fields_for_visible_nodes`
  - 证明 template metadata 已经把 typography 字段解析进 shared `UiResolvedStyle`
- `cargo test -p zircon_runtime --lib render_extract_uses_label_when_schema_text_default_is_empty --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final -- --nocapture`
  - 证明 schema 注入的空 `text` default 不会遮蔽 authored `label`，button/asset actions 仍会在 `UiRenderExtract` 中产出可见文本
- `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture`
  - 2026-04-29 fresh broad Runtime UI suite 通过 126 passed / 0 failed；运行前的 lower render test compile blocker 已由当前 node-and-cluster-cull accessor tests 清除，但 Cargo 输出仍保留一个 unrelated `cluster_work_item_buffer` dead-code warning
- `cargo test -p zircon_runtime --lib ui::tests::text_layout --locked --jobs 1`
  - 证明 `UiRenderExtract` 已经输出文本分行、对齐和裁剪后的中性 layout DTO，而不是只保留裸字符串
- `cargo test -p zircon_runtime screen_space_ui_plan_keeps_text_batches_for_quad_commands`
  - 证明带背景的节点仍然会独立产出文本 batch，不再把 text 当成 quad-only 占位
- `cargo test -p zircon_runtime screen_space_ui_plan_routes_sdf_text_to_a_separate_batch`
  - 证明 `UiTextRenderMode::Sdf` 已经进入单独 backend 路由
- `cargo test -p zircon_runtime screen_space_ui_plan_keeps_auto_text_in_a_separate_batch`
  - 证明 `UiTextRenderMode::Auto` 不会在 planner 阶段被硬编码吞成 native，而是保留给 text backend 结合字体资产决策
- `cargo test -p zircon_runtime --lib screen_space_ui_plan_uses_resolved_text_layout_lines_as_batches --locked --jobs 1`
  - 证明 graphics planner 会把 extract 阶段的 resolved text lines 分别送进 text batch，而不是重新用整段文本和节点 frame 排版
- `cargo test -p zircon_runtime --lib sdf_atlas --locked --jobs 1`
  - 证明 SDF atlas/cache owner 会按 glyph + font asset + family + size 生成稳定 slot key，跨 batch 去重，空白只保留 advance 不分配 slot；2026-05-23 的 M7 cache-report/persistent-cache slices 进一步断言非空帧保留旧 slot、返回旧 glyph 时不 re-add、空 SDF 帧清理 cache、超过 cache 上限时只淘汰最久未使用 inactive slot，并记录 retained/stable/relocated/added/evicted slot 数和 atlas resize 标志；同日 quality-parameter slice 断言自定义 slot size/min grid 会改变 atlas size 和 slot rect，而默认 planner 行为保持原值
- `cargo test -p zircon_runtime --lib sdf_font_bake --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-sdf-font-bake --message-format short --color never`
  - 2026-05-01 fresh focused SDF bake suite 通过 4 passed / 0 failed；证明 renderer-local `fontsdf` bake 会为 `A`、`I`、`O` 生成不同 alpha pattern，输出不等于旧 rounded-rect placeholder，whitespace 只保留 advance，不可见/missing glyph 策略稳定不 panic。2026-05-23 的 M7 bake-report slice 扩展同一测试面，断言 bake report 会记录 slot 数、可见/空 glyph 数、atlas byte 数、非零像素数和 loaded font 数，并覆盖 empty atlas plan 的零像素报告。
- `cargo test -p zircon_runtime --lib sdf_draw_plan --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture`
  - 2026-04-29 fresh focused SDF draw-plan suite 通过 5 passed / 0 failed；后续真实 bake slice 保持同一测试面通过，证明 GPU SDF renderer 会从 atlas plan 和真实 glyph metrics 生成每个可见 glyph 的 textured quad，上传 atlas alpha mask，并按 text frame、`text_align`、clip frame 和 viewport 计算 position/uv
- `cargo test -p zircon_runtime --lib sdf_prepare_report --locked --jobs 1`
  - 2026-05-23 M7 text prepare report slice 覆盖 renderer-local SDF prepare report 汇总，断言 text batch、atlas slot、atlas size、resize、bake report、atlas upload byte/full-upload 标志和 vertex count 会进入同一 report DTO；2026-05-24 upload-report slice 又断言当前实际 full texture upload 与 dirty slot/dirty byte 计划统计同时存在，stable retained atlas dirty 为 0，added/relocated slot 会进入 future partial-upload dirty 预算
- `cargo test -p zircon_runtime --lib sdf_upload --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-sdf-upload-report-20260524 --message-format short --color never`
  - 2026-05-24 focused Cargo attempt completed runtime lib-test compilation with warnings only, then the 15 minute command timeout cut test enumeration and produced BrokenPipe; no SDF compile diagnostic or test assertion failure was produced before the timeout
- `cargo test -p zircon_runtime --lib text_backend_routing --locked --jobs 1`
  - 证明 SDF routing contract 不会把普通 native 文本送进 SDF atlas input，`Auto` 解析成 native/sdf 后也不会跨 backend 混用
- `cargo test -p zircon_runtime --lib text_prepare_report --locked --jobs 1`
  - 2026-05-23 M7 text prepare report slice 覆盖 text system 汇总，断言输入 auto/native/sdf batch 数、解析后的 native/sdf batch 数、SDF atlas cache report 和 SDF renderer prepare report 会被合并保存；2026-05-24 同步覆盖新增 `SdfAtlasUploadReport` 字段穿过 text prepare report
- `cargo test -p zircon_runtime auto_text_mode_uses_font_asset_default_when_present`
  - 证明 `UiTextRenderMode::Auto` 会优先采用字体资产 manifest 的默认 render mode，并保留显式样式优先级
- `cargo test -p zircon_runtime render_framework_tracks_text_payloads_submitted_with_shared_ui_extracts --locked`
  - 证明 shared `UiRenderExtract` 通过 render framework 提交时，UI command/quad/text payload 统计都会落进 runtime submission stats，editor viewport HUD 和 runtime fixture 走的是同一条 screen-space UI 提交口
- `$env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`
  - 2026-05-18 runtime lib check passed for the graph-owned UI placement build before the focused M7A tests below.
- `$env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --locked render_product_ui --jobs 1 --message-format short --color never`
  - 2026-05-18 M7A pass: 2 passed / 0 failed. This proves Core2d, Forward+, and Deferred compile `runtime-ui` after postprocess and before overlay, and a submitted runtime UI extract records `ui.screen-space`, target size `320x240`, clipped/image/text payload counters, and `last_ui_graph_pass_order = "postprocess-ui-overlay"`.
- `$env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --lib builtin_registry_covers_runtime_ui_executor_id --locked --jobs 1 --message-format short --color never`
  - 2026-05-18 graph registry guard: 1 passed / 0 failed. This pins the built-in executor registry so the runtime UI graph pass cannot silently fall back to the earlier missing-executor failure mode.
- `$env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo test -p zircon_runtime --locked runtime_ui --jobs 1 --message-format short --color never`
  - 2026-05-18 broader runtime UI pass: 23 runtime lib tests passed, and the `runtime_ui_text_render_contract` integration binary passed 6 matching tests. This revalidates the shared runtime UI boundary plus text/capture contracts against the same graph-aware runtime build.
- `$env:CARGO_TARGET_DIR='target/codex-native-material-painter'; cargo check -p zircon_editor --lib --locked --jobs 1 --color never`
  - 2026-05-18 editor lib pass: shared UI/render contracts still type-check for `zircon_editor` after the graph-owned runtime UI placement changes.
- `cargo test -p zircon_runtime production_ui_entry_assets_live_under_crate_assets_not_src --locked`
  - 证明生产入口 `.ui.toml` 没有回流到 `src/`
- `cargo test -p zircon_runtime default_runtime_font_manifest_stays_inside_runtime_assets --locked`
  - 证明默认字体 manifest 解析后的真实 TTF 仍位于 `zircon_runtime/assets/` 内部，而不是继续穿透到 `dev/` 开发树
- `cargo test -p zircon_runtime --test font_asset_manifest_contract --locked`
  - 证明 `.font.toml` 的 `source` 现在只接受作用域内的相对路径，既拒绝绝对路径，也拒绝从 `res://` 资产根逃逸
- `cargo test -p zircon_runtime --test font_asset_manifest_contract project_font_manifest_resolves_through_project_asset_manager --locked`
  - 证明当前打开项目里的 `res://fonts/project.font.toml` 会优先经 `ProjectAssetManager` 解析，并把 `project.ttf` 当成字体 source auxiliary，而不是把项目 scan 过程炸成 unsupported format
- `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --nocapture`
  - 2026-04-29 fresh capture contract 通过 7 passed / 0 failed；证明 runtime UI 文本在最终 capture frame 上已经是 glyph 输出而不是矩形占位，并同时覆盖 `Native`、centered `Sdf` side margins、clip-bound glyph sampling、多行 wrap、opacity dimming，以及正式 `.ui.toml -> compiled surface -> render extract` 链上的 wrap/opacity glyph capture
- `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-sdf-font-bake --message-format short --color never -- --test-threads=1 --nocapture`
  - 2026-05-01 fresh capture contract 通过 8 passed / 0 failed；新增 SDF/background delta 证明真实 bake 后的 `AIO` glyph footprint 保持稀疏，且不再是旧 placeholder block
- `cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked`
  - 证明 runtime fixture 仍走 shared compiler，而不是 runtime-only 特例解析
- `cargo check -p zircon_editor --lib --locked`
  - 证明 editor viewport 的 runtime-style HUD 已经能编进正式 editor lib
- `cargo test -p zircon_editor --lib controller_submits_shared_ui_overlay_through_render_framework --locked`
  - 证明 viewport scene extract 与 shared HUD `UiRenderExtract` 已经通过正式宿主路径一起提交到 render framework
- `cargo test -p zircon_editor --lib render_frame_submission_carries_editor_owned_viewport_text_overlay --locked`
  - 证明 editor-owned viewport HUD 已经进入 `EditorState::render_frame_submission()`，而不是停留在测试桩或旁路拼装
- `cargo test -p zircon_editor render_frame_submission_hud_text_renders_through_runtime_glyph_capture --locked`
  - 证明 editor-owned viewport HUD 文本已经通过 shared runtime text backend 进入真实 glyph capture，而不只是落到 render framework 统计
- 2026-04-29 fresh blocker follow-up 使用 `D:\cargo-targets\zircon-render-plugin-final` 重新验证这条 editor viewport HUD 链：`cargo test -p zircon_editor --lib tests::editing::state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --test-threads=1 --nocapture` 通过 11 passed / 0 failed；`cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --test-threads=1 --nocapture` 通过 47 passed / 0 failed
  - 同一轮确认 `SceneViewportController::build_runtime_overlay_ui()` 的生产路径不再是 `None` stub，并修正了 operation stack 查询测试中 `Remote -> Headless` source 预期；两者都通过 focused Cargo 回归
- `cargo test -p zircon_editor --lib --locked`
  - 当前这条更宽的 editor lib 验证在邻域漂移下仍被阻塞，fresh `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-render-plugin-final --color never -- --test-threads=1` 结果为 773 passed / 131 failed / 1 ignored；首个非级联失败是 UI asset binding schema 期待缺少 `onDrop`，后续还有 drawer-header hook、template host-model shape、asset-browser label、floating-window/shared-drawer projection 等 active UI/template failures，不在本 HUD/text 链上
- `cargo test --workspace --locked`
  - 当前全工作区验证同样仍被邻域 editor/hybrid-GI 漂移阻塞，因此 M1 这里记录的是 targeted boundary/text regressions 绿灯，而不是工作区全绿

这些验证合起来，把“目录规则”“shared 加载链路”“字体/typography 合同”“native/sdf 批次分流”同时锁住。
