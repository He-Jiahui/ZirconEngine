---
related_code:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/mod.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/data_binding/binding_target.rs
  - zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs
  - zircon_runtime_interface/src/ui/component/data_binding/projection_patch.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_result.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_error.rs
  - zircon_runtime_interface/src/ui/component/validation.rs
  - zircon_runtime_interface/src/ui/component/value.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime_interface/src/ui/binding/model/event_kind.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/value_validation.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/data_binding.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/showcase_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/mod.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/showcase.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/component_showcase.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_ui_asset_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/collection_fields.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/preview_images.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/showcase_actions.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_value_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs
  - zircon_editor/src/ui/slint_host/ui/structure_component_tests.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_selection.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - zircon_editor/src/tests/ui/mod.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/slint_host/app/inspector.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/reference_drop_payload.rs
  - zircon_editor/src/ui/slint_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/ui/tests/component_showcase.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/tests/drag_sources.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs
  - zircon_editor/src/ui/slint_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/slint_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_resize.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/component_widgets.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
implementation_files:
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/component/catalog/mod.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/registry.rs
  - zircon_runtime_interface/src/ui/component/category.rs
  - zircon_runtime_interface/src/ui/component/descriptor/mod.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime_interface/src/ui/component/state.rs
  - zircon_runtime/src/ui/component/data_binding/mod.rs
  - zircon_runtime_interface/src/ui/component/data_binding/binding_target.rs
  - zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs
  - zircon_runtime_interface/src/ui/component/data_binding/projection_patch.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_result.rs
  - zircon_runtime_interface/src/ui/component/data_binding/adapter_error.rs
  - zircon_runtime_interface/src/ui/component/validation.rs
  - zircon_runtime_interface/src/ui/component/value.rs
  - zircon_runtime/src/ui/template/asset/compiler/component_props.rs
  - zircon_runtime/src/ui/template/asset/compiler/node_expander.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime_interface/src/ui/binding/model/event_kind.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/value_validation.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs
  - zircon_editor/src/ui/template_runtime/builtin/showcase_template_bindings.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/categories.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/defaults.rs
  - zircon_editor/src/ui/template_runtime/showcase_demo_state/state_panel.rs
  - zircon_editor/src/ui/template_runtime/slint_adapter.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/mod.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/showcase.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/registry.rs
  - zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/component_showcase_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/component_showcase.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_ui_asset_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/collection_fields.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/preview_images.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/showcase_actions.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_menu_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_option_projection.rs
  - zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_value_conversion.rs
  - zircon_editor/src/ui/slint_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/slint_host/ui/reference_component_tests.rs
  - zircon_editor/src/ui/slint_host/ui/structure_component_tests.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_selection.rs
  - zircon_editor/src/tests/ui/mod.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/ui/slint_host/ui/tests/component_showcase.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs
  - zircon_editor/src/ui/slint_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/slint_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/slint_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/slint_host/app/inspector.rs
  - zircon_editor/src/ui/slint_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs
  - zircon_editor/src/ui/slint_host/app/reference_drop_payload.rs
  - zircon_editor/src/ui/slint_host/app/showcase_event_inputs.rs
  - zircon_editor/src/ui/slint_host/app/callback_wiring.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - zircon_editor/src/ui/slint_host/app/tests/drag_sources.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_layout_paths.rs
  - zircon_editor/src/ui/slint_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/slint_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/ui/slint_host/host_contract/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/window.rs
  - zircon_editor/src/ui/slint_host/host_contract/globals.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/mod.rs
  - zircon_editor/src/ui/slint_host/host_contract/data/template_nodes.rs
  - zircon_editor/tests/integration_contracts/workbench_slint_shell.rs
  - zircon_editor/tests/integration_contracts/workbench_window_resize.rs
  - zircon_editor/src/ui/host/builtin_views/activity_windows/activity_window_descriptors.rs
  - zircon_editor/src/ui/host/builtin_views/builtin_view_descriptors.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs
  - zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs
  - zircon_editor/src/ui/workbench/autolayout/constraints/defaults.rs
  - zircon_editor/build.rs
  - zircon_editor/assets/ui/editor/component_showcase.ui.toml
  - zircon_editor/assets/ui/editor/component_widgets.ui.toml
  - zircon_editor/assets/ui/theme/editor_material.ui.toml
plan_sources:
  - user: 2026-04-27 Runtime UI 组件库与 Slint Material Showcase Cutover
  - user: 2026-04-28 继续修复 Component Showcase Slint host retained row state 验证阻断
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - docs/superpowers/plans/2026-04-28-runtime-ui-drag-source-metadata.md
  - docs/superpowers/plans/2026-04-29-slint-fence-ui-toml-cutover.md
  - docs/superpowers/plans/2026-05-01-runtime-ui-real-data-source-adapter.md
  - docs/superpowers/plans/2026-05-01-runtime-ui-complex-components.md
  - docs/superpowers/specs/2026-04-28-runtime-ui-drag-source-metadata-design.md
  - docs/superpowers/specs/2026-05-01-runtime-ui-real-data-source-adapter-design.md
  - docs/superpowers/specs/2026-05-01-runtime-ui-complex-components-design.md
tests:
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/selection.rs
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/complex_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/data_binding.rs
  - zircon_editor/src/tests/host/builtin_window_descriptors.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_selection.rs
  - zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - zircon_editor/src/tests/ui/component_adapter.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/slint_window/generic_host_boundary.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/graphics_surface/runtime_ui_integration.rs
  - zircon_runtime/tests/runtime_ui_text_render_contract.rs
  - zircon_editor/src/ui/slint_host/ui/tests.rs
  - zircon_editor/src/ui/slint_host/app/tests.rs
  - cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never
  - cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture
  - .\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\zircon-ui-cutover-move-first
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\runtime_ui\runtime_ui_manager.rs zircon_runtime\src\tests\ui_boundary\runtime_host.rs zircon_editor\src\tests\host\slint_window\generic_host_layout_paths.rs
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_runtime --lib drop_binding_roundtrip_preserves_reference_payload_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check -- --nocapture
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-showcase-check -- --nocapture
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib builtin_activity_window_documents_are_registered_in_host_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib builtin_pane_views_expose_template_metadata --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_exercises_full_component_action_bindings --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor -- --nocapture
  - cargo test -p zircon_editor --lib showcase_demo_state_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib component_showcase_pane_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_numeric_drag_tracks_two_axis_delta --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_popup_options_dispatch_candidate_selection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_action_chips_dispatch_secondary_actions --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_visual_feedback_and_vector_primitives --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_reference_drop_wells --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_template_materializes_structure_and_collection_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-operation-path -- --test-threads=1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1
  - cargo test -p zircon_editor --lib host_projection_carries_runtime_component_properties_and_routes --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib slint_host_build_uses_material_style --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-check-tests -- --nocapture
  - cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1
  - rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - rustfmt --check zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - rustfmt --check zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - cargo test -p zircon_runtime ui::tests::component_catalog --lib
  - cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog
  - cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog --target-dir D:\cargo-targets\zircon-runtime-ui-catalog-props
  - cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never
  - cargo test -p zircon_editor --lib asset_browser_pointer_drop_applies_real_payload_to_showcase_asset_field --locked --jobs 1
  - cargo test -p zircon_editor --lib hierarchy_pointer_down_arms_scene_instance_payload_for_instance_field_drop --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-catalog-props -- --nocapture
  - cargo test -p zircon_editor --lib hierarchy_pointer_up_clears_scene_instance_payload --locked --jobs 1
  - cargo test -p zircon_editor --lib object_field_drop_accepts_active_scene_instance_payload --locked --jobs 1
  - cargo test -p zircon_editor --lib inspector_pointer_down_arms_active_object_payload_for_object_field_drop --locked --jobs 1
  - cargo test -p zircon_editor --lib object_field_drop_consumes_active_object_drag_payload --locked --jobs 1
  - cargo test -p zircon_editor --lib asset_field_drop_rejects_active_scene_instance_payload --locked --jobs 1
  - cargo test -p zircon_editor --lib instance_field_drop_rejects_active_asset_payload --locked --jobs 1
  - cargo test -p zircon_editor --lib component_showcase --locked --jobs 1
  - cargo test -p zircon_editor --lib slint_host --locked --jobs 1
  - cargo check -p zircon_editor --lib --locked --jobs 1
  - cargo fmt --package zircon_runtime -- --check
  - cargo fmt --package zircon_editor -- --check
  - cargo test -p zircon_runtime component_state_renames_map_keys_and_rejects_duplicate_targets --lib --locked --jobs 1
  - rustfmt --edition 2021 --check zircon_editor/src/ui/template_runtime/showcase_demo_state.rs zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - git diff --check -- zircon_editor/src/ui/template_runtime/showcase_demo_state.rs zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs
  - cargo test -p zircon_editor --lib showcase_search_select_query_edit_is_retained_and_projected --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never
  - cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never
  - cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never
  - cargo test -p zircon_editor --lib showcase_edit_input_maps_collection_row_payloads_to_typed_events --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_component_catalog_contains_showcase_v1_controls --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never
  - rustfmt --edition 2021 --check zircon_runtime/src/ui/component/event.rs zircon_runtime/src/ui/component/state.rs zircon_runtime/src/ui/component/catalog/editor_showcase.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_runtime/src/ui/tests/component_catalog/complex_components.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
  - cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never
  - cargo test -p zircon_editor --lib runtime_component_projection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never
  - git diff --check -- docs/superpowers/specs/2026-05-01-runtime-ui-complex-components-design.md docs/superpowers/plans/2026-05-01-runtime-ui-complex-components.md docs/ui-and-layout/runtime-ui-component-showcase.md .codex/sessions/20260429-0719-runtime-ui-showcase-schema-panel.md zircon_runtime/src/ui/component/event.rs zircon_runtime/src/ui/component/state.rs zircon_runtime/src/ui/component/catalog/editor_showcase.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_runtime/src/ui/tests/component_catalog/complex_components.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs
doc_type: module-detail
---

# Runtime UI Component Showcase

## Scope

This document records the first retained Runtime UI component-library slice. The goal is to make editor screen-space UI components explicit runtime semantics while keeping `.ui.toml` as the business source of truth and keeping Slint as a generic Material host.

The V1 acceptance target is not a visual mock. The showcase asset declares real component nodes, the runtime owns descriptors and typed event/state contracts, and host projection carries enough metadata for a generic renderer to show component role, value, validation, popup, selection, and drag/drop acceptance state. Interactive showcase rows also declare registered template bindings so Button, NumberField, Dropdown, Foldout, Array, Map, and reference-drop controls have concrete event entry points.

## Runtime Component Contracts

`zircon_runtime::ui::component` is the shared contract layer for editor-style controls. It defines:

- `UiComponentDescriptorRegistry`: the registry used by the editor showcase and host projection.
- `UiComponentDescriptor`: component id, category, Material-style role, typed prop schema, state schema, slot schema, supported events, and drop policy. Builder methods keep the short `state(...)` / `slot(...)` names for declaration, while query helpers use `state_prop(...)` / `slot_schema(...)` so Rust method names stay unambiguous.
- `UiPropSchema`, `UiOptionDescriptor`, and `UiSlotSchema`: typed declaration metadata for props, choices, and content slots.
- `UiValue` / `UiValueKind`: bool, int, float, string, color, vec2/3/4, asset reference, instance reference, array, map, enum, flags, and null values.
- `UiComponentEvent`: retained/event-driven control events such as value changed, commit, focus, drag delta and large-step drag delta, popup open/close, option selection, foldout toggle, array element add/set/remove/move, map add/set/rename-key/remove, typed reference drop, and reference clear/locate/open actions.
- `UiComponentState`: demo-retained state that applies events, honors per-control numeric min/max/step/large-step state, records validation errors, tracks focus/hover/press/drag/drop/popup/expanded flags, rejects disabled selection options, rejects non-numeric schema value-kind mismatches before mutating retained values, supports dropdown-style multi-selection arrays, rejects duplicate and missing map keys, and enforces drag/drop policy with rejection messages.

The registry currently includes the V1 showcase set:

- Visual and feedback: `Label`, `RichLabel`, `Image`, `Icon`, `SvgIcon`, `Separator`, `ProgressBar`, `Spinner`, `Badge`, `HelpRow`.
- Input: `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `Radio`, `SegmentedControl`, `InputField`, `TextField`.
- Numeric: `NumberField`, `RangeField`, `ColorField`, `Vector2Field`, `Vector3Field`, `Vector4Field`.
- Selection: `Dropdown`, `ComboBox`, `EnumField`, `FlagsField`, `SearchSelect`.
- Reference: `AssetField`, `InstanceField`, `ObjectField`.
- Structure and collection: `Group`, `Foldout`, `PropertyRow`, `InspectorSection`, `ArrayField`, `MapField`, `ListRow`, `TreeRow`, `ContextActionMenu`.

`SegmentedControl` is intentionally modeled as direct selection rather than popup selection. It supports focus and option selection state, but does not expose `popup_open`, `OpenPopup`, or `ClosePopup`; dropdown-like metadata remains on `Dropdown`, `ComboBox`, `EnumField`, `FlagsField`, `SearchSelect`, and `ContextActionMenu`.

The catalog also treats the authored showcase visual state as part of the component contract. Selection descriptors declare `validation_level`, `selection_state`, `value_text`, `multiple`, `popup_open`, and option id sets for `disabled_options`, `special_options`, `focused_options`, `hovered_options`, and `pressed_options`; reference descriptors declare `drop_hovered` and `active_drag_target` as both props and retained state; collection rows expose `selected`, `focused`, `hovered`, `tree_depth`, and `tree_indent_px`; ContextActionMenu exposes popup anchor and structured menu metadata. This keeps `.ui.toml` rows, retained `UiComponentState`, and generic host-contract projection aligned instead of leaving visual state as untyped ad hoc props.

The `.ui.toml` compiler now consults `UiComponentDescriptorRegistry::editor_showcase()` for native nodes whose `type` matches a Runtime UI component descriptor. `component_props.rs` merges descriptor defaults and per-prop schema defaults before authored props, then validates typed authored/default values and numeric min/max ranges. Unknown layout primitives such as `VerticalBox`, `HorizontalBox`, and `ScrollableBox` remain on the generic template path, and style-only attributes that are not component props are preserved so existing UI assets can still use generic visual attributes. This closes the contract loop where Runtime UI descriptors are not just documentation for host projection but also participate in retained template compilation.

## Showcase Assets

The showcase window is declared in `zircon_editor/assets/ui/editor/component_showcase.ui.toml`. It imports `component_widgets.ui.toml` for the reusable `ShowcaseSection` widget and `editor_material.ui.toml` for style tokens.

The window layout follows a Rider/Unity inspector shape:

- left category navigation with stable control ids for visual/input/collection groups;
- center scroll area with Visual, Input/Numeric, Selection/Reference, and Collection/Inspector groups;
- right state panel with `.ui.toml`-authored `PropertyRow` nodes for selected category, last control/action, current value, validation, drag/drop payload summary, and retained event log.

Every V1 component has a native node row in the showcase. Those rows deliberately remain `.ui.toml` data. There is no component-specific `.slint` tree for the showcase. Authored complex collection components follow the runtime catalog schema: `VirtualList` and `PagedList` author retained row data through the `items` prop, while `collection_items` is reserved for generated host projection rows after state slicing and collection-row materialization.

Interactive rows use `[[...bindings]]` in the `.ui.toml` asset and matching entries in `template_bindings.rs`. The binding payloads are `EditorUiBindingPayload::Custom(UiBindingCall::new("UiComponentShowcase"))` with the demo action and control id as arguments. NumberField drag and large-step drag, RangeField change, popup open/close, dropdown option selection, reference drop/clear/locate/open, Array add/set/remove/move, and Map add/set/rename-key/remove actions are all declared by the asset and registry rather than by a handwritten showcase `.slint` tree. This keeps the showcase event surface event-driven and retained without teaching the host any showcase-specific business layout.

`EditorUiHostRuntime` owns the showcase transient state through `showcase_demo_state.rs`. The reducer consumes the projected binding payloads, maps them to typed `UiComponentEvent` values, applies them to retained `UiComponentState`, and records an event log entry with the control id and changed display value. The reducer covers category selection, button commit, text/value change, toggles, numeric drag begin/update/end and large-step drag, popup open/close, dropdown selection, typed reference drops, reference clear/locate/open actions, foldout/group expansion, Array add/set/remove/move, and Map add/set/rename-key/remove operations.

The retained state is projected back onto the Rust-owned host model. `runtime_host.rs` asks the showcase state to overlay current `value`, authored/catalog `items`, `entries`, generated `collection_items`, `expanded`, `checked`, `focused`, `hovered`, `pressed`, `dragging`, `drop_hovered`, `active_drag_target`, `popup_open`, validation, reference source summaries, and event-log text onto matching host nodes. Category navigation is also retained: the initial `All` state keeps the full showcase visible for broad projection coverage, while `SelectCategory.Visual`, `SelectCategory.Feedback`, `SelectCategory.Input`, `SelectCategory.Numeric`, `SelectCategory.Selection`, `SelectCategory.Reference`, and `SelectCategory.Collections` mark the selected nav button and filter projected demo controls to the matching component family. A fresh projection after an event therefore shows the updated NumberField value, focused list row state, selected dropdown value, open or closed popup state, dropped reference, generated Array/Map child rows, selected category subset, collection counts, and event log without mutating the `.ui.toml` structure.

The right state panel is retained projection, not handwritten host business UI. The `.ui.toml` asset declares stable `PropertyRow` controls such as `ComponentShowcaseSelectedCategory`, `ComponentShowcaseLastControl`, `ComponentShowcaseLastAction`, `ComponentShowcaseCurrentValue`, `ComponentShowcaseValidation`, and `ComponentShowcaseDragPayload`. `showcase_demo_state/state_panel.rs` fills those rows from the last retained event, current `UiComponentState`, validation state, interaction flags, and retained `UiDragSourceMetadata`, so the same Runtime UI state machine drives both the central control rows and the diagnostic panel.

`UiEventKind::Drop` maps to `onDrop` so `AssetField`, `InstanceField`, and `ObjectField` can expose reference-drop semantics directly rather than tunneling through a click or change event.

## Host Contract Boundary

As of the 2026-04-30 follow-up, active `.slint` sources are no longer editor build, runtime, test, or documentation authority. New business UI work must continue through `.ui.toml`, Runtime UI projection, and Rust-owned host contracts rather than editing, restoring, or reading deleted Slint files.

The active tree is now explicitly guarded: `generic_host_layout_paths.rs` asserts no active `.slint` remains under `zircon_editor/ui`, and `generic_host_boundary.rs` forbids editor host sources from depending on deleted Slint source trees, generated Slint includes, `slint_build` / `slint-build` build seams, or `as slint_ui` compatibility aliases. `zircon_editor/build.rs` no longer compiles active `ui/workbench.slint`, no longer stages deleted Slint sources into `OUT_DIR`, and no longer calls `slint_build`.

The generated root seam is also removed: `zircon_editor/src/ui/slint_host/mod.rs` no longer exports `slint::include_modules!()`, and the Rust-owned replacement surface lives under `zircon_editor/src/ui/slint_host/host_contract/**`. The presentation, pane, template-node, and component-showcase conversion code now uses `to_host_contract_*` helpers for the Rust-owned host-contract DTO projection instead of local `to_slint_*` conversion names. The integration contract readers now assert Rust-owned host contracts and `.ui.toml` assets directly instead of joining active or deleted `zircon_editor/ui/workbench*.slint` paths.

Runtime UI input dispatch now stays on the same shared-surface boundary. `RuntimeUiManager::dispatch_pointer_event(...)` and `RuntimeUiManager::dispatch_navigation_event(...)` are crate-local forwarding methods over the owned `UiSurface`, so the manager can act as the runtime host facade while capture, focus, and navigation handling remain implemented by the shared tree/surface dispatch contracts.

The graphics acceptance path now covers every builtin Runtime UI fixture, not only the pause menu and clipped inventory list. The feature-gated `runtime_ui_integration.rs` acceptance submits each fixture through `RuntimeUiManager::build_frame()` into the screen-space UI pass and verifies that render stats show non-trivial UI command output plus quad or text payload contribution. This keeps graphics validation at the Runtime UI boundary instead of adding fixture-specific renderer behavior.

Before the fence, the Slint build style was switched to Material in `zircon_editor/build.rs`, and the `.slint` files acted as host primitives and generated DTO surfaces. The historical notes below describe that pre-fence state only; current validation and implementation must not inspect deleted Slint source copies.

Two Rust-owned host paths now carry runtime component semantics:

- `EditorUiHostRuntime -> host_contract` projection exposes component role, value text, validation, popup state, selection state, SearchSelect query state, option summary text, individual option ids, checked/expanded/disabled, commit/edit/drag action ids, and accepted drag payload metadata for tests and host-level projections.
- `pane_data_conversion/mod.rs` derives the same component metadata for generic `TemplatePaneNodeData`, including both popup option summary text and the individual candidate list. The Rust-owned host projection can therefore consume runtime component rows, popup candidate rows, and numeric drag action metadata without knowing showcase-specific structure.

The Rust-owned host projection consumes `TemplatePaneNodeData` generically. It does not contain the list of showcase controls, groups, labels, or demo values.

`UiComponentShowcase` now also has a pane-body payload and template metadata, so a docked Activity Window is rendered by the generic host template path instead of falling through to `FallbackPane`. The pane conversion path uses `editor.window.ui_component_showcase`, computes layout for the available pane size, and forwards the resulting runtime component nodes to the Rust-owned host-contract pane projection.

SearchSelect rows project the authored or retained `.ui.toml` `query` prop into `TemplatePaneNodeData.search_query`. The generic TemplatePane uses that retained query text for the SearchSelect primitive alongside the current selected value, and `SearchSelectQueryChanged` routes live query edits back through the retained `UiComponentState` `query` property, so the search/filter state remains runtime/UI-asset data instead of becoming a hardcoded Slint showcase label.

Editable field rows project a separate `TemplatePaneNodeData.commit_action_id` alongside `edit_action_id`. InputField, TextField, NumberField, and RangeField therefore keep live `ValueChanged.*` edits and submitted `Commit.*` events as distinct Runtime UI bindings, while legacy/default template-node conversion leaves the commit id empty for non-component rows.

The editor host owns a dedicated `EditorUiHostRuntime` for the showcase transient state. During host startup, `host_lifecycle.rs` loads the builtin Runtime UI templates into that runtime, and `apply_presentation.rs` threads the runtime into docked panes and native floating panes before converting Runtime UI nodes into `TemplatePaneNodeData`. The default scene projection path remains generic; the supplied runtime is only used to overlay retained demo values onto the Runtime UI projection.

Generic component rows now carry `dispatch_kind = "showcase"` and the selected binding action id when the projected runtime node exposes a `UiComponentShowcase/...` binding. Popup-capable controls prefer an `OpenPopup` action while closed and switch to their `SelectOption` action once the retained state says the popup is open. Their TOML `options` array is projected as host-contract/runtime models, so the popup layer renders individual candidate rows rather than only a single summary label. `ContextActionMenu` can also project retained `menu_items` as structured `TemplatePaneMenuItemData` rows through `structured_menu_items`, with dedicated `raw`, stable `action_id`, `label`, `shortcut`, `checked`, `disabled`, and `separator` fields parsed from `.ui.toml`; the generic popup layer renders those menu rows without embedding showcase menu content in deleted Slint files, and disabled or separator rows do not dispatch selection. Candidate row clicks dispatch the selected option id through `PaneSurfaceHostContext.component_showcase_option_selected`, with structured menu rows sending `action_id` rather than the encoded display row. `callback_wiring.rs` forwards that generic option event into Rust host state. Numeric rows carry `begin_drag_action_id`, `drag_action_id`, and `end_drag_action_id`, so pointer down/update/up can dispatch `BeginDrag`, `DragDelta`, and `EndDrag` through the same retained event reducer while ordinary activation remains event-driven. Editable fields carry a separate `edit_action_id`, allowing InputField, TextField, NumberField, and RangeField rows to use a generic `LineEdit` and dispatch live value text through the same `.ui.toml` binding and retained reducer path. Multi-operation rows also carry generic `TemplatePaneActionData` chips, so AssetField exposes Find/Open/Clear, ArrayField exposes Add/Set/Remove/Move, and MapField exposes Add/Set/Remove directly in the showcase window. `TemplatePaneNodeData` also projects `value_number`, normalized `value_percent`, parsed `value_color`, `selected`, `focused`, `hovered`, `pressed`, `dragging`, `drop_hovered`, `tree_depth`, `tree_indent_px`, generated `collection_items`, structured Array/Map `collection_fields`, and structured menu metadata, allowing the retained host to render Material-like checkbox, radio, switch, progress bar, range track, color swatch, list focus, tree indentation, tree/collection summaries, inline collection edit rows, context menu rows, and reference drop-hover wells without hardcoding the showcase component list into deleted Slint files. The showcase state overlay preserves declarative ListRow/TreeRow visual props unless a runtime flag is active, so default inactive state does not erase selected, focused, hovered, or tree-depth authored data before host-contract conversion. It now also projects `media_source`, `icon_name`, `has_preview_image`, `preview_image`, and typed `vector_components`, so Image, Icon, SvgIcon, Separator, Spinner, Badge, HelpRow, and Vector2/3/4 rows render as retained host primitives instead of falling back to plain value text. Preview loading resolves document-relative media sources under `assets/`, icon library paths under `assets/icons/`, and semantic Ionicons names under `assets/icons/ionicons/<icon>.svg`, which keeps `.ui.toml` image/icon data reusable by plugin component drawers. Reference controls use the existing retained metadata as a Material drop well: AssetField, InstanceField, and ObjectField show the current reference value, accepted drag payload kinds, validation or rejected-drop messages, and action chips without requiring new business Slint or duplicate projection fields. Structure and collection controls now use the same retained surface: Group, Foldout, InspectorSection, PropertyRow, ArrayField, MapField, ListRow, and TreeRow render disclosure/summary chrome from `expanded`, `value_text`, `selection_state`, `selected`, generated child-row metadata, and action-chip metadata instead of plain value text. The Rust-owned host-contract pane surface has generic activation, edit, drag, option-selection, action-chip, and role-specific primitive data for component rows; it tracks two-dimensional pointer movement and maps right/up movement to positive deltas and left/down movement to negative deltas for NumberField and RangeField style controls. Asset browser content routing and pane content routing share down/up/move/scroll dispatch through Rust-owned callback wiring, and `callback_wiring.rs` forwards those callbacks into Rust host state.

`pane_surface_actions.rs` maps the generic control activation, live edit, drag delta, option selection, or secondary action chip back to the projected `EditorUiBinding`, applies `showcase_demo_state.rs` through `EditorUiHostRuntime::apply_showcase_demo_binding`, marks the presentation dirty, and lets the next retained projection redraw the updated value or validation message. Numeric/edit/action demo input construction lives in `showcase_event_inputs.rs`: numeric live edits parse valid number text into float values and pass invalid text through as a string so the retained numeric validation path can surface the error, collection row payloads become typed Array/Map row events, and option selections become `UiComponentShowcaseDemoEventInput::SelectOption` so Dropdown/ComboBox/Enum/Flags/SearchSelect/ContextActionMenu rows use the same runtime reducer as non-hosted tests. Secondary action chips reuse the existing binding ids for reference clear/locate/open, Array add/set/remove/move, and Map add/set/remove operations. Real reference-drop source selection lives in `reference_drop_payload.rs`, keeping active asset/scene/object payload priority and stale-slot cleanup out of the broader pane action dispatcher. Module plugin enablement, packaging, and target-mode actions live in `module_plugin_actions.rs`, so the pane surface dispatcher no longer owns plugin manifest parsing or status label formatting. `pane_data_conversion/mod.rs` remains the pane-level orchestration boundary; Runtime component projection now lives in `pane_data_conversion/pane_component_projection/mod.rs`, while TOML scalar/list/color parsing lives in `pane_data_conversion/pane_value_conversion.rs`. Rust-owned host contracts therefore remain a generic host surface, while `.ui.toml` and the runtime reducer remain the business truth for the showcase interaction.

The 2026-04-29 reference-action parity follow-up extends the secondary Find/Open/Clear chip route from AssetField to InstanceField and ObjectField. The authored showcase asset and builtin binding table now expose `InstanceFieldLocate`, `InstanceFieldOpen`, `InstanceFieldClear`, `ObjectFieldLocate`, `ObjectFieldOpen`, and `ObjectFieldClear`, and `showcase_actions.rs` projects those bindings as the same generic `TemplatePaneActionData` rows used by AssetField.

Asset Browser content-row pointer events now arm a real `UiDragPayload` for asset items. The payload keeps `kind = Asset` and writes the asset locator as the reference value, while optional `UiDragSourceMetadata` carries source surface, source control id, asset UUID, locator, display name, asset kind, and extension. Accepted `AssetField` drops retain that metadata in `UiComponentState` and project a generic `drop_source_summary` string so the Rust-owned host projection can show the source without hardcoding asset-browser behavior.

Hierarchy pane pointer events now arm a real `UiDragPayload` for scene entries. Left-button down routes through the existing hierarchy pointer bridge, converts node hits into `kind = SceneInstance` payloads with `scene://node/<id>` references, and stores `UiDragSourceMetadata` with `source_surface = "hierarchy"`, `source_control_id = "HierarchyListPanel"`, display name, and a `Scene Instance` source kind. Left-button up clears the active scene payload. `InstanceField` and `ObjectField` drops consume this active scene payload before falling back to showcase synthetic payloads, so scene-tree drag intent reaches the retained Runtime UI drop reducer without adding business UI structure to deleted Slint files.

## Validation Coverage

Runtime tests cover registry completeness, authored showcase prop coverage for validation, popup, option-state, drag/drop, checked, media, row, tree, menu, Array, and Map metadata, direct segmented-control selection semantics, popup selection semantics, NumberField drag/clamp, large-step drag and invalid commit validation, disabled selection option rejection with retained validation state, non-numeric schema value-kind mismatch rejection, special selection option metadata, Dropdown multi-selection, FlagsField retained selection state, Array add/set/remove/move, Map add/set/remove with duplicate and missing-key rejection, and AssetField/InstanceField/ObjectField typed drop plus reference clear/locate/open semantics.

Runtime/editor tests cover:

- `editor.ui_component_showcase` registration as an Activity Window with the correct template document id;
- builtin template loading and projection of `editor.window.ui_component_showcase`;
- runtime component metadata in Rust-owned host-contract projection for NumberField, Dropdown, and AssetField, including editable `edit_action_id`, submit/accept `commit_action_id`, numeric `drag_action_id`, popup option summary text, structured option ids, and individual popup candidate rows for dropdown-like controls;
- materialized primitive metadata and rendering hooks for Checkbox, Radio, ToggleButton, ProgressBar, RangeField, and ColorField, including projected `value_number`, normalized `value_percent`, and parsed `value_color`;
- materialized primitive metadata and rendering hooks for Image, Icon, SvgIcon, Separator, Spinner, Badge, HelpRow, and Vector2/3/4 fields, including projected `media_source`, `icon_name`, loaded `preview_image`, `has_preview_image`, and typed vector component models;
- Material reference drop-well rendering for AssetField, InstanceField, and ObjectField, including accepted drag payload metadata, current reference display, validation/rejection message display, and action-chip placement that does not cover the reference value;
- Inspector-style structure and collection row rendering for Group, Foldout, InspectorSection, PropertyRow, ArrayField, MapField, ListRow, and TreeRow, including retained expanded/collapsed state, value summaries, and action-chip placement beside collection summaries;
- deeper retained structure coverage for generated Array/Map child rows, structured Array/Map `collection_fields`, ListRow selected/focused/hovered state projection, TreeRow `tree_depth`/`tree_indent_px` hierarchy projection, NumberField dragging projection, ContextActionMenu menu-row metadata, stable menu `action_id` dispatch, checked/disabled/separator/shortcut menu parsing, and host-contract rendering data for `selected`, `focused`, `dragging`, `drop_hovered`, `tree_indent_px`, `collection_items`, `collection_fields`, and `structured_menu_items`;
- SearchSelect query projection from `.ui.toml` into `TemplatePaneNodeData.search_query`, retained `SearchSelectQueryChanged` mutation, and generic host-contract rendering data for the retained query text;
- showcase binding projection for NumberField drag/commit, Dropdown change, popup open/close, and AssetField drop, including the custom runtime demo action payload;
- retained showcase demo state application for projected bindings, including category selection, input value mutation, editable-field commit mutation, SearchSelect query mutation, toggle state, numeric drag and large-step drag, popup open/close, dropdown selection, typed asset drop, reference clear/locate/open, Array add/set/move/remove, Map add/set/remove, and event-log recording;
- Runtime UI asset compilation of registered component schema defaults and typed prop rejection, while preserving generic style attributes not declared by component schemas;
- retained category navigation filtering that keeps the default all-components projection for broad coverage, then narrows host projection after Visual, Feedback, Input, Numeric, Selection, Reference, or Collections navigation events;
- retained state-panel projection that shows selected category, last control/action, current value, validation, and retained drag/drop source summaries through `.ui.toml` `PropertyRow` nodes;
- Runtime UI manager pointer and navigation dispatch forwarding through the owned shared `UiSurface`, including focus capture and handled navigation results;
- feature-gated all-fixture Runtime UI submission into `WgpuRenderFramework`, plus text-render contract coverage for the formal UI pipeline;
- real Asset Browser content-row pointer events arming a metadata-backed `UiDragPayload`, accepted `AssetField` drops retaining `UiDragSourceMetadata`, generic host projection of `drop_source_summary` without asset-browser-specific template logic, and an end-to-end pointer-drop safeguard proving a real asset payload reaches the showcase AssetField;
- real Hierarchy pane pointer events arming metadata-backed `SceneInstance` payloads, pointer-up clearing stale scene payloads, `InstanceField` consumption of the scene reference, and `ObjectField` accepting the same active scene-instance payload through the shared retained drop reducer;
- docked `UiComponentShowcase` pane conversion to Rust-owned host-contract `TemplatePaneNodeData` so the showcase uses the Runtime UI template instead of a fallback pane;
- no-Slint source/build guard coverage for the Rust-owned host-contract seam.

## Recent Validation

On 2026-04-30, Task 7 Runtime UI graphics fixture acceptance was added and validated under the existing `runtime-ui-integration-tests` feature. `cargo test -p zircon_runtime --lib render_framework_submits_all_builtin_runtime_ui_fixtures --features runtime-ui-integration-tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 1 test / 0 failed / 1195 filtered out after waiting for an artifact lock, and `cargo test -p zircon_runtime --test runtime_ui_text_render_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime-graphics --message-format short --color never -- --test-threads=1 --nocapture` passed 7 tests / 0 failed.

On 2026-04-30, Task 6 Runtime UI dispatch acceptance was validated through the manager facade and the existing runtime boundary suite. `cargo test -p zircon_runtime --lib runtime_ui_manager_dispatches_pointer_and_navigation_through_shared_surface --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 1 test / 0 failed / 1190 filtered out, `cargo test -p zircon_runtime --lib ui_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed 17 tests / 0 failed / 1174 filtered out, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-runtime --message-format short --color never` passed. Targeted formatting also passed after applying rustfmt: `rustfmt --edition 2021 --check zircon_runtime\src\ui\runtime_ui\runtime_ui_manager.rs zircon_runtime\src\tests\ui_boundary\runtime_host.rs zircon_editor\src\tests\host\slint_window\generic_host_layout_paths.rs`.

On 2026-04-30, the no-Slint Task 5 equivalent guards were rechecked after the hard fence: `cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 3 tests / 0 failed / 841 filtered out, and `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed / 838 filtered out. The earlier parallel attempt for those editor guards timed out while waiting on Cargo package/artifact locks; the sequential reruns are the accepted evidence.

On 2026-04-30, the Module Plugins pane-body backfill was rechecked as a Task 4 `.ui.toml` slice. `module_plugins_body.ui.toml` is registered as `pane.module_plugins.body`, `ModulePluginsPaneBody/FocusModulePlugins` projects through the builtin binding table, `module_plugin_list_slot` is the stable hybrid slot, and `PanePayload::ModulePluginsV1` carries plugin catalog data into the runtime host projection. Focused checks passed with `cargo test -p zircon_editor --lib builtin_pane_body_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (2 tests / 0 failed), `cargo test -p zircon_editor --lib builtin_hybrid_pane_body_documents_declare_stable_native_slot_names --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (1 test / 0 failed), and `cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` (1 test / 0 failed).

On 2026-04-30, the Slint fence root seam was revalidated after removing both generated root staging and `slint::include_modules!()`. The follow-up alias guard `cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` first failed on `as slint_ui` imports in the presentation and pane conversion modules, then passed after those modules imported the Rust-owned surface as `host_contract`. The generated-build dependency guard `cargo test -p zircon_editor --lib editor_host_source_guard_rejects_hyphenated_generated_build_dependency --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` first failed before `slint-build` was forbidden, then passed after the source marker was added. `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed / 837 filtered out. The focused pane body check `cargo test -p zircon_editor --lib builtin_pane_body_ --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` passed 2 tests / 0 failed after `ModulePluginsPaneBody/FocusModulePlugins` was registered. Full editor lib validation `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` passed 841 tests / 0 failed / 1 ignored, and `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --nocapture` passed 27 tests / 0 failed. This is scoped editor evidence for the fenced Rust-owned host-contract seam; it does not claim workspace-wide acceptance.

The same continuation reran focused no-Slint guards after docs/session cleanup: `cargo test -p zircon_editor --lib editor_host_sources_do_not_depend_on_deleted_slint_trees --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 1 test / 0 failed; `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 6 tests / 0 failed; and `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never` passed 27 tests / 0 failed. An earlier same-day workspace validator attempt passed `cargo build --workspace --locked` and then stopped at the active pluginization nested Cargo boundary because `zircon_plugins/Cargo.lock` still needed the fixture rename / Slint build-dependency refresh. That intermediate diagnostic was lockfile state in `zircon_plugins`, not a regression in the Rust-owned editor host-contract seam, and it was superseded by the final validator pass below.

The same-day final rerun superseded the earlier nested plugin lockfile blocker after the fixture lockfile state settled. Focused editor verification passed in the shared target: `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never`, `cargo test -p zircon_editor --doc --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never`, `cargo test -p zircon_editor --lib generic_host_boundary --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`, `cargo test -p zircon_editor --lib generic_host_layout_paths --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture`, and `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-cutover-move-first --message-format short --color never -- --test-threads=1 --nocapture` all passed. Workspace validation then passed with `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir E:\cargo-targets\zircon-ui-cutover-move-first`: `cargo build --workspace --locked` OK and `cargo test --workspace --locked` OK. Final source searches found no active `zircon_editor/ui/**/*.slint`, no remaining `.slint` files under `temp/slint-migration`, no live `to_slint_*` conversion helpers in editor host source, and only test names or no-Slint absence guards for remaining `slint` text hits.

On 2026-04-27, the focused showcase cutover checks passed with `--locked` and `--jobs 1` against `D:\cargo-targets\zircon-codex-editor-check-tests`:

- `cargo test -p zircon_editor --lib component_showcase`
- `cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics`
- `cargo test -p zircon_editor --lib component_showcase_template_popup_options_dispatch_candidate_selection`
- `cargo test -p zircon_editor --lib component_showcase_template_action_chips_dispatch_secondary_actions`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_runtime_component_primitives`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_visual_feedback_and_vector_primitives`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_reference_drop_wells`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_structure_and_collection_rows`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane`
- `cargo test -p zircon_editor --lib component_showcase_template_fields_dispatch_live_edits`
- `cargo test -p zircon_runtime --lib ui::tests::component_catalog`
- `cargo test -p zircon_editor --lib slint_host_build_uses_material_style`
- `cargo check -p zircon_editor --lib`
- `cargo test -p zircon_editor --lib builtin_activity_windows_expose_window_template_documents`
- `cargo test -p zircon_editor --lib host_projection_carries_runtime_component_properties_and_routes`
- `cargo test -p zircon_editor --lib builtin_activity_window_documents_are_registered_in_host_runtime`
- `cargo test -p zircon_editor --lib builtin_pane_views_expose_template_metadata`

During this validation loop, an intermediate compile attempt briefly observed a mismatched `store_last_runtime_outputs` signature while parallel runtime graphics work was changing the same support layer. A fresh rerun after that support-layer source settled passed the showcase and editor checks above.

On 2026-04-28, the deeper retained-state slice added tests for collection child-row projection, complete control-state projection, number-field begin/end drag dispatch, and generic ContextActionMenu menu rendering. After concurrent Cargo sessions drained and unrelated compile drift in active graphics/editor work settled, the focused Windows checks passed with `--locked` and `--jobs 1`:

- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library -- --nocapture`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor -- --nocapture`
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library`
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-component-library-editor`

The `component_showcase` editor filter passed 16 focused tests, and the runtime component catalog filter passed 10 tests. Existing unrelated warnings remained visible in the Windows output as dead-code warnings in runtime graphics advanced plugin output-access modules. No Runtime UI showcase failures were observed.

Later on 2026-04-28, the host projection code was split so `pane_data_conversion/mod.rs` no longer owns the component-specific conversion table. The focused WSL checks used `CARGO_TARGET_DIR=/tmp/zircon-target-ui-showcase` and passed:

- `cargo check -p zircon_editor --lib --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`

The first broad `component_showcase` rerun timed out during test-profile rebuild before producing a Rust or Slint diagnostic; the narrower pane projection test completed after compiling the test profile, and the immediate full `component_showcase` rerun then passed 16 tests. The same unrelated runtime graphics dead-code warnings and WSL console-size notices remained visible.

The follow-up Array/Map row-edit slice on 2026-04-28 added `TemplatePaneCollectionFieldData` and projects typed `collection_fields` for ArrayField and MapField rows. The focused WSL checks used the same target directory and passed:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_collection_field_edit_rows --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The first RED run of `component_showcase_template_materializes_collection_field_edit_rows` failed on the expected missing `collection_fields` assertion. After implementation, the full `component_showcase` filter passed 17 focused tests. WSL reported artifact-directory lock waits while other Cargo sessions were active, but no Runtime UI showcase failure remained.

The follow-up ContextActionMenu slice on 2026-04-28 added `TemplatePaneMenuItemData` and projects structured `structured_menu_items` for menu rows. The focused WSL checks used `CARGO_TARGET_DIR=/tmp/zircon-target-ui-showcase` and passed:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_structured_context_menu_rows --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The first RED run of `component_showcase_template_materializes_structured_context_menu_rows` failed at compile time on the expected missing `structured_menu_items` projection field. After implementation, the full `component_showcase` filter passed 18 focused tests. The first green attempt exceeded the tool timeout while compiling dependencies, but the underlying Cargo process drained and the immediate rerun passed without any Runtime UI showcase failure.

The follow-up ListRow/TreeRow slice on 2026-04-28 added explicit selected and tree hierarchy projection for generic component rows. `ListRowDemo` declares selected/focused/hovered state in `.ui.toml`, `TreeRowDemo` declares depth and indentation, and `template_pane.slint` applies the retained indentation to the disclosure icon and text columns. The focused WSL checks used `CARGO_TARGET_DIR=/tmp/zircon-target-ui-showcase` and passed:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_tree_and_list_row_state --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The first RED run failed on the expected missing template use of `tree_depth`. After implementation, the full `component_showcase` filter passed 19 focused tests. `cargo check` passed with unrelated unused warnings in `zircon_editor/src/ui/slint_host/app/asset_drag_payload.rs`.

At that stage, asset and instance drag/drop were semantic contracts and demo transient state. The active `runtime-ui-drag-source-metadata` session first added asset-browser row integration through existing editor data sources without changing the component descriptor layer. The later Hierarchy follow-up now covers scene-tree scene-instance payload arming for `InstanceField` and `ObjectField`, while other non-asset drag sources such as dedicated instance lists remain outside this slice.

The latest Windows follow-up keeps the generic popup layer aligned with retained `.ui.toml` metadata by projecting `has_popup_anchor`, `popup_anchor_x`, and `popup_anchor_y` through `TemplatePaneNodeData`; `ContextActionMenu` now positions its popup from authored anchor data when present and falls back to the row frame otherwise. The same pass also fixed the real asset-browser drag-source fixture so `PaneSurfaceHostContext.asset_content_pointer_event` is validated with a visible catalog row and produces `UiDragPayload` through the normal Slint host -> asset pointer bridge path instead of an empty no-project snapshot. The focused `asset_drag_payload`, `component_showcase`, `tests::editor_event::runtime`, and `zircon_app` editor CLI parser filters passed against `D:\cargo-targets\zircon-codex-editor-component-drawer-bindings`.

The Hierarchy scene-instance drag-source follow-up on 2026-04-28 adds `PaneSurfaceHostContext.hierarchy_pointer_event`, routes left-button down/up from `HierarchyPaneView`, stores `active_scene_drag_payload` on the Slint editor host, and consumes that payload for `InstanceFieldDropped` and `ObjectFieldDropped`. Focused tests were added for hierarchy pointer-down arming, pointer-up clearing, and ObjectField scene-instance consumption. `cargo fmt --package zircon_editor` passed, `git diff --check` passed for the tracked touched files with only LF-to-CRLF warnings, and static symbol checks confirmed the new callback and payload path. The focused Cargo run for `hierarchy_pointer_down_arms_scene_instance_payload_for_instance_field_drop` timed out in the local Cargo compile queue before any Rust/Slint diagnostic; a later 300-second rerun against `D:\cargo-targets\zircon-runtime-ui-catalog-props` also timed out before producing test output. No residual Cargo/rustc process for that target or test remained afterward, while unrelated concurrent validation processes were left untouched. Pending reruns:

- `cargo test -p zircon_editor --lib hierarchy_pointer_down_arms_scene_instance_payload_for_instance_field_drop --locked --jobs 1`
- `cargo test -p zircon_editor --lib hierarchy_pointer_up_clears_scene_instance_payload --locked --jobs 1`
- `cargo test -p zircon_editor --lib object_field_drop_accepts_active_scene_instance_payload --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The follow-up module-slimming pass moves real drag-source tests out of the oversized `zircon_editor/src/ui/slint_host/app/tests.rs` host test file and into `zircon_editor/src/ui/slint_host/app/tests/drag_sources.rs`. The extracted module owns Asset Browser payload metadata tests, AssetField real/drop fallback tests, Hierarchy scene-instance payload tests, and ObjectField scene-instance payload consumption. `app/tests.rs` remains the shared host harness and pointer-routing test boundary, dropping from roughly 1880 lines to roughly 1454 lines before further feature work. Formatting passed with `cargo fmt --package zircon_editor`; focused Cargo compilation remains pending behind the same local compile queue pressure.

The WSL follow-up on 2026-04-28 verifies the generic media-preview path for the showcase: `TemplatePaneNodeData` carries `preview_image: image` and `has_preview_image`, `ImageDemo` uses the local `ui/editor/showcase_checker.svg` asset, `IconDemo` resolves `icon_name` through the ionicons asset folder, and `SvgIconDemo` resolves its retained source into a loaded Slint SVG image with non-zero dimensions. The same WSL pass verifies the ContextActionMenu popup anchor values projected from `.ui.toml`.

The next Array/Map row-edit follow-up on 2026-04-28 extends `TemplatePaneCollectionFieldData` with `edit_action_id`. ArrayField and MapField child rows now project their row-specific Set binding ids, and the generic collection `LineEdit` dispatches `row_id=value` through `component_showcase_control_edited`. `pane_surface_actions.rs` converts `array-<index>=...` payloads into `UiComponentShowcaseDemoEventInput::SetElement` and `map-<key>=...` payloads into `SetMapEntry`, parsing numeric collection text into `UiValue::Float` and keeping non-numeric text as `UiValue::String`. This keeps row-level editing in the retained event path instead of adding showcase-specific Slint business logic.

Validation note for this follow-up: `cargo fmt --package zircon_editor` passed, `git diff --check` passed for the touched files with only existing LF-to-CRLF warnings, and a first Windows compile attempt caught and fixed an invalid `SlintUiHostBindingProjection.dispatch_kind` field access. Focused Cargo tests could not be completed in this pass because concurrent Windows and WSL Cargo sessions repeatedly held editor/runtime build work for more than the tool timeout; the timed-out Cargo processes started by this pass were terminated, while unrelated concurrent validation processes were left untouched. The pending focused checks are:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_collection_field_edit_rows --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The active drag target follow-up on 2026-04-28 fills the last retained control-state gap from the V1 projection list. `TemplatePaneNodeData` now carries `active_drag_target`, `pane_data_conversion/pane_component_projection/mod.rs` reads it from `.ui.toml`, legacy template-node conversion defaults it to false, and `template_pane.slint` renders active drag targets with a stronger border/background than ordinary drop-hover. `AssetFieldDemo` marks both `drop_hovered` and `active_drag_target` in `.ui.toml`, giving the showcase a static retained example of an active reference drop target without touching real asset-browser drag-source ownership.

Validation note for this active-state follow-up: `cargo fmt --package zircon_editor` passed, `git diff --check` passed for the touched files with only existing LF-to-CRLF warnings, and static inspection confirmed `active_drag_target` is present in the shared Slint DTO, generic template styling, runtime component projection, legacy conversion defaults, and showcase asset. The focused WSL test `cargo test -p zircon_editor --lib component_showcase_template_materializes_deep_collection_and_menu_state --locked --jobs 1` was started with `CARGO_TARGET_DIR=/tmp/zircon-target-ui-showcase`, but exceeded the tool timeout while rebuilding dependency crates such as `wgpu_core`; the timed-out WSL Cargo/rustc processes from this pass were terminated. Other concurrent Windows editor tests were left untouched. Pending reruns remain:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_deep_collection_and_menu_state --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

Focused WSL commands used `CARGO_TARGET_DIR=/tmp/zircon-target-ui-showcase`:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_visual_feedback_and_vector_primitives --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_template_materializes_structured_context_menu_rows --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

Fresh focused Windows commands:

- `cargo test -p zircon_editor --lib asset_drag_payload --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1`
- `cargo test -p zircon_editor --lib tests::editor_event::runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1`
- `cargo test -p zircon_app --features target-editor-host editor_cli_operation --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --test-threads=1`

The SearchSelect query follow-up on 2026-04-28 adds `TemplatePaneNodeData.search_query`, projects the `SearchSelectDemo` `query = "number"` metadata through `pane_data_conversion/pane_component_projection/mod.rs`, defaults the field for legacy template-node conversion, and renders a retained SearchSelect primitive that shows the selected value and query text in `template_pane.slint`. The RED run failed at compile time on the expected missing `search_query` DTO field before implementation. Focused Windows validation then passed against `D:\cargo-targets\zircon-codex-editor-component-drawer-bindings`:

- `cargo test -p zircon_editor --lib component_showcase_template_materializes_search_select_query_state --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --nocapture`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --nocapture`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --nocapture`
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings`
- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-codex-editor-component-drawer-bindings -- --nocapture`

The SearchSelect retained-query follow-up on 2026-04-30 closes the live edit path for that projected query state. `SearchSelectDemo` now declares `UiComponentShowcase/SearchSelectQueryChanged` in `component_showcase.ui.toml` and the builtin binding table, `showcase_demo_state.rs` maps the `ValueChanged.SearchSelectQuery` route to the retained `query` property, and the generic host edit action exposes that binding while the existing option-selection route continues to mutate only the selected `value`.

Focused validation for this follow-up passed in `target\codex-runtime-ui-showcase-validation`: `cargo test -p zircon_editor --lib showcase_search_select_query_edit_is_retained_and_projected --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` ran 1 test / 0 failed, `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` ran 1 test / 0 failed, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` completed. An intermediate command using the stale `component_showcase_template_popup_options_dispatch_candidate_selection` filter matched 0 tests and is not acceptance evidence.

The editable-field commit projection follow-up on 2026-05-01 adds `TemplatePaneNodeData.commit_action_id` and projects the authored Submit bindings for `InputFieldDemo`, `TextFieldDemo`, `NumberFieldDemo`, and `RangeFieldDemo`. `pane_component_projection/showcase_actions.rs` selects the existing `*Committed` binding ids without changing `.ui.toml`; `template_node_conversion.rs` defaults the field to empty for legacy/non-component nodes. The retained reducer already applies `Commit.*` through `UiComponentEvent::Commit`, and `component_showcase_state.rs` now verifies committed text and numeric values mutate retained state, project back to host values, and append typed event-log rows. The first focused projection run failed at compile time on the expected missing `commit_action_id` DTO field before implementation.

Focused validation for this follow-up passed in `target\codex-runtime-ui-showcase-validation`: `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` ran 1 test / 0 failed, `cargo test -p zircon_editor --lib showcase_demo_state_applies_projected_bindings_to_retained_values_and_log --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` ran 1 test / 0 failed, `cargo test -p zircon_editor --lib showcase_edit_input_maps_collection_row_payloads_to_typed_events --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` ran 1 test / 0 failed, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` completed. Parallel final validation attempts and one cold sequential rerun timed out on package/artifact/build locks or dependency rebuilds before producing project diagnostics; the accepted evidence is from the warmed sequential reruns.

The structured option-row follow-up on 2026-04-28 upgrades generic selection popups from string-only candidate rows to retained `TemplatePaneOptionData` rows. `Dropdown`, `ComboBox`, `EnumField`, `FlagsField`, and `SearchSelect` still carry the legacy `options: [string]` fallback, but `pane_data_conversion/pane_component_projection/mod.rs` now also derives `structured_options` from authored `.ui.toml` `options`, current `value`, `disabled_options`, and `special_options`. The generic Slint popup prefers structured rows, renders selected/disabled/special state, dispatches stable option ids, and blocks disabled option dispatch. `DropdownDemo` now marks `runtime` as a special option and `debug` as disabled, while `FlagsFieldDemo` projects selected flags from its array value.

Validation note for this follow-up: `cargo fmt --package zircon_editor` passed, and `git diff --check` passed for the touched showcase/template/projection files with only existing LF-to-CRLF warnings. The RED Cargo run for `component_showcase_template_popup_options_dispatch_candidate_selection` could not reach the new Runtime UI assertions because the current workspace is blocked earlier by unrelated duplicate `SceneRendererAdvancedPluginOutputs` virtual-geometry accessor definitions in `zircon_runtime`. Pending reruns once that external compile blocker is cleared:

- `cargo test -p zircon_editor --lib component_showcase_template_popup_options_dispatch_candidate_selection --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase_pane_projects_runtime_component_nodes_for_template_pane --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The structured option-state follow-up on 2026-04-28 extends those option rows with retained `focused`, `hovered`, `pressed`, and `matched` state so selection popups can show keyboard focus, pointer hover, active press, and search-query match state through the same `.ui.toml -> projection -> Slint host` path as the rest of the component state chain. `DropdownDemo` now authors `focused_options`, `hovered_options`, and `pressed_options` for visible showcase coverage, while `SearchSelectDemo` derives `matched` from its retained `query = "number"` value. The generic popup applies row background and focus border styling from those projected fields while continuing to block disabled option dispatch. This pass also extracts option-row projection into `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_option_projection.rs`, leaving `pane_data_conversion/pane_component_projection/mod.rs` as the component orchestration boundary instead of accumulating more selection-specific parsing helpers.

Validation note for this follow-up: `cargo fmt --package zircon_editor` passed, `git diff --check` passed for tracked touched files with only existing LF-to-CRLF warnings, and a trailing-whitespace scan covered the new `pane_data_conversion/pane_option_projection.rs` file. A focused Cargo RED run could not reach the new assertions because Cargo was waiting on the shared artifact-directory lock; the broader editor/runtime Cargo validation remains pending behind the same external runtime compile/lock pressure described above. Pending reruns remain the same focused component-showcase commands plus `cargo check -p zircon_editor --lib`.

The structured menu-row follow-up on 2026-04-28 applies the same retained interaction-state model to `ContextActionMenu`. `TemplatePaneMenuItemData` now carries `focused`, `hovered`, and `pressed` flags in addition to raw id, stable action id, label, shortcut, checked, disabled, and separator state. The generic popup uses those fields for row background and focus border styling while preserving disabled/separator dispatch rejection, and it dispatches `menu_item.action_id` so encoded display metadata such as `Duplicate|hovered,pressed|Ctrl+D` never leaks into the event reducer. `ContextActionMenuDemo` now authors `Inspect|checked,focused|Ctrl+I` and `Duplicate|hovered,pressed|Ctrl+D`, which lets the showcase validate menu focus and active-row states without embedding business UI structure in `.slint`. Menu-row parsing moved into `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_menu_projection.rs`, keeping `pane_data_conversion/pane_component_projection/mod.rs` focused on orchestration.

Validation note for this follow-up: `cargo fmt --package zircon_editor` passed. `git diff --check` passed for the tracked touched docs, tests, and Slint template files with only existing LF-to-CRLF warnings, and a trailing-whitespace scan covered the untracked `pane_data_conversion/pane_menu_projection.rs` plus this session note. The first focused Cargo RED run and the later focused GREEN attempt for `component_showcase_template_materializes_structured_context_menu_rows` both timed out in the active local Cargo queue before producing a Rust/Slint diagnostic; no residual process from that specific focused test remained afterward, and unrelated concurrent validation processes were left untouched. Pending reruns remain the focused context-menu test, the pane projection test, broad `component_showcase`, and `cargo check -p zircon_editor --lib`.

Task 7 documentation/validation for the Runtime UI drag-source metadata slice ran fresh focused Windows commands from the repository root without a custom target directory:

- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1` passed: 16 tests passed, 0 failed.
- `cargo test -p zircon_editor --lib asset_browser_pointer_drop_applies_real_payload_to_showcase_asset_field --locked --jobs 1` passed: 1 test passed, 0 failed.
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1` passed: 20 tests passed, 0 failed.
- `cargo check -p zircon_editor --lib --locked --jobs 1` passed.

The broad `slint_host` validation was then unblocked. The shared projection mismatch in `apply_presentation_prefers_shared_visible_drawer_projection_when_legacy_geometry_is_zeroed` was traced to a stale expectation that omitted the scene viewport toolbar height even though the default Scene document pane intentionally enables toolbar chrome. The assertion now derives expected viewport content from the shared document frame plus document/header/toolbar chrome, matching the adjacent visible-drawer projection tests. Two compile drift fixes were also required while rerunning the broad filters: manual editor HUD `UiRenderCommand` constructors now set `text_layout: None`, and the Slint adapter forwards `HostMenuChromeData.menus` into generated `HostMenuChromeMenuData` / `HostMenuChromeItemData` models.

Fresh final Windows evidence after those fixes:

- `cargo test -p zircon_editor --lib slint_host --locked --jobs 1` passed: 114 tests passed, 0 failed.
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1` passed: 20 tests passed, 0 failed.
- `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1` passed: 16 tests passed, 0 failed.
- `cargo check -p zircon_editor --lib --locked --jobs 1` passed.

An intermediate runtime component-catalog rerun briefly saw unrelated `VirtualGeometryGpuReadback` accessor migration errors while concurrent graphics work was updating that support layer. A fresh rerun after the source settled passed the focused runtime component catalog. Workspace-wide validation was still not run because the checkout remains heavily dirty with active runtime graphics/plugin/editor work outside this Runtime UI drag-source slice.

The follow-up Runtime UI component catalog alignment pass on 2026-04-28 widened the descriptor schema for showcase-authored props and retained states: selection validation/popup metadata, SegmentControl selection state, Image/Icon value props, ToggleButton/Checkbox/Radio checked props, reference drop-hover and active-drag-target props/states, InspectorSection expanded props, ListRow selected/focused/hovered props/states, TreeRow depth/indent props, ContextActionMenu popup anchor/menu metadata, and MapField validation state. A stale duplicate `render_framework_rejects_quality_gated_unknown_executor_during_registration` test in `render_framework_bridge.rs` was removed so the focused runtime UI filter could compile. Fresh Windows checks passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_runtime/src/graphics/tests/render_framework_bridge.rs`
- `cargo test -p zircon_runtime ui::tests::component_catalog --lib` passed: 16 tests passed, 0 failed.

The next catalog guard on 2026-04-28 adds an editor-side regression test that parses `component_showcase.ui.toml` with `toml::Value`, walks every authored native node, and verifies every prop on a registered Runtime UI component exists in `UiComponentDescriptorRegistry::editor_showcase()`. That test caught the remaining dropdown option-state props (`disabled_options`, `special_options`, `focused_options`, `hovered_options`, and `pressed_options`), which are now declared by the shared selection descriptor helper and covered by the runtime catalog assertions.

Validation note for this guard: `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs` passed. The focused editor test compile was attempted with `--target-dir D:\cargo-targets\zircon-runtime-ui-catalog-props`, but the fresh editor test target exceeded the tool timeout while rebuilding dependencies. A runtime component-catalog rerun then reached unrelated active graphics test drift: `HybridGiScenePrepareResourcesSnapshot` fields had been made private while several graphics tests still used direct field access instead of the existing accessor methods.

The follow-up on 2026-04-28 cleared that graphics compile drift for the focused Runtime UI catalog path by moving the affected Hybrid GI tests to the public snapshot accessors and converting accessor slices into owned helper vectors only at the test-helper boundary. Fresh Windows checks passed:

- `rustfmt --check zircon_runtime/src/graphics/tests/hybrid_gi_resolve_surface_cache.rs zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs`
- `rustfmt --check zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs`
- `cargo test -p zircon_runtime ui::tests::component_catalog --lib` passed: 16 tests passed, 0 failed.

The focused editor guard was retried without the custom target directory through `cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog`, but local validation could not reach the test because concurrent Cargo work from other sessions held the shared package-cache lock. Those unrelated Cargo processes were left untouched. This pending state was superseded by the 2026-05-02 Milestone 0 editor closeout rerun in the isolated closeout target: `cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never` passed after the authored `VirtualList` / `PagedList` nodes were aligned to runtime catalog `items`.

The next Runtime UI catalog normalization follow-up on 2026-04-28 adds an internal descriptor hygiene guard so every editor-showcase component descriptor must use unique prop, state, slot, and event declarations; descriptor default values must match their declared `UiValueKind`; numeric ranges must not invert; and numeric steps must stay positive. This caught and removed the redundant `FlagsField.multiple` prop declaration, leaving the shared selection helper as the single owner of that schema entry. Fresh Windows checks passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`
- `cargo test -p zircon_runtime ui::tests::component_catalog --lib` passed: 17 tests passed, 0 failed.

The descriptor lookup follow-up on 2026-04-28 makes `UiComponentDescriptor` expose symmetrical lookup helpers for props, states, and slots. The catalog hygiene guard now verifies every declared prop, state, and slot is discoverable through those public descriptor APIs, and container rows such as `Group` and `PropertyRow` assert their authored slots through the same route. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/descriptor.rs zircon_runtime/src/ui/tests/component_catalog.rs`

The focused Runtime UI catalog test remains pending for this tiny API follow-up because other active Cargo sessions were still holding long-running editor/runtime/physics validation processes when this slice finished. No Cargo diagnostic was produced for this descriptor lookup change.

The exact-catalog follow-up on 2026-04-28 tightens the V1 registry guard so `UiComponentDescriptorRegistry::editor_showcase()` must expose exactly the current showcase component set rather than only proving that known component ids exist. This prevents accidental extra descriptor registration, silent duplicate-id overwrite, or V1 drift from hiding behind presence-only assertions. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind other active render/runtime/editor Cargo processes; this slice intentionally avoided starting another long Cargo compile while the shared queue was busy.

The registry size API follow-up on 2026-04-28 adds explicit `len()` and `is_empty()` helpers to `UiComponentDescriptorRegistry`, so exact-catalog tests and future host/catalog diagnostics do not need to infer registry size through iterator counting. The V1 exact-catalog guard now asserts the registry is non-empty and compares `registry.len()` to the authored V1 component list. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind active runtime/physics Cargo processes; no project diagnostic was produced for this API-only follow-up.

The registry id-set follow-up on 2026-04-28 adds `UiComponentDescriptorRegistry::component_ids()` and uses it in the exact-catalog guard to compare the actual registered component id set with the authored V1 catalog. This keeps diagnostics pointed at the concrete id drift instead of only detecting a size mismatch plus missing known ids. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this API-only follow-up.

The registry category API follow-up on 2026-04-28 adds `UiComponentDescriptorRegistry::descriptors_in_category()`. The V1 catalog guard now verifies the exact component-id set in each Runtime UI category, matching the Showcase navigation groups without requiring editor host code or tests to duplicate category ownership logic. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind active Cargo processes; no project diagnostic was produced for this API-only follow-up.

The registry containment API follow-up on 2026-04-28 adds `UiComponentDescriptorRegistry::contains()` so presence checks can use an explicit registry contract instead of spelling existence as `descriptor(id).is_some()`. The V1 presence guard now uses that public API while keeping descriptor retrieval for tests that need schema details. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this API-only follow-up.

The registry category-set API follow-up on 2026-04-28 adds `UiComponentDescriptorRegistry::categories()`, pairing the category-id iterator with `descriptors_in_category()`. The V1 catalog guard now verifies that the editor showcase registry exposes the complete category set before checking each category's exact component ids. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs zircon_runtime/src/ui/tests/component_catalog.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this API-only follow-up.

The registry API documentation follow-up on 2026-04-28 adds Rustdoc to the public `UiComponentDescriptorRegistry` construction, lookup, size, id iteration, category iteration, and category-filter helpers. The docs call out deterministic iteration order and the editor showcase catalog role, making these Runtime UI component-library APIs read as stable contracts rather than incidental test helpers. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/catalog.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this documentation-only follow-up.

The descriptor lookup documentation follow-up on 2026-04-28 adds Rustdoc to the public `UiComponentDescriptor` query helpers for prop schemas, retained-state schemas, slot schemas, supported event kinds, and accepted drag payload kinds. This keeps descriptor-level lookup documentation aligned with the documented registry lookup surface, while preserving the explicit `state_prop()` and `slot_schema()` API names used by the current runtime catalog tests. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/descriptor.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this documentation-only follow-up.

The descriptor builder documentation follow-up on 2026-04-28 adds Rustdoc to the public option, prop schema, slot schema, and component descriptor builder helpers. This documents stable ids, typed defaults, required slots/props, numeric range/step metadata, supported-event registration, and drag/drop policies at the declaration boundary that builds the Runtime UI component catalog. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime/src/ui/component/descriptor.rs`

Focused Cargo validation remains queued behind active editor/runtime Cargo processes; no project diagnostic was produced for this documentation-only follow-up.

The typed value documentation follow-up on 2026-04-28 adds Rustdoc to `UiValueKind`, `UiValue`, and the core value helpers that report concrete kinds, coerce numeric-like values for reducers, format host-facing display text, and convert TOML literals into Runtime UI values. This documents the typed prop/state/event payload boundary that component descriptors and retained state share. Fresh Windows formatting passed:

- `rustfmt --check zircon_runtime_interface/src/ui/component/value.rs`

Focused Cargo validation remains queued behind active editor/runtime/physics Cargo processes; no project diagnostic was produced for this documentation-only follow-up.

The component category documentation follow-up on 2026-04-28 adds Rustdoc to `UiComponentCategory` and each category variant, aligning the category enum with the documented registry `categories()` and `descriptors_in_category()` query APIs. The docs clarify which Runtime UI component families belong to visual, input, numeric, selection, reference, collection, container, and feedback buckets. Formatting and Cargo validation are pending because this pass intentionally avoided starting another command while other active Cargo work was still in flight.

The extracted real drag-source test module then reached a lower Runtime graphics compile boundary before editor linkage. The fix is intentionally support-only: the seed-backed executed-cluster helper exports now match the split pass module boundary, the test-only ordering re-export is gated behind `cfg(test)`, and the builtin render-feature dispatcher uses the public `graphics::feature::RenderFeatureCapabilityRequirement` path. `cargo fmt --package zircon_runtime --package zircon_editor` passed. The focused editor compile command `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-shared\editor --no-run drag_sources` still needs a quiet Cargo queue; the latest attempts were blocked by active shared package-cache/build-directory locks from unrelated sessions rather than a Runtime UI assertion.

The follow-up test-structure cleanup moves the contiguous `component_showcase_*` Slint host assertions out of `zircon_editor/src/ui/slint_host/ui/tests.rs` into `zircon_editor/src/ui/slint_host/ui/tests/component_showcase.rs`. This keeps the parent file as the shared host fixture/projection harness and leaves the showcase-specific template/projection checks in a focused module. The parent file drops from roughly 3460 lines to roughly 2665 lines, while the new showcase test module is roughly 797 lines. Formatting checks passed with `cargo fmt --package zircon_editor -- --check`, `cargo fmt --package zircon_runtime -- --check`, and direct `rustfmt --edition 2021 ... --check` on the split files; Cargo test execution remains behind the same active local Cargo queue.

The next module-slimming slice extracts the Runtime UI Showcase binding table out of `zircon_editor/src/ui/template_runtime/builtin/template_bindings.rs` into `zircon_editor/src/ui/template_runtime/builtin/showcase_template_bindings.rs`. `template_bindings.rs` now owns the general workbench/editor binding registry and extends it with the showcase-specific list, while the new module owns the `UiComponentShowcase/*` binding ids and custom demo action payloads. This drops the main binding file from roughly 952 lines to roughly 581 lines and gives future showcase controls a focused binding home instead of growing the mixed workbench binding table.

The Runtime catalog test file received the same boundary cleanup: `component_state_*` event/reducer tests now live in `zircon_runtime/src/ui/tests/component_catalog/component_state.rs`, while `component_catalog.rs` keeps descriptor presence, schema normalization, and shared assertion helpers. The parent catalog test drops from roughly 1235 lines to roughly 538 lines, and the new state-event module is roughly 699 lines. `cargo fmt --package zircon_runtime` was run after the extraction.

The MapField row-edit follow-up adds retained key editing alongside the existing value cell editing. `UiComponentEventKind::RenameMapKey` and `UiComponentEvent::RenameMapKey` are now part of the runtime component contract, and `MapField` descriptors advertise that event. `UiComponentState` applies it by moving the old map value to the replacement key, rejecting duplicate target keys and missing source keys without creating fallback entries. On the editor side, `TemplatePaneCollectionFieldData` now carries `key_edit_action_id`; Map rows project it from the same `UiComponentShowcase/MapFieldSetEntry` binding as value edits, while Array rows and empty rows leave it empty. The generic `template_pane.slint` renders a `collection-field-key` `LineEdit` for Map keys and dispatches `key:<row_id>=<new-key>` payloads, and `pane_surface_actions.rs` translates those payloads into `UiComponentShowcaseDemoEventInput::RenameMapEntry`. This keeps Map key edits on the retained `.ui.toml -> projection -> Slint host -> event queue -> UiComponentState` route instead of adding showcase-specific Slint behavior.

Current validation for the MapField key-edit slice is partially blocked by the active local Cargo queue. `cargo fmt --package zircon_runtime`, `cargo fmt --package zircon_editor`, `cargo fmt --package zircon_runtime -- --check`, and `cargo fmt --package zircon_editor -- --check` passed. A focused `cargo test -p zircon_runtime component_state_renames_map_keys_and_rejects_duplicate_targets --lib --locked --jobs 1` attempt timed out after four minutes while other unrelated Cargo/rustc processes were active; no Rust assertion or compiler diagnostic was produced for this slice. Focused editor/runtime reruns remain the next validation step once the shared Cargo pressure drains.

The collection row type-metadata follow-up extends that same path so Array/Map child rows carry editor-role hints instead of only raw display strings. `TemplatePaneCollectionFieldData` now includes `key_component_role`, `value_component_role`, and `value_checked`. `pane_data_conversion/pane_component_projection/mod.rs` derives those roles from authored `element_type` / `key_type` / `value_type` plus the current `UiValue`: booleans become `checkbox`, numeric values become `number-field`, asset and instance references become reference roles, vectors/colors keep their own roles, and `UiComponentRef` style element refs project as `reference-field`. The generic template uses the role metadata to render boolean child values as a retained checkbox-style editor that dispatches the toggled bool value through the same row payload path; other values remain editable through the retained `LineEdit` path. `pane_surface_actions.rs` now parses `true`/`false` collection edit payloads into `UiValue::Bool` before falling back to numeric or string values.

Validation for the type-metadata follow-up: formatting and static diff checks passed (`cargo fmt --package zircon_editor`, `cargo fmt --package zircon_runtime`, both `--check` variants, and targeted `git diff --check` with only LF/CRLF warnings). A focused editor test command for `component_showcase_template_materializes_collection_field_edit_rows` was attempted with the warmed editor target directory, but it timed out after four minutes while the local Cargo queue was still occupied by unrelated editor/runtime jobs and produced no project diagnostic.

The next collection-row operation follow-up adds row-specific action metadata for item removal and Array reordering. `TemplatePaneCollectionFieldData` now carries `remove_action_id`, `move_up_action_id`, `move_up_payload`, `move_down_action_id`, and `move_down_payload`. Array rows derive those payloads from their retained index (`array-0=1`, `array-2=1`, etc.) and Map rows expose remove payloads keyed by `map-<key>`. The generic template renders compact row action affordances and dispatches them through `node_edited`, so the same event parsing path can map them back into `RemoveElement`, `MoveElement`, or `RemoveMapEntry` demo inputs. This removes another showcase-only assumption: row operations now belong to the retained collection row projection instead of hardcoded component-level action chips.

Because collection row rendering was becoming its own responsibility, the Slint host row implementation was split out of `template_pane.slint` into `template_collection_field_row.slint`. `TemplatePane` now keeps only the collection row loop and forwards row edit callbacks, while the extracted row component owns key/value editors, bool toggles, and row-level remove/move affordances. This keeps the generic pane host closer to an orchestration boundary and gives future Array/Map validation UI a smaller component to extend.

The collection child-row validation follow-up makes Array/Map row metadata more than editable text. `pane_data_conversion/pane_component_projection/mod.rs` now derives per-row `validation_level` and `validation_message` from the declared `element_type`, `key_type`, and `value_type`: empty collections project a warning row, map keys are checked against required/non-string key constraints, numeric and boolean value rows reject incompatible `UiValue` kinds, reference-like rows warn when their value is empty, and vector/color rows validate their expected shape. `template_collection_field_row.slint` renders those retained validation messages directly in the row header, uses warning/error chrome from the row metadata, and hides editors on empty-state rows so an empty collection reads as an inspector state rather than an inert input field. The Runtime reducer now also writes collection operation errors into `UiValidationState` for missing array indices, duplicate map keys, and missing map keys, so failed row operations can surface through the same retained validation channel as numeric and reference-drop errors.

Validation for this follow-up: targeted Rust formatting passed for `pane_data_conversion/pane_component_projection/mod.rs`, `structure_component_tests.rs`, `ui/tests/component_showcase.rs`, `state.rs`, and `component_state.rs`; targeted `git diff --check` passed for the touched Runtime UI/editor Slint/doc files with only LF/CRLF warnings. Focused Cargo execution is still subject to the active local Cargo queue described above, so the next quiet-window rerun should cover the new private projection tests plus the existing Runtime component-state filter.

The asset reference-list drag-source follow-up extends the real drag source chain beyond the asset content list. `AssetReferenceListPointerBridge` now exposes press handling, `ProjectedReferencePanel` forwards left-button down/up pointer events, `PaneSurfaceHostContext` carries `asset_reference_pointer_event`, and `SlintEditorHost::asset_reference_pointer_event` resolves References / Used By rows into retained `UiDragPayloadKind::Asset` payloads. Payload metadata records the specific source surface (`browser.references`, `activity.used_by`, etc.), source control id, uuid, locator, display name, resource kind, and extension; non-project external references remain non-draggable. The showcase drop path can therefore consume assets dragged from Asset Browser reference panels with the same typed accept/reject semantics as content-list assets.

Validation for this reference-list drag-source follow-up: targeted `rustfmt --edition 2021 --check` passed for the touched editor Rust files, targeted `git diff --check` passed for the touched docs/Rust/Slint files with only LF/CRLF warnings, and a trailing-whitespace scan passed for the untracked drag-source helper/test and reference-panel Slint files. Focused Cargo execution was not started in this slice because several unrelated Cargo/rustc editor/runtime builds were already active in the shared workspace; the next quiet-window rerun should cover `asset_drag_payload_resolves_reference_panel_metadata` and `asset_reference_pointer_down_arms_active_asset_drag_payload`.

The transient control-state projection follow-up completes another part of the retained interaction chain for the showcase. `UiComponentShowcaseDemoState` now treats explicit component state as authoritative for transient flags, so `focused`, `dragging`, `hovered`, `pressed`, `drop_hovered`, and `active_drag_target` are projected as explicit `true` or `false` values once a real event has touched that control. This lets runtime events such as `Hover(false)`, `Press(false)`, `DropHover(false)`, and `ActiveDragTarget(false)` override authored showcase props in `component_showcase.ui.toml` instead of leaving stale visual state stuck on. The editor showcase state test now exercises both sides of that transition for `ListRowDemo` and `AssetFieldDemo`.

Validation for this transient-state follow-up: `rustfmt --edition 2021 --check zircon_editor/src/ui/template_runtime/showcase_demo_state.rs zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs` passed, and `git diff --check -- zircon_editor/src/ui/template_runtime/showcase_demo_state.rs zircon_editor/src/tests/host/template_runtime/component_showcase_state.rs` passed with only LF/CRLF warnings. Focused Cargo reruns remain pending because several unrelated Cargo/rustc jobs were already active; the next quiet-window checks should include `cargo test -p zircon_editor --lib showcase_demo_state_projects_collection_children_and_control_flags --locked --jobs 1` and the runtime component-state filter that covers transient flag reducers.

The ContextActionMenu trigger-position follow-up makes popup placement event-driven instead of only authored/static. `ContextActionMenu` now advertises retained `popup_anchor_x` and `popup_anchor_y` state plus the `OpenPopupAt` event in catalog coverage; `UiComponentState` stores the pointer anchor and opens the popup through the same reducer path as `OpenPopup`. The showcase binding table and `component_showcase.ui.toml` add `UiComponentShowcase/ContextActionMenuOpenAt`, `UiComponentShowcaseDemoState` accepts `OpenPopupAt { x, y }`, and `SlintUiHostNodeModel` exposes `has_popup_anchor`, `popup_anchor_x`, and `popup_anchor_y` for host tests and downstream projection. The generic Slint host now forwards right-button context requests from `TemplatePaneNode` with panel-space pointer coordinates through `PaneSurfaceHostContext.component_showcase_control_context_requested`, and `pane_surface_actions.rs` dispatches those coordinates into retained showcase state while keeping ordinary click activation on a deterministic demo anchor.

Validation for this trigger-position follow-up: targeted Rust formatting passed for the touched Runtime UI tests plus editor showcase/runtime host files with `rustfmt --edition 2021`, and targeted `git diff --check` passed for the touched Runtime/editor/doc files with only LF/CRLF warnings. A focused RED-style Cargo attempt for `cargo test -p zircon_runtime --lib component_state_opens_context_action_menu_at_pointer_anchor --locked --jobs 1` timed out in the active local Cargo queue before any Rust/Slint diagnostic, so focused Cargo reruns remain pending. The next quiet-window checks should cover `component_state_opens_context_action_menu_at_pointer_anchor`, `showcase_context_action_menu_opens_at_retained_pointer_anchor`, and the `pane_surface_actions` unit test that parses `"x,y"` context anchors.

The Inspector object drag-source follow-up adds the missing dedicated `UiDragPayloadKind::Object` source path for ObjectField. `SlintEditorHost` now keeps an `active_object_drag_payload` alongside the existing asset and scene-instance payloads, `ObjectFieldDropped` consumes object payloads before falling back to scene-instance or asset payloads, and `inspector.rs` can package the currently selected Inspector object as `object://scene/node/<id>` with `Scene Object: <name>` source metadata. Asset content/reference, Hierarchy, and Inspector pointer-down handlers also clear the other active reference payload slots before arming their own payload, so stale source state does not leak across real drag origins. The generic Inspector host forwards left-button down/up from the Transform header through `PaneSurfaceHostContext.inspector_reference_pointer_event`, so the flow stays retained and event-driven: Inspector pointer event -> typed drag payload -> showcase drop binding -> `UiComponentEvent::DropReference` -> retained ObjectField state and source summary projection.

Validation for this object drag-source follow-up: targeted `rustfmt --edition 2021 --check` passed for the touched editor Rust host/test files, targeted `git diff --check` passed for the touched editor Slint/Rust/doc files with only LF/CRLF warnings, and the untracked drag-source test module plus session note passed a trailing-whitespace scan. Focused Cargo reruns should include `object_field_drop_consumes_active_object_drag_payload`, `inspector_pointer_down_arms_active_object_payload_for_object_field_drop`, and `inspector_pointer_up_clears_active_object_payload` once the local Cargo queue is quiet.

The real drag-source rejection follow-up closes a host-side semantic gap in the reference drop path. `SlintEditorHost::demo_input_for_showcase_action` no longer falls back to the synthetic showcase payload when a real but incompatible drag payload is active. For `AssetFieldDropped`, the host prefers an active asset payload but will still deliver a scene-instance or object payload to Runtime if that is the real drag source, allowing `UiComponentState` and the `AssetField` drop policy to reject it and project `validation_level = error`. `InstanceFieldDropped` mirrors that behavior by preferring scene instances and forwarding active asset/object payloads for rejection. `ObjectFieldDropped` keeps the broad object/scene/asset acceptance path. After any real reference drop, the host clears all active reference payload slots so a finished drag cannot leak into the next retained event.

Validation for this rejection follow-up: `rustfmt --edition 2021` was run on `pane_surface_actions.rs` and `drag_sources.rs`. A focused RED-style Cargo attempt for `cargo test -p zircon_editor --lib asset_field_drop_rejects_active_scene_instance_payload --locked --jobs 1` timed out after three minutes behind unrelated active Cargo/rustc processes and produced no compiler or assertion diagnostic. The next quiet-window checks should rerun both new rejection tests plus the existing accepted real-payload drag-source tests.

The follow-up module-slimming pass extracts the active reference drop payload priority logic from `pane_surface_actions.rs` into `reference_drop_payload.rs`. The extracted module owns only three responsibilities: choose the target-preferred active payload kind for AssetField, InstanceField, or ObjectField drops; consume the matching active slot; and clear all active reference payload slots after a real drop. This keeps `pane_surface_actions.rs` focused on binding/action dispatch and leaves future source-specific drag semantics in a smaller reference-drop module.

The next module-slimming pass extracts deterministic showcase demo input construction into `showcase_event_inputs.rs`. That module now owns the action-id-to-`UiComponentShowcaseDemoEventInput` mapping plus live edit payload parsing for numeric fields, collection row edits, map-key renames, context popup anchors, and deterministic fallback reference drops. The existing collection/context payload parser test moved with the code, while `pane_surface_actions.rs` keeps the runtime binding lookup, dispatch, and plugin-action handling.

The following module-slimming pass extracts module plugin action handling into `module_plugin_actions.rs`. The new module owns `ModulePluginAction` parsing, enable/disable dispatch, packaging policy cycling, target-mode cycling, project manifest persistence, status labels, and the focused unit tests for plugin action parsing and deterministic policy cycles. `pane_surface_actions.rs` now delegates `ModulePluginAction` clicks through the host method and otherwise stays focused on pane/control dispatch plus Runtime UI showcase binding application.

The Slint host component-projection slimming pass splits `pane_data_conversion/pane_component_projection/mod.rs` into a small orchestration file and three focused child helpers. `collection_fields.rs` now owns Array/Map child-row projection, role inference, empty-state rows, and per-row validation tests; `showcase_actions.rs` owns the deterministic binding/action preference table and secondary action-chip projection; `preview_images.rs` owns Image/Icon/SvgIcon preview candidate resolution and Slint image loading. This leaves `pane_data_conversion/pane_component_projection/mod.rs` focused on converting a retained Runtime UI node into `TemplatePaneNodeData` while keeping collection semantics, action routing, and media preview lookup independently extensible.

The pane data conversion slimming pass extracts UI Asset Editor presentation DTO mapping into `pane_data_conversion/pane_ui_asset_conversion.rs`. The extracted module owns asset editor string selections, preview canvas node conversion, slot-target conversion, and the full `UiAssetEditorPanePresentation -> UiAssetEditorPaneData` mapping. `pane_data_conversion/mod.rs` keeps the retained pane selection, legacy template node conversion, builtin Runtime UI projection, and component showcase projection entry points, so Widget Editor payload conversion no longer lengthens the generic Runtime UI pane conversion surface.

The legacy template-node conversion pass extracts shared fallback `ViewTemplateNodeData -> TemplatePaneNodeData` defaults into `template_node_conversion.rs`. Both `apply_presentation.rs` and `pane_data_conversion/mod.rs` now use that helper for non-Runtime retained nodes, which keeps fields such as `focused`, `hovered`, `pressed`, `drop_hovered`, `active_drag_target`, popup anchor defaults, collection defaults, menu defaults, and media preview defaults in one location instead of duplicating them across host projection entry points.

The 2026-04-29 schema/categorization pass makes the Runtime UI descriptor registry active during `.ui.toml` compilation and makes showcase category selection affect actual host projection. `UiDocumentCompiler::default()` now owns the editor-showcase component registry, applies schema/default handling through `component_props.rs`, and rejects registered component props whose authored TOML values cannot satisfy the declared `UiValueKind` or numeric range. The compiler module inventory guard now includes `component_props.rs`, so the component-schema compiler stage is tracked with the rest of the asset compiler pipeline. The showcase retained state now starts in an `All` projection state for existing broad projection coverage, marks the selected nav button after category actions, and filters demo controls to Visual/Feedback, Input/Numeric/Selection/Reference, or Structure/Collection groups. The retained state/control map also covers ColorField, Vector2/3/4Field, PropertyRow, and InspectorSection demo controls, so the category-filtered projection keeps all descriptor-backed V1 demo rows aligned with the runtime component catalog. Regression tests were added in `zircon_runtime/src/ui/tests/asset.rs` and `zircon_editor/src/tests/host/template_runtime/component_showcase_category.rs`; initial Cargo validation was deferred until the milestone completion gate because the active user instruction forbade testing before the milestone was complete. The completion validation below covers those deferred checks.

- `cargo test -p zircon_runtime --lib ui_asset_compiler --locked --jobs 1`
- `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1`
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`
- `cargo check -p zircon_editor --lib --locked --jobs 1`

The follow-up schema/state-panel pass on 2026-04-29 closes retained-state gaps that remained after category projection. `UiComponentState::apply_value` now validates every registered component prop kind, not only numeric props: ColorField, Vector2/3/4Field, reference, enum, flags, array, and map values reject incompatible `UiValueKind` payloads with `UiComponentEventError::InvalidValueKind`, leave the previous retained value intact, and project validation error state for the control. Map removal now rejects missing keys through the same `MissingMapKey` validation path as Set/Rename. The editor showcase asset now declares explicit `PropertyRow` diagnostics for selected category, last control, last action, current value, validation, and drag/drop payload summary; `showcase_demo_state/categories.rs` owns the category/nav mapping, while `showcase_demo_state/state_panel.rs` owns those diagnostic row projections. Failed retained events are logged before returning the error, so the right state panel selects the failed control and shows its validation instead of continuing to summarize the previous successful event. InstanceField and ObjectField also now match AssetField's secondary reference actions through authored and builtin Locate/Open/Clear bindings plus generic action-chip projection.

This pass also keeps touched files split by responsibility instead of stacking more logic into already-large modules. The showcase state reducer delegates category filtering to `categories.rs` and diagnostic row projection to `state_panel.rs`; the runtime component-state value-kind coverage lives in `component_state/value_validation.rs` instead of pushing the parent test module over the guardrail. The category-filter regression now lives in `component_showcase_category.rs`, leaving `component_showcase_state.rs` focused on retained reducer state and keeping it below the large-file warning threshold after the cleanup split.

Completion-gate validation for this follow-up now has fresh evidence. `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1` passed with 22 tests after one initial compile-only timeout; `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1` then passed with 124 tests, including the collection validation, component catalog, template, asset compiler, boundary, and shared UI runtime coverage. `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1` passed with 23 tests, covering the retained showcase state, category filtering, state panel, action chips, popup options, collection/menu rows, reference wells, and generic Slint host projection. Earlier focused editor single-test reruns hit dependency-compile timeouts without project diagnostics, but the broader `component_showcase` filter contains and passed those focused assertions. A later cleanup gated test-only SDF atlas helpers behind `cfg(test)`, removed the unused editor meta save path, and marked test/host-only showcase input variants as intentionally retained, so the final editor `cargo check` rerun no longer reports those dead-code warnings.

Completion-gate validation for the schema/category/state-panel slice was run after the milestone implementation pass. `cargo test -p zircon_runtime --lib ui_asset_compiler --locked --jobs 1` passed with 4 tests after one initial compile-only timeout. `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1` passed with 124 tests. The first editor rerun in the default target directory failed before compilation with a Cargo dep-info write error because `target\debug\.fingerprint` was absent while Cargo was writing `zircon_editor` artifacts; the rerun used the explicit isolated target `target\codex-runtime-ui-showcase-validation` for this validation exception. `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation` then passed with 23 tests after rebuilding the target, including `showcase_category_selection_filters_projected_demo_controls` from the extracted category test module. `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation` passed without dead-code warnings. Final formatting and whitespace checks for the cleanup are recorded in the closeout report rather than duplicated here.

The Dropdown authored-selection metadata follow-up closes the remaining V1 selection semantics gap in the focused Runtime UI showcase scope. `UiComponentState` now treats retained `disabled_options` values as selection policy, not only descriptor-baked disabled option descriptors, so `.ui.toml` auth can reject disabled ids while preserving the previous retained value. `showcase_demo_state/defaults.rs` now owns the control-id map and default retained values for the showcase; `DropdownDemo` seeds its authored `multiple = true` and `disabled_options = ["debug"]` semantics into retained state before the first event, so selecting `editor` produces a multi-value selection alongside `runtime`, and selecting `debug` projects an error validation message instead of mutating the value. Regression coverage lives in `component_state/selection.rs` for the lower reducer behavior and `component_showcase_selection.rs` for the editor retained projection path.

Focused TDD evidence for this follow-up: `cargo test -p zircon_runtime --lib component_state_rejects_disabled_option_ids_from_retained_metadata --locked --jobs 1` first failed because selecting retained disabled option `debug` returned `Ok(())`, then passed after the reducer used retained `disabled_options`. `cargo test -p zircon_runtime --lib component_state_applies_dropdown_multiple_selection_and_special_options --locked --jobs 1` passed after moving selection coverage into `component_state/selection.rs`. The first editor RED attempt timed out during dependency compilation before a Rust diagnostic; after the lower reducer fix and current editor source settled, `cargo test -p zircon_editor --lib showcase_dropdown_uses_authored_multiple_and_disabled_option_metadata --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation` passed and verified multi-value projection plus rejected disabled-option validation. Broader focused validation then passed with `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1` at 126 tests, `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation` at 24 tests, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation`. The editor validation emitted a pre-existing runtime graphics dead-code warning in `virtual_geometry_node_and_cluster_cull_pass/output.rs`, which belongs to the active render-feature/plugin session rather than this Runtime UI slice.

The 2026-04-30 follow-up keeps `zircon_editor/ui/**/*.slint` absent and treats any deleted Slint copy as non-authoritative. The root build/include seam is Rust-owned through `host_contract/**`, integration-contract source readers assert `.ui.toml` assets and Rust host contracts directly, and scoped Cargo evidence is recorded above. Workspace-wide acceptance is still intentionally not claimed.

The 2026-05-01 validation closeout pass rechecked the Runtime UI layer before moving into real-window and Inspector data-source work. `cargo test -p zircon_runtime --lib ui::tests --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` did not reach Runtime UI assertions because the `zircon_runtime` lib-test build stopped in unrelated graphics/test drift: missing hybrid-GI include files under `graphics/tests/boundary.rs`, a stale `HybridGiScenePrepareResourcesSnapshot` import in `graphics/scene/mod.rs`, an unresolved test-only `fontsdf` path in `graphics/scene/scene_renderer/ui/sdf_font_bake.rs`, and virtual-geometry readback API name drift in graphics tests. The follow-up normal library gate, `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never`, passed with graphics/runtime warnings only. Current conclusion: the Runtime UI normal library surface type-checks, while the broad Runtime UI lib-test filter is externally blocked by active graphics lib-test configuration/API drift rather than a Runtime UI reducer, schema, or showcase assertion failure.

The same closeout pass then reran the editor Runtime UI host gates in the isolated target directory. `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` passed with 17 component-showcase tests, covering Rust-owned template metadata, option/action callbacks, reference wells, structure/collection projection, category filtering, retained selection/query state, context menus, full component action bindings, and retained demo-state projection. `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` also passed. The editor run emitted four local warnings for unused Inspector component-adapter plumbing (`component_adapter/inspector.rs`, `component_adapter/registry.rs`, and the retained `apply_binding` helper), which are now treated as the next implementation seam for real Inspector data binding rather than as showcase failures.

The first real Inspector binding slice connects that seam without removing the existing editor event journal path. `EditorEventRuntime::dispatch_ui_component_adapter_event` now routes `UiComponentEventEnvelope` values through `EditorUiComponentAdapterRegistry`, refreshes editor reflection after changed component adapter results, and surfaces adapter status text through the existing status line. The Inspector adapter supports `ValueChanged` and `Commit` envelopes for `name`, `parent`, and `transform.translation.{x,y,z}` targets under the `inspector` domain, mapping Runtime UI `UiValue` payloads into the existing draft/apply binding dispatch so subject resolution, selected-node synchronization, and scene mutation policy remain shared with the legacy Inspector commands. The Slint host Inspector change callback now creates a real Runtime UI component envelope before dispatching the old `DraftCommand.SetInspectorField` route, so actual window callbacks exercise the new data-binding adapter while preserving focus handling, journal records, and presentation effects. Focused evidence: `cargo test -p zircon_editor --lib child_window_inspector_control_focuses_source_window_before_runtime_dispatch --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` passed, `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` passed, and the post-slice `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` passed with 17 tests. The remaining warnings are the pre-existing runtime graphics warnings recorded above.

The follow-up merge pass aligned the Inspector adapter implementation with the parallel real-data-source session interface: the editor runtime entry point is `dispatch_ui_component_adapter_event`, and `UiComponentAdapterError::MissingSource` uses `source_name` so `thiserror` does not interpret the field as an error source. The Inspector adapter now returns structured `UiComponentAdapterError` values directly, rejects unsupported subjects/properties/fields without mutation, and the migrated Inspector change callback uses the adapter as the authoritative path for supported fields instead of falling back to legacy draft dispatch. Dedicated adapter regressions live in `zircon_editor/src/tests/ui/component_adapter.rs`; the wider confirmation gate `cargo test -p zircon_editor --lib inspector --locked --jobs 1 --target-dir target\codex-runtime-ui-showcase-validation --message-format short --color never` passed with 57 tests. During that run the Inspector pane template projection also regained action-control text fallback for empty authored text/label values, so `ApplyDraft` projects as `Apply Draft` through the Runtime UI host-node path.

The 2026-05-01 real-data-source adapter milestone hardens the shared Runtime UI envelope contract and the editor adapter bridge. `zircon_runtime::ui::component::data_binding` now owns the typed `UiComponentBindingTarget`, `UiComponentEventEnvelope`, `UiComponentProjectionPatch`, `UiComponentAdapterResult`, and `UiComponentAdapterError` surface, and serde rejects mismatched wire event kinds before they can reach runtime dispatch. `zircon_editor::ui::template_runtime::component_adapter` now routes showcase envelopes through a registry-backed adapter path and resolves the `inspector` domain into the existing inspector draft mutation flow for `entity://selected`. Focused validation passed for the new slice with `cargo test -p zircon_editor --lib inspector_component_adapter --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`, `cargo test -p zircon_editor --lib inspector --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-runtime-ui-real-data-source-adapter --message-format short --color never`. The matching runtime-only focused test remains blocked by unrelated active graphics/plugin cutover paths and `--locked` lockfile drift, so the accepted evidence for this milestone is the editor adapter bridge plus the shared contract tests rather than a clean workspace-wide pass.

The 2026-05-01 component-library continuation extends the Runtime UI contract beyond Inspector binding into the next retained data layers. The component catalog now advertises `VirtualList`, `PagedList`, and `WorldSpaceSurface`: virtual lists carry retained `items`, total count, viewport start/count, item extent, and overscan metadata; paged lists carry retained page index/size/count and total count; world-space surfaces carry retained world transform, size, pixels-per-meter, billboard/depth policy, render order, and camera target metadata. `TemplatePaneNodeData` and the Slint host component projection now preserve those fields so the generic host contract can walk real complex-list, pagination, and world-space UI nodes without inventing new side channels later.

The same pass adds the first shared data-source descriptor layer for property reflection and asset-editor binding. `zircon_runtime::ui::component::data_binding` now exports `UiComponentDataSourceDescriptor` and `UiComponentDataSourceKind`, alongside new `UiComponentBindingTarget::reflection` and `UiComponentBindingTarget::asset_editor` constructors. The editor component adapter registry advertises the selected-entity Inspector source, selected component/asset reflection sources, and UI Asset Editor widget/layout/slot/binding/style sources with stable `source_name`, subject, path-prefix, writability, and value-kind metadata. This keeps future Inspector, asset editor, and reflection binding work on one typed source contract rather than parallel hard-coded strings.

Focused validation for this continuation passed in `target\\codex-runtime-ui-showcase-validation`: `cargo test -p zircon_runtime --lib runtime_component_catalog_contains_showcase_v1_controls --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never`; `cargo test -p zircon_editor --lib runtime_component_projection_preserves_virtualization_and_pagination_metadata --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never`; `cargo test -p zircon_editor --lib runtime_component_projection_preserves_world_space_metadata --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never`; `cargo test -p zircon_runtime --lib component_data_source_descriptor_names_required_sources_without_editor_types --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never`; and `cargo test -p zircon_editor --lib editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never`. The user-requested Inspector confirmation gate was also rerun after the adapter interface merge: `cargo test -p zircon_editor --lib inspector --locked --jobs 1 --target-dir target\\codex-runtime-ui-showcase-validation --message-format short --color never` passed with 57 tests. All runs emitted only the pre-existing runtime graphics warnings already attributed to active graphics sessions.

The next 2026-05-01 continuation connects two of the retained component-library surfaces to real host behavior. The `asset_editor` component adapter domain now routes `ValueChanged` / `Commit` envelopes through `dispatch_ui_component_adapter_event` into the existing `EditorManager` UI Asset Editor mutation APIs. The target `subject` source carries the live `ViewInstanceId`, and supported paths include widget text/control id, slot mount/padding/preferred size, layout preferred size, and selected slot/layout semantic values. This keeps asset-editor field edits on the shared Runtime UI envelope contract while avoiding direct edits inside the active `asset_editor_sessions` implementation owned by another session.

The same continuation makes `VirtualList` host projection window-aware. Retained `collection_items` are sliced by `viewport_start`, `viewport_count`, and `overscan` before becoming the Slint host model, so large-data list projection can feed only the visible window plus guard rows instead of materializing the whole collection into the UI surface. World-space UI remains represented by the host contract metadata added in the previous pass; real RHI/world rendering hookup is still intentionally deferred to the graphics/RHI owner sessions to avoid cross-session collisions.

The complex-components follow-up moves those surfaces from metadata-only descriptors toward retained Runtime UI component behavior. `VirtualList` now advertises and applies `SetVisibleRange`, normalizing negative starts/counts and clamping the visible window against retained `total_count` while keeping `items` separate for lazy data. `PagedList` advertises and applies `SetPage`, clamping page size, deriving page count from retained totals, and clamping page index to the available page range. `WorldSpaceSurface` advertises and applies transform and surface metadata events, stores Vec3 transform values, Vec2 surface size, clamped `pixels_per_meter`, render order, billboard/depth flags, and camera target, and rejects non-positive scale or surface size before mutation. The editor projection remains a generic consumer: its visible collection window clamps negative starts and overscan edges deterministically, but graphics/RHI world rendering remains out of scope.

Focused validation for the complex-components follow-up passed on 2026-05-01. Targeted `rustfmt --edition 2021 --check` passed for `zircon_runtime/src/ui/component/event.rs`, `zircon_runtime/src/ui/component/state.rs`, `zircon_runtime/src/ui/component/catalog/editor_showcase.rs`, `zircon_runtime/src/ui/tests/component_catalog.rs`, `zircon_runtime/src/ui/tests/component_catalog/complex_components.rs`, and `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs`. `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir target\codex-runtime-ui-complex-components --message-format short --color never` passed with 37 tests. The E: target drive was below the repo free-space threshold, so the remaining editor/type-check gates switched to `D:\cargo-targets\zircon-runtime-ui-complex-components`; `cargo test -p zircon_editor --lib runtime_component_projection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never` passed with 4 tests after one earlier E: dependency-compile timeout. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never` and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never` both completed. Remaining warnings are outside the complex-component reducer/projection slice: runtime graphics dead-code/unused warnings, `UiComponentApiVersionParseError` import drift in the UI asset component contract, editor component-adapter descriptor helpers that are currently unused outside tests, and an unrelated `world_space_submission::*` host-contract export warning in the editor test build. `git diff --check` reported only LF-to-CRLF notices for tracked touched files, and a targeted trailing-whitespace scan over the untracked complex-component spec/plan and split test/catalog files produced no matches.

The world-space host-to-render handoff now has a Rust-owned submission-list boundary without touching RHI. `zircon_editor::ui::slint_host::host_contract::data::world_space_submission` converts projected `TemplatePaneNodeData` values into `WorldSpaceUiSurfaceSubmission` records by filtering `world_space_enabled` nodes, resolving world size from explicit `world_width/world_height` or `frame / pixels_per_meter`, copying world transform/billboard/depth/order/camera metadata, and sorting by render order plus stable ids. This gives the render integration work a plain host-side list to consume instead of reaching back into Slint models or Runtime UI template nodes.

The catalog/category stabilization follow-up removes the fragile self-reexport dependency inside the Runtime UI component catalog and descriptor validator. Catalog internals now import descriptor-owned types directly from the sibling `descriptor` module instead of reaching through the parent `crate::ui::component` public re-export surface, and descriptor validation uses its local `UiHostCapability` import for the virtualized-layout capability check. This keeps `Numeric` / `Selection` descriptor registration and category enumeration isolated from transient root re-export ordering drift while preserving the public `zircon_runtime::ui::component::*` API.

The 2026-05-01 category-helper regression fix keeps the V1 category authority inside the descriptor constructors. The `numeric()` helper now starts from `UiComponentCategory::Numeric` and reapplies the pointer, keyboard, text-input, and text-rendering capabilities that made it input-like; the `selection()` helper now starts from `UiComponentCategory::Selection` and reapplies pointer and keyboard capabilities before adding option/value schemas. The previous helper delegation through `input()` preserved behavior but registered Number/Range/Color/Vector and Dropdown/Combo/Enum/Flags/SearchSelect descriptors as `Input`, which made `UiComponentDescriptorRegistry::categories()` omit `Numeric` and `Selection`. Fresh validation on the shared target passed: `cargo test -p zircon_runtime --lib runtime_component_catalog_contains_showcase_v1_controls --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never` passed 1 test, and `cargo test -p zircon_runtime --lib ui::tests::component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-srp-rhi-main-chain --message-format short --color never` passed 37 tests after the disk-policy target cleanup.

The 2026-05-01 closeout pass resolves the highest-priority Runtime UI component-library blockers and records fresh focused evidence. Runtime catalog internals no longer depend on root `component` re-exports for descriptor-owned types, and descriptor validation no longer self-imports `UiHostCapability` through the public component facade. The focused category guard now passes again, confirming the editor showcase registry exposes the full V1 category set including `Numeric` and `Selection`.

UI Asset Editor field mutations now use the real component adapter route from the actual Slint host detail callback. Widget, slot, layout, and slot/layout semantic value/field set actions create `UiComponentEventEnvelope` values targeting the `asset_editor` domain and dispatch through `dispatch_ui_component_adapter_event`; delete actions remain direct manager commands because they are not value commits. The asset-editor adapter also accepts dynamic `slot.semantic.field.*` and `layout.semantic.field.*` target paths, so structured semantic field edits share the same adapter seam.

The generic property data-source contract now includes `UiComponentDataSourceFieldDescriptor` and `UiComponentDataSourceFieldOption`, covering field path, display name, value kind, writability, group, collapsed state, numeric range/step, options, array element kind, map key/value kinds, reference kind, and validation state/message. This gives Reflection and future property/asset editors a typed field-schema boundary instead of only source-level descriptors.

Fresh focused validation passed in `E:\cargo-targets\zircon-runtime-ui-closeout`: `cargo test -p zircon_runtime --lib runtime_component_catalog_contains_showcase_v1_controls --locked --jobs 1`; `cargo test -p zircon_runtime --lib complex_components --locked --jobs 1`; `cargo test -p zircon_runtime --lib component_data_source_field_descriptor_covers_property_editor_metadata --locked --jobs 1`; `cargo check -p zircon_editor --lib --locked --jobs 1`; `cargo test -p zircon_editor --lib asset_editor_component_adapter_updates_selected_widget_text --locked --jobs 1`; `cargo test -p zircon_editor --lib world_space_ui_surface_submissions --locked --jobs 1`; `cargo test -p zircon_editor --lib runtime_component_projection_slices_virtualized_visible_collection_items --locked --jobs 1`; `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1`; and `cargo test -p zircon_editor --lib inspector --locked --jobs 1`. The runs emit existing runtime graphics dead-code/unused warnings outside this Runtime UI slice.

The remaining closeout pass adds the missing non-RHI boundaries for mutation transactions and world-space interaction. `UiComponentAdapterResult` now carries `dirty`, `transaction_id`, and `mutation_source` metadata, giving Inspector, UI Asset Editor, and future reflection adapters a common transaction/dirty-reporting envelope for undo/redo and save integration. The UI Asset Editor adapter populates that metadata for selected widget/slot/layout mutations.

World-space UI now has a viewport-side handoff cache and a host-side hit-test boundary without rendering through RHI. `WorldSpaceUiSurfaceSubmission` exposes viewport hit bounds, and `SlintViewportController` can cache the latest world-space UI submission list separately from the ordinary shared 2D UI overlay. This makes the next graphics/RHI integration a consumer of prepared host data rather than a crawler of Slint or Runtime UI state.

Fresh focused validation passed in `E:\cargo-targets\zircon-runtime-ui-closeout`: `cargo test -p zircon_runtime --lib component_projection_patch_keeps_attribute_and_state_values_separate --locked --jobs 1`; `cargo test -p zircon_editor --lib world_space_ui_surface_submissions --locked --jobs 1`; and `cargo test -p zircon_editor --lib controller_caches_world_space_ui_submission_list_without_rendering_it --locked --jobs 1`. These supplement the earlier closeout evidence for catalog/category, complex components, data-source fields, Asset Editor adapter, component_showcase, inspector, and editor check.

## 2026-05-01 remaining-gap closeout: data-source fields and large-window semantics

This closeout extends the Runtime UI component contract in three areas that were still blocking real inspector/asset-editor convergence.

- Data-source descriptors now carry resolved field schema entries. `UiComponentDataSourceDescriptor` can include `fields`, and the inspector selected-entity source publishes writable `name`, `parent`, and transform translation fields with grouping, range, step, and reference metadata. Editor-side adapter registry sources now expose component reflection, asset reflection, and UI asset editor field schemas for widget, slot, layout, binding, and style domains.
- `VirtualList` now records a retained large-data window instead of only the visible start/count. `SetVisibleRange` writes `visible_end`, overscan-aware `requested_start`/`requested_count`, normalized `overscan`, and `scroll_offset`, giving host code enough information to request lazy data slices without depending on an immediate full list.
- `PagedList` now records `page_start`, `page_end`, `empty`, and an overflow-safe `page_count`. The page-count formula avoids `total_count + page_size - 1` overflow for very large data sources.
- Component showcase demo state now understands complex component events for `SetVisibleRange`, `SetPage`, `SetWorldTransform`, and `SetWorldSurface`, and projects the resulting runtime state metadata back into host node attributes. This keeps complex controls on the same Runtime UI event envelope path as simpler inputs.
- Inspector adapter mutation results now include transaction/source metadata, aligning inspector mutations with the asset editor adapter result contract.
- A lower-level UI asset component-contract regression was corrected while validating this work: selector target collection now returns the populated `SelectorTargetSet` instead of an empty result.

Validation evidence:

- `cargo test -p zircon_runtime --lib complex_components --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed, 6 tests.
- `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-ui-complex-components --message-format short --color never`: passed, 39 tests after adding the paged-list overflow regression.
- `cargo test -p zircon_runtime --lib data_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed, 6 tests.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed. Remaining warnings are pre-existing graphics/runtime dead-code warnings plus the non-RHI world-space viewport cache entry point that is intentionally not consumed until the rendering/RHI handoff.

Remaining boundary:

- Actual RHI consumption of world-space UI submissions remains deliberately out of scope for this UI component-library pass.

## 2026-05-01 follow-up closeout: complex Showcase nodes and adapter schema regression locks

The previous closeout added the runtime and adapter-side semantics for complex component events and resolved data-source fields. This follow-up wires those semantics into the actual Component Showcase asset and locks the editor adapter surface with focused tests.

- `component_showcase.ui.toml` now includes real `VirtualList`, `PagedList`, and `WorldSpaceSurface` showcase nodes in the Collections section. Each node has a concrete binding route into the Runtime UI event path: `SetVisibleRange.VirtualList`, `SetPage.PagedList`, `SetWorldTransform.WorldSpaceSurface`, and `SetWorldSurface.WorldSpaceSurface`.
- The showcase retained-state tests now dispatch those complex bindings and assert the resulting runtime state, including virtual-list requested ranges and scroll offset, paged-list page windows, and world-space transform/surface metadata.
- Editor component-adapter tests now assert that inspector mutation results carry dirty, transaction, and mutation-source metadata, and that the adapter registry publishes field schema for inspector, reflection, and asset-editor sources.
- The non-RHI world-space viewport cache is intentionally annotated as a pending render-consumer boundary so editor lib checks do not report it as accidental dead code before the graphics/RHI handoff lands.

Validation evidence:

- `cargo test -p zircon_editor --lib showcase_demo_state_applies_complex_component_runtime_events --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed, 1 test.
- `cargo test -p zircon_editor --lib component_adapter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed, 8 tests.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never`: passed. Editor introduced no new warnings; remaining warnings are existing runtime graphics warnings outside this UI component-library scope.

## 2026-05-01 world-space UI render/RHI consumption closeout

- World-space UI submissions now leave the host-only cache and are folded into the viewport UI render extract during `SlintViewportController::submit_extract_with_ui`.
- The Slint host recompute path collects world-space-enabled pane/template nodes from the current host scene presentation and submits them to the viewport before the next render frame submission.
- The render-side consumption path intentionally reuses `UiRenderExtract -> RenderFramework::submit_frame_extract_with_ui -> ViewportRenderFrame::with_ui -> ScreenSpaceUiRenderer`, so world-space UI reaches the existing wgpu/RHI UI pass without binding editor component logic directly to low-level RHI code.
- Current implementation renders projected world-space surfaces as screen-space UI quads with label text, depth/billboard metadata reflected in styling and ordering. A future true 3D/depth-aware UI geometry pass can replace the projection consumer behind the same submission contract.
- Validation: `cargo test -p zircon_editor --lib controller_submits_world_space_ui_surfaces_through_render_framework_ui_extract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --color never` passed with 1 test, 0 failed; only pre-existing runtime graphics warnings were emitted.

## 2026-05-02 authored showcase schema closeout

The Milestone 0 editor recovery found one authored asset drift in the Component Showcase: the `VirtualListDemo` and `PagedListDemo` nodes had authored `collection_items`, but the runtime component catalog declares retained list data as `items`. The asset now authors `items`; `collection_items` remains generated host projection state produced by `showcase_demo_state` and pane projection after visible-window slicing.

Validation evidence:

- `cargo test -p zircon_editor --lib component_showcase_authored_props_are_declared_by_runtime_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never`: passed.
- `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-validation-closeout --message-format short --color never`: passed as part of the editor closeout package gate, `876 passed; 0 failed; 1 ignored`.

## 2026-05-02 UI Asset component-contract field closeout

The UI Asset Editor adapter schema now includes the selected component root-class policy as a writable field on the asset-editor source. `EditorUiComponentAdapterRegistry` publishes `component.root_class_policy` with enum options `append_only` and `closed`, and `asset_editor::apply_asset_editor_component_envelope` routes commits for that path to the existing editor manager/session mutation API. This keeps the runtime `UiRootClassPolicy` contract as the source of truth while allowing component-adapter property editors to author the value through the same `UiComponentEventEnvelope` path as widget, slot, layout, and semantic fields.

Focused validation evidence from `E:\cargo-targets\zircon-ui-m10-root-class-authoring`:

- `cargo test -p zircon_editor --lib asset_editor_component_adapter_updates_selected_component_root_class_policy --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never`: passed.
- `cargo test -p zircon_editor --lib editor_component_adapter_registry_advertises_reflection_and_asset_editor_sources --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never`: passed.
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never`: passed with `204 passed; 0 failed; 675 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m10-root-class-authoring --message-format short --color never`: blocked before compile by unrelated runtime-interface manifest/lock drift.

## 2026-05-02 world-space UI pointer route closeout

- World-space UI surfaces now participate in viewport pointer routing before scene viewport dispatch.
- `SlintViewportController::route_world_space_ui_pointer_event` resolves the topmost submitted world-space surface by projected viewport bounds and captures it across Down/Move/Scroll/Up until release.
- The Slint viewport pointer callback now asks the world-space UI route first; when a surface is hit, the interaction is consumed by the world-space UI layer instead of leaking into scene camera/object selection.
- The route remains host-side/editor-owned. The render framework still consumes only `UiRenderExtract`; editor interaction semantics do not move into RHI.
- Added focused coverage for world-space UI pointer hit/capture behavior in the viewport controller tests. Full verification was delayed by long-running Windows Cargo compile/file-lock contention; the last observed blocker was no longer present in source, and `UiPointerEventKind` is confirmed `Copy`.

## 2026-05-02 VirtualList/PagedList real payload closeout

Related code:
- `zircon_editor/src/ui/slint_host/app/showcase_event_inputs.rs`
- `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs`
- `zircon_runtime/src/ui/component/state.rs`

The showcase event adapter now accepts value-bearing real-window payloads for complex collection components instead of relying only on fixed demo defaults. `VirtualListScrolled` can parse either named payloads such as `start=512,count=48`, compact payloads such as `128,24`, or a single start index that reuses the showcase default visible count. `PagedList` actions can parse named payloads such as `page=3,size=100`, compact payloads such as `4,50`, or a single page index that reuses the showcase default page size.

This keeps the existing action-only demo path intact while giving the real host/edit event path a stable route into `UiComponentShowcaseDemoEventInput::SetVisibleRange` and `UiComponentShowcaseDemoEventInput::SetPage`. Those inputs continue through the existing runtime component event resolver and state projection, so viewport metadata and pagination metadata remain sourced from the runtime component state rather than being special-cased in the host.

Validation status: focused unit coverage was added for both payload shapes. Cargo execution is deferred until the active Windows Cargo/Rustc processes from earlier validation attempts release their locks.

## 2026-05-02 Action-only complex list state advancement

Related code:
- `zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs`
- `zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs`
- `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs`

The component showcase action-only path now advances complex list controls from retained Runtime UI state instead of dispatching fixed sample values every time. `VirtualListScrolled` reads the current `viewport_start`, `viewport_count`, and `total_count` from showcase state and submits the next clamped visible window. `PagedListNextPage` reads the current `page_index`, `page_size`, and `page_count` and submits the next clamped page request.

This complements the value-bearing payload route: real host callbacks can either send explicit list/page payloads through the edited-value path or rely on action-only callbacks to advance from current state. Both paths still converge on the Runtime UI component reducer and host projection metadata.

## 2026-05-02 Asset Editor binding mutation adapter closeout

Related code:
- `zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs`

The UI Asset Editor binding inspector now aligns its registered data-source schema with the real mutation path for core binding fields. `binding.id.set`, `binding.event.set`, and `binding.route.set` now build Runtime UI component envelopes through the same `dispatch_ui_component_adapter_event` path already used by widget, slot, and layout fields. The asset-editor adapter now consumes `binding.id`, `binding.event`, and `binding.route` target paths and forwards them to the existing `EditorManager` mutation API.

This removes another direct field-mutation bypass from the real window callback path while preserving direct manager operations for structural commands such as add/delete and for advanced route/action targets that are not yet represented in the generic asset-editor data source schema.

## 2026-05-02 Asset Editor binding route schema and adapter completion

Related code:
- `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs`
- `zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs`

The asset-editor data source now describes the binding route/action target fields that already exist in the real UI Asset Editor callback surface. `binding.route_target` and `binding.action_target` are registered as writable asset-editor fields and are consumed by the asset-editor mutation adapter. Their real window callbacks now dispatch Runtime UI component envelopes instead of directly mutating through `EditorManager`.

After this change, the basic binding inspector fields `id`, `event`, `route`, `route_target`, and `action_target` share the same Runtime UI adapter path, transaction metadata, mutation source, projection patch contract, and presentation-dirty behavior as widget, slot, and layout inspector edits.

## 2026-05-02 Reflection data source mutation closeout

Related code:
- `zircon_editor/src/ui/template_runtime/component_adapter/reflection.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs`
- `zircon_editor/src/tests/ui/component_adapter.rs`

The generic reflection data source now has a real mutation path for selected entity/component fields instead of being descriptor-only. Reflection envelopes targeting `component://selected` or `entity://selected` are accepted by the component adapter registry and are delegated to the selected-entity Inspector draft mutation path for writable entity fields. This keeps the first resolver grounded in the already-supported Inspector field set instead of introducing a parallel mutation implementation.

The component reflection source now advertises concrete writable entity fields (`name`, `parent`, and `transform.translation.x/y/z`) plus the existing aggregate `transform.translation` Vec3 field. The aggregate Vec3 path is consumed directly by the reflection adapter and split into Inspector draft writes for the three translation axes. Fields that are only descriptive at this stage remain present but are not marked writable until a matching runtime mutation path exists.

Focused tests were added for reflection name mutation and aggregate translation mutation. Cargo execution remains deferred while existing Windows Cargo/Rustc processes are active.

## 2026-05-02 Runtime component data source access point

Related code:
- `zircon_editor/src/ui/host/editor_event_runtime_access.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/registry.rs`
- `zircon_editor/src/tests/ui/component_adapter.rs`

`EditorEventRuntime` now exposes the unified Runtime UI component data-source descriptors through `ui_component_data_sources()`. This keeps the registry as the owner of source descriptors while giving real host/property-editor code a stable runtime access point for Inspector, reflection, and asset-editor sources.

This is intentionally a thin query boundary: mutation still goes through `dispatch_ui_component_adapter_event`, while source discovery goes through the runtime access method. The separation gives future generic property editors a single place to enumerate available sources without directly depending on the adapter registry module.

## 2026-05-02 Runtime implementation import cutover

Related code:
- `zircon_editor/src/**`
- `zircon_runtime/src/ui/{surface,dispatch,template,component,event_ui}`
- `zircon_runtime_interface/src/ui/**`

The editor-side Runtime UI imports have been split along the dependency boundary. Stable DTOs and event envelopes remain sourced from `zircon_runtime_interface`, while runtime-owned implementation APIs now resolve through `zircon_runtime::ui::*`. This includes surface trees/extracts, pointer dispatchers, template compiler/build APIs, `UiEventManager`, and `UiComponentDescriptorRegistry`.

This avoids an impossible reverse dependency where `zircon_runtime_interface` would need to re-export runtime implementation types even though `zircon_runtime` already depends on the interface crate. It also directly addresses the validation blocker where editor compilation failed on missing `zircon_runtime_interface::ui::{surface, dispatch, template}` implementation symbols.

## 2026-05-02 Runtime tree implementation import cutover

Related code:
- `zircon_editor/src/**`
- `zircon_runtime/src/ui/tree/**`
- `zircon_runtime_interface/src/ui/tree/**`

The editor-side Runtime UI tree imports now follow the same split as surface, dispatch, and template APIs. Interface tree DTOs remain available for cross-boundary contracts, but editor code that calls implementation methods such as `UiTree::node()` now imports from `zircon_runtime::ui::tree`. This removes the validation drift where editor projection code compiled against the interface DTO type and then failed on runtime-only tree accessors.

## 2026-05-02 Runtime UI render-framework type convergence

Related code:
- `zircon_editor/src/ui/slint_host/viewport/submit_extract.rs`
- `zircon_editor/src/ui/slint_host/viewport/world_space_ui.rs`
- `zircon_editor/src/ui/slint_host/viewport/test_render_framework.rs`
- `zircon_editor/src/ui/slint_host/viewport/tests/fake_render_framework.rs`
- `zircon_runtime/src/core/framework/render/framework.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`

The editor Runtime UI host has been hard-cut to consume `zircon_runtime::ui::*` for the active UI implementation path. This removes the duplicated `zircon_runtime_interface::ui::*` versus `zircon_runtime::ui::*` type split that previously blocked descriptor re-export validation and downstream Asset Editor / world-space submission checks.

The render framework/RHI-facing UI submission path now consumes the runtime-owned `UiRenderExtract` and runtime-owned `UiFrame` geometry. World-space UI surfaces therefore merge into the viewport UI extract without crossing through an interface-copy `UiRenderExtract`, and the screen-space UI renderer receives the same runtime layout type carried by the extracted commands and text layout batches.

Validation evidence:
- `cargo test -p zircon_editor --lib component_adapter --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 12 component adapter tests.
- `cargo test -p zircon_editor --lib world_space --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 6 world-space and viewport UI submission tests.

Known non-blocking output:
- Existing runtime graphics warnings remain around unused imports, dead code, and an unused `execution_draws` local. These are not introduced by the UI type convergence and did not block the focused validation.

## 2026-05-02 Component catalog and interface event helper closeout

Related code:
- `zircon_runtime_interface/src/ui/component/event.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`
- `zircon_runtime/src/graphics/tests/render_framework_bridge.rs`
- `zircon_runtime/src/ui/tests/component_catalog.rs`

The shared component event contract now exposes `UiComponentEvent::kind()` on the interface side as well as the runtime side. This keeps `UiComponentEventEnvelope` self-validation compiling before the runtime catalog tests execute, and prevents event-kind drift between serialized component envelopes and typed events.

Runtime graphics UI renderer tests now construct `UiTreeId` and `UiNodeId` from `crate::ui::event_ui`, matching the runtime-owned `UiRenderExtract` path used by the renderer. This completes the local test fixture convergence after the render-framework UI extract cutover.

Validation evidence:
- `cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 39 component catalog tests.
- This catalog suite includes selection/state coverage and validates the expanded Runtime UI catalog categories, including Numeric and Selection descriptors.

Known non-blocking output:
- Existing runtime graphics test warnings remain around unused virtual-geometry helpers and debug readback accessors. They are unrelated to the Runtime UI component catalog/category path.

## 2026-05-02 Showcase and Inspector focused validation

Related code:
- `zircon_editor/src/ui/slint_host/app/showcase_event_inputs.rs`
- `zircon_editor/src/ui/template_runtime/showcase_demo_state.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/reflection.rs`

Focused editor validation now covers the Runtime UI showcase and inspector/asset-editor field mutation path after the runtime UI type convergence.

Validation evidence:
- `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 18 showcase tests.
- `cargo test -p zircon_editor --lib inspector --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 57 inspector-related tests.

The passing showcase suite covers category filtering, retained component state, complex component runtime events, and runtime component projection semantics. The passing inspector suite covers `dispatch_ui_component_adapter_event` adapter paths, real host callback dispatch, UI Asset Editor structured inspector field projection/update, drag source behavior, and reflection/action dispatch integration.

## 2026-05-02 UI Asset Editor adapter boundary validation

Related code:
- `zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs`
- `zircon_editor/src/ui/template_runtime/component_adapter/asset_editor.rs`
- `zircon_editor/src/tests/host/slint_window/ui_asset_editor.rs`

The UI Asset Editor detail dispatch contract now treats authored binding fields as component-adapter field commits instead of direct host detail actions. The structural host test was updated to assert that `binding.id`, `binding.event`, `binding.route`, `binding.route_target`, and `binding.action_target` flow through `dispatch_ui_asset_component_adapter_commit`, and that the asset editor component adapter owns the corresponding `EditorManager` mutation calls.

Validation evidence:
- `cargo test -p zircon_editor --lib ui_asset_editor --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 204 UI Asset Editor tests.

This closes the stale direct-dispatch expectation caught after the broader inspector suite had already proven the runtime adapter mutation path.

## 2026-05-02 Runtime render-framework UI bridge validation

Related code:
- `zircon_runtime/src/core/framework/render/framework.rs`
- `zircon_runtime/src/graphics/runtime/render_framework/render_framework_impl/trait_impl.rs`
- `zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs`
- `zircon_runtime/src/graphics/tests/render_framework_bridge.rs`

The RHI-facing render framework bridge was validated after switching shared UI submissions to runtime-owned `UiRenderExtract`, `UiFrame`, `UiTreeId`, and `UiNodeId` types. The bridge test suite confirms that frame submission, pipeline capability checks, and shared UI text payload accounting still pass with the runtime UI extract path.

Validation evidence:
- `cargo test -p zircon_runtime --lib render_framework_bridge --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-ui-closeout --message-format short --color never` passed: 29 render framework bridge tests.

Known non-blocking output:
- Existing runtime graphics test warnings remain around unused virtual-geometry debug/readback helpers and one unused render local. These warnings predate the Runtime UI bridge convergence and do not block the focused UI bridge validation.
