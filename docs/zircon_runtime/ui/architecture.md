---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/graphics/types/viewport_render_frame_from_public_runtime.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/arrange/grid_masonry.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/routing_diagnostics.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/arrangement.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/linear_slots.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/fallback_policy.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/grid_slots.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_input_manager_tests.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/surface/pointer
  - zircon_runtime/src/ui/surface/navigation
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/v2/style/tokens.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/slot_contract.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_v2_style.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_style_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_document.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_accessibility_extract.rs
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/state.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_component_catalog_editor_showcase.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility/extraction.rs
  - zircon_runtime/src/ui/tests/accessibility/naming_relations.rs
  - zircon_runtime/src/ui/tests/accessibility/focus_diagnostics.rs
  - zircon_runtime/src/ui/tests/accessibility/description_references.rs
  - zircon_runtime/src/ui/tests/accessibility/activation_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/dispatch_effects.rs
  - zircon_runtime/src/ui/tests/event_routing/shared_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/rebuild_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/render_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/mutation_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/material_layout/button_icon_metrics.rs
  - zircon_runtime/src/ui/tests/material_layout/row_label_metrics.rs
  - zircon_runtime/src/ui/tests/material_layout/field_values.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/v2/style.rs
  - zircon_runtime/src/ui/v2/style/runtime_state.rs
  - zircon_runtime/src/ui/v2/style/tokens.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_v2_style.rs
  - zircon_runtime/src/ui/tests/material_layout/asset_icon_roles.rs
  - zircon_runtime/src/ui/tests/material_layout/constraints_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_material_layout.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/template/loader_instance_validation.rs
  - zircon_runtime/src/ui/tests/template/interaction_bindings.rs
  - zircon_runtime/src/ui/tests/template/surface_containers.rs
  - zircon_runtime/src/ui/tests/template/slot_contracts.rs
  - zircon_runtime/src/ui/tests/template/layout_compute.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/slot_contract.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes.rs
  - zircon_runtime/src/ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/document/validation.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_style_apply.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_template_document.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_template.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs
  - zircon_runtime/src/ui/tests/component_catalog/descriptor_contracts.rs
  - zircon_runtime/src/ui/tests/component_catalog/registry_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/retained_events.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/collection_mutation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/reference_sources.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/interaction_numeric.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/action_selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/text_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/planned_layers.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/editor_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_x_runtime.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/folder_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/boundary/template_namespace.rs
  - zircon_runtime/src/ui/tests/boundary/layout_tree_surface.rs
  - zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs
  - zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/taffy_flex.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/taffy_wrap_grid.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/zircon_fallback.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/route_trace_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/lifecycle.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/metrics_dirty.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/basic_editing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/selection_navigation.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/word_shortcuts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/text_ime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/focus_state.rs
  - zircon_runtime/src/ui/tests/focus_navigation/property_mutation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/tab_directional.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/input_method.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/owner_validation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/high_precision_dispatch.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/drag_drop.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/popup_tooltip.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/route_trace.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_markdown.py
implementation_files:
  - docs/zircon_runtime/ui/architecture.md
  - zircon_runtime/src/ui/public_runtime_frame.rs
  - zircon_runtime/src/ui/tests/runtime_ui_support
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/navigation.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_input_manager_tests.rs
  - zircon_runtime/src/ui/surface/render/collection_rows/table.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/property_mutation/metadata_dirty.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/table/columns.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary/runtime_15_m2/ui.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/non_network_server_naming.py
  - zircon_runtime/src/ui/accessibility/extract.rs
  - zircon_runtime/src/ui/accessibility/extract/state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_accessibility_extract.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase.rs
  - zircon_runtime/src/ui/component/catalog/editor_showcase/descriptor_builders.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_component_catalog_editor_showcase.rs
  - zircon_runtime/src/ui/tests/accessibility.rs
  - zircon_runtime/src/ui/tests/accessibility/extraction.rs
  - zircon_runtime/src/ui/tests/accessibility/naming_relations.rs
  - zircon_runtime/src/ui/tests/accessibility/focus_diagnostics.rs
  - zircon_runtime/src/ui/tests/accessibility/description_references.rs
  - zircon_runtime/src/ui/tests/accessibility/activation_actions.rs
  - zircon_runtime/src/ui/tests/accessibility/value_actions.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_accessibility.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/event_routing/component_events.rs
  - zircon_runtime/src/ui/tests/event_routing/dispatch_effects.rs
  - zircon_runtime/src/ui/tests/event_routing/shared_input.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_event_routing.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/rebuild_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/incremental_layout.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/render_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains/mutation_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/material_layout.rs
  - zircon_runtime/src/ui/tests/material_layout/button_icon_metrics.rs
  - zircon_runtime/src/ui/tests/material_layout/row_label_metrics.rs
  - zircon_runtime/src/ui/tests/material_layout/field_values.rs
  - zircon_runtime/src/ui/tests/material_layout/asset_icon_roles.rs
  - zircon_runtime/src/ui/tests/material_layout/constraints_children.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_material_layout.rs
  - zircon_runtime/src/ui/tests/template.rs
  - zircon_runtime/src/ui/tests/template/loader_instance_validation.rs
  - zircon_runtime/src/ui/tests/template/interaction_bindings.rs
  - zircon_runtime/src/ui/tests/template/surface_containers.rs
  - zircon_runtime/src/ui/tests/template/slot_contracts.rs
  - zircon_runtime/src/ui/tests/template/layout_compute.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_template.rs
  - zircon_runtime/src/ui/tests/component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/catalog_inventory.rs
  - zircon_runtime/src/ui/tests/component_catalog/descriptor_contracts.rs
  - zircon_runtime/src/ui/tests/component_catalog/registry_queries.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/retained_events.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/collection_mutation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/reference_sources.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/interaction_numeric.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/action_selection.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/text_inputs.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_component_state_keyboard.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mod.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/planned_layers.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/editor_components.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/mui_x_runtime.rs
  - zircon_runtime/src/ui/tests/component_catalog/material_foundation/folder_structure.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_component_catalog_material_foundation.rs
  - zircon_runtime/src/ui/tests/boundary.rs
  - zircon_runtime/src/ui/tests/boundary/template_namespace.rs
  - zircon_runtime/src/ui/tests/boundary/layout_tree_surface.rs
  - zircon_runtime/src/ui/tests/boundary/binding_event_roots.rs
  - zircon_runtime/src/ui/tests/boundary/asset_fixture_projection.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_boundary.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/arranged_authority.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/taffy_flex.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/taffy_wrap_grid.rs
  - zircon_runtime/src/ui/tests/surface_frame_authority/zircon_fallback.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_surface_frame_authority.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/route_trace_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/semantic_actions.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/timers_disabled.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/directional.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/drag_reorder.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/virtualization.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs
  - zircon_runtime/src/ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_event_abi.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/lifecycle.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/pointer_routes.rs
  - zircon_runtime/src/ui/tests/runtime_window_input_pump/metrics_dirty.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_window_input_pump.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/basic_editing.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/selection_navigation.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/word_shortcuts.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/text_ime.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/focus_state.rs
  - zircon_runtime/src/ui/tests/focus_navigation/property_mutation.rs
  - zircon_runtime/src/ui/tests/focus_navigation/tab_directional.rs
  - zircon_runtime/src/ui/tests/focus_navigation/modal_popup.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_focus_navigation.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/window_timer.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_order.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/route_matrix.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager/touch_pointer.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/input_method.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/owner_validation.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/high_precision_dispatch.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/drag_drop.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/popup_tooltip.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership/route_trace.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_runtime_input_ownership.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/layout/pass/pipeline.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/incremental.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/arrange/grid_masonry.rs
  - zircon_runtime/src/ui/layout/pass/responsive_mui.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/routing_diagnostics.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/arrangement.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/linear_slots.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/fallback_policy.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass/grid_slots.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/ui_taffy_layout_pass.rs
  - zircon_runtime_interface/src/ui/layout/engine.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge/compute.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
  - zircon_runtime/src/ui/v2/cache.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/template/asset/component_contract/validation.rs
  - zircon_runtime_interface/src/ui/template/asset/component_contract/api_version.rs
  - zircon_runtime_interface/src/tests/ui_v2_contracts.rs
  - zircon_runtime/src/tests/runtime_absorption/dynamic_api_session/v2_contract.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/pipeline.rs
  - zircon_runtime/src/ui/template/loader.rs
  - zircon_runtime/src/ui/template/validate.rs
  - zircon_runtime/src/ui/template/instance.rs
  - zircon_runtime/src/ui/template/build/interaction.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/artifact.rs
  - zircon_runtime/src/ui/tests/template_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_markdown.py
plan_sources:
  - user: 2026-06-13 runtime architecture implementation request
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - CLAUDE.md
tests:
  - runtime_09_m0_ui_architecture_static_passed
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_doc_records_current_boundaries
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_baselines_match_current_source_scan
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_v2_verdict_matches_runtime_and_interface_modules
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_input_events_route_through_single_dispatch_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_navigation_legacy_reply_rename_reduces_ui_input_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt
  - zircon_runtime::tests::runtime_absorption::structure_convention::production_file_budget::ui_accessibility_extract::runtime_15_ui_accessibility_extract_state_is_child_owner
  - zircon_runtime::tests::runtime_absorption::structure_convention::production_file_budget::ui_component_catalog_editor_showcase::runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_accessibility::runtime_15_ui_accessibility_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_event_routing::runtime_15_ui_event_routing_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_surface_dirty_domains::runtime_15_ui_surface_dirty_domains_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_material_layout::runtime_15_ui_material_layout_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_template::runtime_15_ui_template_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_surface_frame_authority::runtime_15_ui_surface_frame_authority_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_runtime_input_reply_routes::runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::structure_convention::test_file_budget::ui_runtime_input_reply_routes::runtime_15_ui_runtime_input_reply_route_children_are_folder_backed
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_taffy_layout_pass_order_uses_bridge_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_virtualization_scroll_boundary_records_invalidation_authority
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_template_pipeline_boundary_records_compile_instance_validate_authority
  - zircon_runtime::ui::tests::scroll_virtualization::virtualized_list_only_materializes_visible_window
  - zircon_runtime::ui::tests::scroll_virtualization::scroll_offset_invalidates_virtualization_window
  - zircon_runtime::ui::tests::scroll_virtualization::non_virtualized_scroll_offset_keeps_full_window_dirty_domain
  - zircon_runtime::ui::tests::template_pipeline::template_validate_rejects_unknown_component_contract
  - zircon_runtime::ui::tests::template_pipeline::template_instance_failure_surfaces_loader_error
  - zircon_runtime::ui::tests::template_pipeline::compiled_template_artifact_stays_binary_leaf_dto_not_generated_source
  - ui_architecture_boundary targeted audit
  - docs/zircon_runtime/ui/v2.md
  - docs/zircon_runtime/ui/dispatch/input_manager.md
  - docs/zircon_runtime/ui/platform_input.md
  - docs/zircon_runtime/ui/layout/pass.md
  - docs/zircon_runtime/ui/surface/default_interactions.md
  - docs/zircon_runtime/ui/template/pipeline.md
  - docs/zircon_runtime/ui/template/asset/dependency_index.md
doc_type: module-detail
---

# Runtime UI Architecture M0 Boundary

runtime_09_m0_ui_architecture_static_passed

本文件完成 Runtime 09 的 M0.1 模块边界图与 M0.2 v2 双代裁决，并记录 M1.1 输入路由单点权威化、M1.2 导航回复、pointer 回复、pointer capture fallback、table row-label fallback、template component-name fallback、property visibility flag、responsive MUI visibility flag、accessibility open-state fallback、layout engine backend name 与 surface default interaction fallback 命名收敛、M2.1 Taffy 桥接与 layout pass 顺序权威化、M2.2 virtualization/scroll 边界声明、M3.1 template compile/instance/validate pipeline 边界切片，以及 Editor UI 01.M1.S2 的 `platform_input/` 平台输入归一化 owner。当前 UI 生产代码的 legacy 命名桶已清零；完整 UI behavior filters 仍必须等待 owner 空窗或重新协调。

## Owner Verdict

`zircon_runtime::ui` owns runtime-only UI behavior: platform input adapters, layout passes, dispatch, render extraction inputs, text/layout engines, template compilation, surface/tree mutation, runtime v2 prototype loading, v2 style resolution, and v2 surface construction.

`zircon_runtime_interface::ui` owns neutral UI contract DTOs. Its `ui::v2` surface is the stable data schema layer for arenas, asset records, compiled graphs, repeat expansion records, and style DTOs. It must not own runtime mutation, route ordering, layout pass execution, cache invalidation, or render extraction.

`zircon_editor::ui` owns editor workbench authoring and retained-host consumption. It can consume v2 assets and runtime projection APIs, but it cannot define runtime UI route authority, layout backend ownership, or template compilation behavior.

## Module Boundary Map

Current scan baseline:

- `ui/` top-level entries: 19 = 15 directories plus `module.rs`, `prelude.rs`, `public_runtime_frame.rs`, and `style.rs` (`mod.rs` is the root façade and is excluded from this audit count).
- `surface/` entries: 20 in the current worktree scan.
- Full UI-tree `legacy` hits: `ui_legacy_hits=54`.
- Production UI `legacy` hits/files after excluding tests and fixtures: `ui_legacy_production_hits=0` / `ui_legacy_production_files=0`.
- Production UI `taffy` hits/files after excluding tests and fixtures: `ui_taffy_production_hits=175` / `ui_taffy_production_files=10`.

| Module | Runtime owner | Boundary note |
|---|---|---|
| `module.rs` | Runtime UI module declaration | Module descriptor/config wiring only. |
| `public_runtime_frame.rs` | Public runtime frame DTO | Production frame bundle handed to graphics conversion without exposing fixture manager support as production UI API. |
| `layout/` | Constraints, pass sequence, scroll, style mapping, Taffy bridge, virtualization | Owns layout execution and backend adaptation. M2.1 makes `taffy_bridge/compute.rs` the only Taffy tree-build/compute owner, `pass/pipeline.rs` the authoritative pass-order owner, and leaves `style_mapping.rs` as a DTO adapter rather than a layout backend executor. M2.2 makes `layout/scroll.rs::UiScrollVirtualizationPlan` / `plan_scrollable_virtual_window(...)` the owner for scroll offset clamping, viewport/content invalidation, and virtual-window dirty decisions while `layout/virtualization.rs` remains pure window math. |
| `surface/` | Retained surface state and runtime interaction state | Owns arranged output, hit testing, focus, popup stack, input state, component state, property mutation, default interactions, reflection snapshots, render collection data, timeline, and diagnostics. |
| `dispatch/` | UI route manager | Owns the route authority entry point documented by `UiInputManager`: capture, popup, preview, target, bubble, focus, default action. |
| `platform_input/` | Platform input normalization | Feature-gated `platform-winit` owner for winit `WindowEvent` and modifier normalization before events enter `UiWindowInputPumpBatch`; editor-local winit interpretation is slated for deletion in Editor UI 01.M1.S3. |
| `template/` | Asset/build/instance/loader/validate/pipeline boundary | Owns template compilation, validation, dependency indexing, hot reload coordination, and old template migration surfaces. M3.1 records `UiTemplateRuntimePipeline` as the thin `load -> validate -> instance -> build` owner boundary while v2 remains the replacement mainline. |
| `v2/` | Runtime v2 loader, compiler, prototype cache, style resolver, surface builder, surface tree | Runtime implementation of the v2 schema. It consumes `zircon_runtime_interface::ui::v2` DTOs and creates runtime surfaces. |
| `component/` | Component descriptor catalog and state reducers | Owns runtime catalog metadata and component state reduction, including editor shell component entries that are still runtime projection data. |
| `tree/` | Runtime tree extensions and hit-test helpers | Utility owner for UI node tree traversal and hit-test integration. |
| `binding/` | Binding event router and update report | Runtime binding/event bridge into surface mutation reports. |
| `event_ui/` | UI event manager | Runtime UI event manager layer. `UiBindingCodec` is an interface-owned contract type after Runtime 10 M2.1. |
| `style.rs` and `theme/` | Typed style fields and active theme registry | Owns runtime style resolution inputs and theme token registry over v2 resolved style DTOs. |
| `text/` | Crate-private text layout support | Runtime internal helper; broader text stack ownership remains Runtime 01 M2. |
| `accessibility/` | Runtime accessibility extraction | Reads surface state into accessibility output; Runtime 09 M1.2 names the open-state compatibility list as `fallback_properties`, leaving remaining accessibility behavior as runtime-owned extraction rather than legacy migration debt. |
| `icon_atlas/` | Runtime icon atlas support | Leaf runtime UI support module. |
| `tests/` | Runtime UI test tree | Not a production owner; excluded from production debt file counts above. Runtime fixture/manager support lives under `tests/runtime_ui_support` and is mounted only through `#[cfg(test)]`. |

Dependency direction:

```text
template asset/build -> v2 cache/compiler -> v2 surface_tree -> surface
component catalog/state -> v2 surface_tree + surface default interactions
surface -> layout pass -> arranged output
platform_input -> window input pump
surface input/focus/popup state -> dispatch route authority
binding/event_ui -> surface mutation reports and route results
public_runtime_frame -> graphics::types::ViewportRenderFrame conversion
tests/runtime_ui_support -> v2 prototype cache + surface + theme
interface::ui::v2 DTOs -> runtime::ui::v2 implementation
```

Runtime 10 M2.1 removes the remaining runtime-local UI contract duplicates instead of adding compatibility aliases. `UiBindingCodec` now lives only under `zircon_runtime_interface::ui::event_ui`, and `UiAssetSchemaVersionPolicy` plus schema-version constants live only under `zircon_runtime_interface::ui::template::asset::schema`; runtime template/schema code imports those interface contracts directly and keeps behavior in `UiAssetSchemaMigrator`.

No M0 blocker-level owner inversion was found in the scanned module graph. The remaining work is not "unknown ownership"; it is explicit cleanup of debt-bearing areas:

1. UI legacy naming and migration terms: production UI source now contains no `legacy` scan hits after Runtime 09 M1.2 cutovers. `runtime_09_m1_2_navigation_legacy_reply_renamed_static_passed_cargo_pending` removed the navigation reply variable from this bucket by renaming the local route reply to `routed_reply`; `runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending` removed the pointer reply local naming debt by using `routed_result`; `runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending` removed the pointer capture fallback API wording debt, and Editor UI 01.M4.S1 now requires indexed ownership through `has_pointer_capture_for_owner`; `runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending` removed the render table row-label fallback wording debt by using `split_row_label_table_text`; `runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending` removed the template interaction inference wording debt by using `component_name_interaction_fallback`; `runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending` removed the property mutation visibility-transition wording debt by using `state_visible_flag`; `runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending` removed the responsive MUI visibility DTO wording debt by using the same `state_visible_flag` semantic name; `runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending` removed the accessibility open-state fallback wording debt by using `fallback_properties`; `runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending` removed the layout-engine public backend wording debt by hard-cutting to `UiLayoutEngineBackend::Zircon`, `UiLayoutEngineCapability::zircon()`, and `zircon_selected_count`; `runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending` removed the final surface default interaction fallback wording debt by using `fallback_properties` in `default_open_boolean_value(...)`.
2. Taffy backend exposure: 9 production files currently mention `taffy`. `runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending` hardens the intended owner shape: `taffy_bridge/compute.rs` owns Taffy tree construction and `compute_layout`, `pass/taffy_arrange.rs` owns eligibility/fallback/recursive arrange only, and `style_mapping.rs` remains `runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter`.
3. Virtualization and scroll cache invalidation: `runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending` records `UiScrollVirtualizationPlan` as the layout-owned authority for scroll offset, viewport/content extent, and visible-range invalidation. Tree scrolling and layout arrange consume that planner; render, hit grid, and editor code do not compute virtual windows.
4. Template generation and migration: `runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending` records `UiTemplateRuntimePipeline`, `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`, `UiTemplateRuntimePipelineError`, template failure-path anchors, and the binary DTO generated policy. Old recursive template paths still coexist with v2 runtime paths as migration/test surfaces.

## M1.1 Input Route Authority

runtime_09_m1_1_ui_input_route_authority_static_passed_cargo_pending

The normalized runtime UI input path is now:

```text
platform/window input -> UiInputEvent -> UiSurface::dispatch_input_event
  -> surface/input/dispatch.rs -> leaf dispatchers
  -> route_authority diagnostic note
  -> component/focus/popup/default-action result
```

`surface/input/dispatch.rs` is the single exit for normalized `UiInputEvent` dispatch. It collects pointer, navigation, keyboard, text, IME, analog, mouse-motion, drag-drop, popup, tooltip/typeahead/submenu/toast timer, and accessibility leaf results, then calls `annotate_authoritative_input_dispatch`.

`surface/input/route_authority.rs` consumes `zircon_runtime::ui::dispatch::UI_INPUT_ROUTE_ORDER` and records a diagnostic note shaped as `route_authority=runtime_09_m1_1_ui_input_route_authority;policy=...;stages=...`. The stage list is derived from the route policy while preserving the manager-owned order: pointer capture, popup stack, preview tunnel, direct target, bubble path, focus path, default action.

Bypass owner verdict:

- `UiSurface::dispatch_input_event`, `UiSurface::dispatch_input_event_with_manager`, test-support `RuntimeUiManager::dispatch_input_event`, and window-pump normalization are the normalized route authority path.
- `UiSurface::dispatch_pointer_event*` and test-support `RuntimeUiManager::dispatch_pointer_event` remain direct pointer leaf helpers for existing low-level callers/tests.
- `UiSurface::dispatch_navigation_event` and test-support `RuntimeUiManager::dispatch_navigation_event` remain direct navigation leaf helpers for existing low-level callers/tests.
- These direct pointer/navigation entry points are not unowned bypasses; they are recorded as `runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers` until callers can migrate to normalized `UiInputEvent` dispatch or the helpers are retired.

## M1.2 Pointer Reply Naming

runtime_09_m1_2_pointer_legacy_reply_renamed_static_passed_cargo_pending

The direct pointer route helpers now name the dispatch result as `routed_result` in `surface/input/pointer.rs` and `surface/input/pointer_reply.rs`. This is a local naming cutover only: route order, focus/popup capture behavior, diagnostics, and returned `UiInputDispatchResult` data flow are unchanged.

`runtime_09_pointer_legacy_reply_rename_reduces_ui_input_debt` keeps both files from reintroducing `legacy` wording while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Pointer Capture Naming

runtime_09_m1_2_pointer_capture_fallback_renamed_static_passed_cargo_pending

The high-precision pointer effect path now calls `has_pointer_capture_for_owner` in `surface/input/effect/focus_pointer.rs`, backed by the per-pointer capture map in `surface/input/state/pointer_capture.rs`. Editor UI 01.M4.S1 removed the unindexed single-pointer fallback state, so high-precision pointer mode requires an explicit `UiPointerId -> UiNodeId` capture entry for the same owner while `surface.focus.captured` remains the focused capture owner snapshot.

`runtime_09_pointer_capture_fallback_rename_reduces_ui_input_debt` keeps the old `has_legacy_or_indexed_pointer_capture_for_owner` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Table Row Label Fallback Naming

runtime_09_m1_2_table_row_label_fallback_renamed_static_passed_cargo_pending

The table collection-row renderer now calls `split_row_label_table_text` in `surface/render/collection_rows/table.rs` when explicit `cells` / `columns` / `options` data is absent and the row label must be split into display cells. The behavior is unchanged: explicit cell arrays still win, and row-label fallback splitting still follows the same whitespace token cases.

`runtime_09_table_row_label_fallback_rename_reduces_ui_render_debt` keeps the old `split_legacy_table_text` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Template Component-Name Fallback Naming

runtime_09_m1_2_template_component_name_fallback_renamed_static_passed_cargo_pending

The template build interaction inference now calls `component_name_interaction_fallback` in `template/build/interaction.rs` when a node has no authored `input_*` metadata and no binding-derived input capability. The behavior is unchanged: explicit input metadata remains authoritative, binding event capabilities still infer input flags first, and the component-name fallback still only promotes `Button`, `IconButton`, and `TextField`.

`runtime_09_template_component_name_fallback_rename_reduces_ui_template_debt` keeps the old `legacy_component_interaction_fallback` and `legacy_interactive` wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Property Visibility Flag Naming

runtime_09_m1_2_property_visibility_flag_renamed_static_passed_cargo_pending

The property mutation visibility transition helper now calls the `UiVisibility::effective(...)` boolean input `state_visible_flag` in `surface/property_mutation.rs`. The behavior is unchanged: `visibility_transition_dirty(...)` still compares the current and next effective layout occupancy using the same retained `UiStateFlags::visible` value, and only marks layout dirty when occupancy changes.

`runtime_09_property_visibility_flag_rename_reduces_ui_surface_debt` keeps the old `legacy_visible` local wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Responsive MUI Visibility Flag Naming

runtime_09_m1_2_responsive_mui_visibility_flag_renamed_static_passed_cargo_pending

The responsive MUI layout pre-pass now names the resolved authored `visible` boolean as `state_visible_flag` in `layout/pass/responsive_mui.rs`. The behavior is unchanged: `display`, `visibility`, and authored `visible` still resolve before measurement, and the pre-pass still writes the same `UiStateFlags::visible` value and dirty domains before full or incremental layout solving.

`runtime_09_responsive_mui_visibility_flag_rename_reduces_ui_layout_debt` keeps the old `legacy_visible` DTO wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## M1.2 Accessibility Open-State Fallback Naming

runtime_09_m1_2_accessibility_open_state_fallback_renamed_static_passed_cargo_pending

The accessibility snapshot extractor now names the retained/component-state open-state compatibility list `fallback_properties` in `accessibility/extract.rs`. The behavior is unchanged: authored `open_property` remains authoritative, then the same component-state property is read, then retained/component-state alternatives `expanded`, `popup_open`, and `open` are checked before true runtime expanded/popup flags.

`runtime_09_accessibility_open_state_fallback_rename_reduces_ui_a11y_debt` keeps the old `legacy_properties` and `legacy_property` helper wording from returning while the remaining Runtime 09 M1.2 legacy bucket is handled file by file.

## Runtime 15 M3 UI accessibility test folder split

runtime_15_ui_accessibility_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps the production accessibility owner unchanged in `ui/accessibility/extract.rs` and only splits the oversized test owner. `ui/tests/accessibility.rs` now owns shared fixtures, helper construction, and child module mounting; behavior tests live in `ui/tests/accessibility/extraction.rs`, `ui/tests/accessibility/naming_relations.rs`, `ui/tests/accessibility/focus_diagnostics.rs`, `ui/tests/accessibility/description_references.rs`, `ui/tests/accessibility/activation_actions.rs`, and `ui/tests/accessibility/value_actions.rs`.

`runtime_15_ui_accessibility_tests_are_folder_backed` locks the parent/child layout, prevents representative extraction/value-action tests from moving back into the parent, preserves all 49 accessibility tests, and keeps every UI accessibility test owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI accessibility widget actions test folder split

runtime_15_ui_accessibility_widget_actions_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production accessibility action dispatch unchanged and only splits the oversized widget-action test owner. `ui/tests/accessibility_widget_actions.rs` now owns shared surface fixtures, dispatch helpers, binding report assertions, and child module mounting; disclosure tests live in `ui/tests/accessibility_widget_actions/disclosure_actions.rs`, popup tests live in `ui/tests/accessibility_widget_actions/popup_actions.rs`, and tooltip/menu fallback tests live in `ui/tests/accessibility_widget_actions/tooltip_menu.rs`.

`runtime_15_ui_accessibility_widget_actions_tests_are_folder_backed` locks the parent/child layout, prevents representative disclosure/popup/tooltip-menu tests from moving back into the parent, preserves all 11 accessibility widget-action tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI layout slots test folder split

runtime_15_ui_layout_slots_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production slot layout unchanged and only splits the oversized layout-slot test owner. `ui/tests/layout_slots.rs` now owns shared constraints, pointer node fixtures, render/hit frame lookup helpers, and child module mounting; linear and free slot tests live in `ui/tests/layout_slots/linear_free.rs`, overlay and scroll surface-frame tests live in `ui/tests/layout_slots/overlay_scroll.rs`, and flow/grid/masonry tests live in `ui/tests/layout_slots/flow_grid_masonry.rs`.

`runtime_15_ui_layout_slots_tests_are_folder_backed` locks the parent/child layout, prevents representative linear/free, overlay/scroll, and flow/grid/masonry tests from moving back into the parent, preserves all 10 layout-slot tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI surface-frame authority test folder split

runtime_15_ui_surface_frame_authority_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production surface-frame authority unchanged and only splits the oversized surface-frame authority test owner. `ui/tests/surface_frame_authority.rs` now owns shared constants, surface/button/layout fixtures, Taffy/Grid/SizeBox construction helpers, and child module mounting; arranged/focus authority tests live in `ui/tests/surface_frame_authority/arranged_authority.rs`, Taffy flex tests live in `ui/tests/surface_frame_authority/taffy_flex.rs`, wrap/grid tests live in `ui/tests/surface_frame_authority/taffy_wrap_grid.rs`, and Zircon SizeBox fallback coverage lives in `ui/tests/surface_frame_authority/zircon_fallback.rs`.

`runtime_15_ui_surface_frame_authority_tests_are_folder_backed` locks the parent/child layout, prevents representative arranged/focus, Taffy flex, wrap/grid, and Zircon fallback tests from moving back into the parent, preserves all 9 surface-frame authority tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI surface dirty domains test folder split

runtime_15_ui_surface_dirty_domains_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production dirty-domain rebuild behavior unchanged and only splits the oversized dirty-domain test owner. `ui/tests/surface_dirty_domains.rs` now owns shared surface fixtures, dirty flag helpers, layout route assertions, keyboard event fixtures, fixed constraints, and child module mounting; rebuild phase tests live in `ui/tests/surface_dirty_domains/rebuild_domains.rs`, incremental layout route tests live in `ui/tests/surface_dirty_domains/incremental_layout.rs`, render-only invalidation tests live in `ui/tests/surface_dirty_domains/render_domains.rs`, and route state mutation tests live in `ui/tests/surface_dirty_domains/mutation_state.rs`.

`runtime_15_ui_surface_dirty_domains_tests_are_folder_backed` locks the parent/child layout, prevents representative rebuild, incremental layout, render-only, and mutation/state tests from moving back into the parent, preserves all 13 dirty-domain tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI material layout test folder split

runtime_15_ui_material_layout_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production Material measurement unchanged and only splits the oversized material-layout test owner. `ui/tests/material_layout.rs` now owns shared Material leaf measurement, render command, intrinsic constraint helpers, and child module mounting; button/icon tests live in `ui/tests/material_layout/button_icon_metrics.rs`, row/label tests live in `ui/tests/material_layout/row_label_metrics.rs`, field value tests live in `ui/tests/material_layout/field_values.rs`, asset/icon role tests live in `ui/tests/material_layout/asset_icon_roles.rs`, and authored constraint/child-content tests live in `ui/tests/material_layout/constraints_children.rs`.

`runtime_15_ui_material_layout_tests_are_folder_backed` locks the parent/child layout, prevents representative button/icon, row/label, field value, asset/icon role, and constraints/children tests from moving back into the parent, preserves all 23 material-layout tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI template test folder split

runtime_15_ui_template_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production template loading and surface projection unchanged and only splits the oversized template test owner. `ui/tests/template.rs` now owns shared template TOML fixtures, tree/root helpers, and child module mounting; loader/instance validation tests live in `ui/tests/template/loader_instance_validation.rs`, interaction binding tests live in `ui/tests/template/interaction_bindings.rs`, surface/container projection tests live in `ui/tests/template/surface_containers.rs`, slot contract tests live in `ui/tests/template/slot_contracts.rs`, and layout compute coverage lives in `ui/tests/template/layout_compute.rs`.

`runtime_15_ui_template_tests_are_folder_backed` locks the parent/child layout, prevents representative loader, interaction, surface, slot, and layout compute tests from moving back into the parent, preserves all 22 template tests, and keeps every owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI component catalog test folder split

runtime_15_ui_component_catalog_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production component descriptors and registry behavior unchanged and only splits the oversized component catalog test owner. `ui/tests/component_catalog.rs` now owns child module mounting plus shared descriptor, registry, schema, category, and drag-source helpers; V1 catalog inventory coverage lives in `ui/tests/component_catalog/catalog_inventory.rs`, descriptor tier/schema validation lives in `ui/tests/component_catalog/descriptor_contracts.rs`, and host-capability, palette view, and registry revision coverage lives in `ui/tests/component_catalog/registry_queries.rs`.

`runtime_15_ui_component_catalog_tests_are_folder_backed` locks the parent/child layout, prevents representative catalog, descriptor, registry query, and revision tests from moving back into the parent, preserves all 7 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI boundary test folder split

runtime_15_ui_boundary_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps the production UI namespace and asset fixture boundaries unchanged and only splits the oversized boundary test owner. `ui/tests/boundary.rs` now owns shared file-system and path helpers plus child module mounting; template namespace coverage lives in `ui/tests/boundary/template_namespace.rs`, layout/tree/surface/dispatch namespace coverage lives in `ui/tests/boundary/layout_tree_surface.rs`, binding/event/root structural coverage lives in `ui/tests/boundary/binding_event_roots.rs`, and runtime UI asset fixture plus `.zui` projection checks live in `ui/tests/boundary/asset_fixture_projection.rs`.

`runtime_15_ui_boundary_tests_are_folder_backed` locks the parent/child layout, prevents representative template, layout, surface, binding, event, asset fixture, and `.zui` projection tests from moving back into the parent, preserves all 32 migrated parent tests, and keeps every owner under the Runtime 15 file budget. The 2026-07-03 `runtime_15_ui_boundary_zui_surface_projection_guard_sync_static_passed_cargo_deferred` follow-up aligns the structure guard with `zui_surface_projection_does_not_call_template_tree_builder`, after the older `ui_v2_*` wording was retired. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 UI boundary runtime-host forbidden attribute literal cleanup

runtime_15_ui_boundary_runtime_host_literal_cleanup_static_passed_cargo_deferred

Runtime 15 keeps the production UI namespace and runtime host behavior unchanged while removing a source-scan false positive from the boundary guard itself. `tests/ui_boundary/runtime_host.rs` now builds the forbidden dead-code allow attribute through `DEAD_CODE_ALLOW_ATTRIBUTE = concat!("#[allow(", "dead_code", ")]")` instead of embedding that attribute as a direct test-source literal.

`runtime_ui_host_surface_splits_production_frame_from_test_support` still verifies that `ui/mod.rs` exposes the production `PublicRuntimeFrame`, keeps runtime UI support mounted only for tests, and does not reintroduce the old direct runtime UI module surface. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI component state test folder split

runtime_15_ui_component_catalog_component_state_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production component state mutation behavior unchanged and only splits the oversized component-state test parent. `ui/tests/component_catalog/component_state.rs` now owns only shared imports and child module mounting; retained number/dropdown/drop event tests live in `ui/tests/component_catalog/component_state/retained_events.rs`, array/map mutation tests live in `ui/tests/component_catalog/component_state/collection_mutation.rs`, reference source serialization and drop behavior live in `ui/tests/component_catalog/component_state/reference_sources.rs`, and transient interaction plus numeric/range clamp coverage lives in `ui/tests/component_catalog/component_state/interaction_numeric.rs`.

`runtime_15_ui_component_catalog_component_state_tests_are_folder_backed` locks the parent/child layout, prevents representative retained, collection, reference source, interaction, and numeric tests from moving back into the parent, preserves all 18 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI component state keyboard test folder split

runtime_15_ui_component_catalog_component_state_keyboard_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production keyboard component-state behavior unchanged and only splits the oversized keyboard test owner. `ui/tests/component_catalog/component_state/keyboard.rs` now owns only shared imports, `menu_option`, and child module mounting; action selection coverage lives in `ui/tests/component_catalog/component_state/keyboard/action_selection.rs`, menu/tree/table navigation and prefix typing coverage lives in `ui/tests/component_catalog/component_state/keyboard/menu_navigation.rs`, text-input edit coverage lives in `ui/tests/component_catalog/component_state/keyboard/text_inputs.rs`, and numeric/range control coverage lives in `ui/tests/component_catalog/component_state/keyboard/numeric_controls.rs`.

`runtime_15_ui_component_catalog_component_state_keyboard_tests_are_folder_backed` locks the parent/child layout, prevents representative action, menu navigation, text input, and numeric/range tests from moving back into the parent, preserves all 14 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M4 UI component state-reducer keyboard menu submenu owner split

runtime_15_ui_component_state_reducer_keyboard_menu_submenu_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps menu focus-control detection, keyboard text typeahead, search query/filter state, search option flattening, recursive filtering, and filtered option visibility in `ui/component/state_reducer/keyboard/menu.rs` while splitting submenu state transitions into `ui/component/state_reducer/keyboard/menu/submenu.rs`. The child owner now owns submenu focus-loop checks, hover-pending state, open/close state writes, active parent index updates, invalid submenu pruning, submenu target lookup, and submenu string state writes.

`runtime_15_ui_component_state_reducer_keyboard_menu_submenu_is_child_owner` locks the parent/child layout, prevents submenu constants and state-machine helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.

## Runtime 15 M4 UI component state-reducer tree view editing owner split

runtime_15_ui_component_state_reducer_tree_view_editing_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps tree-view identity, keyboard expand/collapse, toggle expanded, select option, multi/single selection, range selection, ordered/expanded/selected node id helpers, and disabled-option checks in `ui/component/state_reducer/tree_view.rs` while splitting rename/editing state transitions into `ui/component/state_reducer/tree_view/editing.rs`. The child owner now owns begin/cancel/commit rename, editing property fallback, editing state clearing, rename payload writes, tree node label lookup, and edit text validation.

`runtime_15_ui_component_state_reducer_tree_view_editing_is_child_owner` locks the parent/child layout, prevents editing constants and rename state-machine helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.

## Runtime 15 M3 UI Material foundation test folder split

runtime_15_ui_component_catalog_material_foundation_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production Material component descriptors unchanged and only splits the oversized Material foundation test parent. `ui/tests/component_catalog/material_foundation/mod.rs` now owns only Material foundation module mounting plus shared schema helpers; planned catalog inventory and common MUI customization checks live in `ui/tests/component_catalog/material_foundation/planned_layers.rs`, editor descriptor contract checks live in `ui/tests/component_catalog/material_foundation/editor_components.rs`, MUI surface/overlay/feedback checks live in `ui/tests/component_catalog/material_foundation/mui_surface_overlay.rs`, MUI X/runtime visibility checks live in `ui/tests/component_catalog/material_foundation/mui_x_runtime.rs`, and the existing catalog folder-shape guard lives in `ui/tests/component_catalog/material_foundation/folder_structure.rs`.

`runtime_15_ui_component_catalog_material_foundation_tests_are_folder_backed` locks the parent/child layout, prevents representative planned-layer, editor, surface/overlay, MUI X, and folder-structure tests from moving back into the parent, preserves the split Material foundation coverage, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI asset test folder split

runtime_15_ui_asset_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production UI asset loading, migration, and component schema compilation unchanged and only splits the oversized `ui/tests/asset.rs` parent. The parent now owns shared imported widget/style/layout fixtures and child module mounting; stable stylesheet/rule id coverage lives in `ui/tests/asset/style_rule_ids.rs`; style write API and reorder coverage lives in `ui/tests/asset/style_write_apis.rs`; loader validation coverage lives in `ui/tests/asset/loader_validation.rs`; imported widget/reference compiler coverage lives in `ui/tests/asset/document_compiler.rs`; source/flat fixture migration coverage lives in `ui/tests/asset/fixture_migration.rs`; and runtime component schema coverage lives in `ui/tests/asset/component_schema.rs`.

`runtime_15_ui_asset_tests_are_folder_backed` locks the parent/child layout, prevents representative style id/write API, loader validation, document compiler, fixture migration, and component schema tests from moving back into the parent, preserves all 32 moved tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI asset surface index test folder split

runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production asset indexing and hot reload behavior unchanged and only splits the near-budget `ui/tests/asset_surface_index.rs` owner. The parent now owns shared asset/surface fixture helpers and child module mounting; surface dependency edge tests live in `ui/tests/asset_surface_index/surface_edges.rs`; node resource registration and precise node target tests live in `ui/tests/asset_surface_index/node_resources.rs`; and dirty target application plus template rebuild target tests live in `ui/tests/asset_surface_index/dirty_targets.rs`.

`runtime_15_ui_asset_surface_index_tests_are_folder_backed` locks the parent/child layout, prevents representative surface-edge, node-resource, and dirty-target tests from moving back into the parent, preserves all 12 moved tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI asset MUI web form style test folder split

runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production MUI form style loading and retained template compilation unchanged and only splits the near-budget form-style test parent. `ui/tests/asset_mui_web_form_style.rs` now owns the shared MUI form style/layout TOML fixtures, module mounting, and find/attribute/class helpers; form control utility-class coverage lives in `ui/tests/asset_mui_web_form_style/form_controls.rs` for ButtonBase, InputBase/FilledInput/OutlinedInput, FormControl/FormLabel, NativeSelect, ScopedCssBaseline, TextField, and Autocomplete slots.

`runtime_15_ui_asset_mui_web_form_style_tests_are_folder_backed` locks the parent/child layout, prevents the comprehensive form-control test from moving back into the parent, preserves the single moved test, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI asset MUI X web style test folder split

runtime_15_ui_asset_mui_web_mui_x_style_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production MUI X style loading and retained template compilation unchanged and only splits the oversized MUI X asset-style test parent. `ui/tests/asset_mui_web_mui_x_style.rs` now owns the shared MUI X style/layout fixture, module mounting, and find/attr/class helpers; DataGrid coverage lives in `ui/tests/asset_mui_web_mui_x_style/data_grid.rs`; TreeView coverage lives in `ui/tests/asset_mui_web_mui_x_style/tree_view.rs`; Date/Time Picker coverage lives in `ui/tests/asset_mui_web_mui_x_style/date_time_pickers.rs`; chart and gauge coverage lives in `ui/tests/asset_mui_web_mui_x_style/charts.rs`; and Agent Chat coverage lives in `ui/tests/asset_mui_web_mui_x_style/agent_chat.rs`.

`runtime_15_ui_asset_mui_web_mui_x_style_tests_are_folder_backed` locks the parent/child layout, prevents representative DataGrid, TreeView, picker, chart/gauge, and Agent Chat tests from moving back into the parent, preserves the split component-family coverage, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI asset MUI web style test folder split

runtime_15_ui_asset_mui_web_style_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production MUI web style loading and compilation unchanged and only splits the oversized asset-style test parent. `ui/tests/asset_mui_web_style.rs` now owns the shared MUI web style TOML fixture, module mounting, and common attr/class assertion helpers; state, sx, readonly, and icon utility coverage lives in `ui/tests/asset_mui_web_style/state_icons.rs`; slot props and native customization alias coverage lives in `ui/tests/asset_mui_web_style/slots_native.rs`; Alert/Snackbar/Skeleton coverage lives in `ui/tests/asset_mui_web_style/feedback.rs`; Paper/Card/AppBar coverage lives in `ui/tests/asset_mui_web_style/surface.rs`; and data-display selector coverage lives in `ui/tests/asset_mui_web_style/data_display.rs`.

`runtime_15_ui_asset_mui_web_style_tests_are_folder_backed` locks the parent/child layout, prevents representative state/icon, slot/native, feedback, surface, and data-display tests from moving back into the parent, preserves all 9 moved tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI taffy layout pass test folder split

runtime_15_ui_taffy_layout_pass_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production Taffy layout bridge and UI layout pass behavior unchanged and only splits the oversized `ui/tests/taffy_layout_pass.rs` parent. The parent now owns shared layout/tree helpers, template metadata helpers, fallback assertion helpers, and child module mounting; route-report and fallback reason aggregation coverage lives in `ui/tests/taffy_layout_pass/routing_diagnostics.rs`; native linear/wrap/grid arrangement plus template measurement coverage lives in `ui/tests/taffy_layout_pass/arrangement.rs`; linear and wrap slot padding/sizing coverage lives in `ui/tests/taffy_layout_pass/linear_slots.rs`; unsupported policy and Zircon fallback coverage lives in `ui/tests/taffy_layout_pass/fallback_policy.rs`; and grid slot placement/span/alignment coverage lives in `ui/tests/taffy_layout_pass/grid_slots.rs`.

`runtime_15_ui_taffy_layout_pass_tests_are_folder_backed` locks the parent/child layout, prevents representative routing, arrangement, linear-slot, fallback-policy, and grid-slot tests from moving back into the parent, preserves all 35 moved tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI runtime window input pump test folder split

runtime_15_ui_runtime_window_input_pump_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production window input pump behavior unchanged and only splits the oversized runtime window input pump test owner. `ui/tests/runtime_window_input_pump.rs` now owns shared surface fixtures, dispatch helpers, window metadata helpers, popup/tooltip helpers, and child module mounting; lifecycle coverage lives in `ui/tests/runtime_window_input_pump/lifecycle.rs`, pointer hover/cancel/touch coverage lives in `ui/tests/runtime_window_input_pump/pointer_routes.rs`, and metrics plus dirty-domain coverage lives in `ui/tests/runtime_window_input_pump/metrics_dirty.rs`.

`runtime_15_ui_runtime_window_input_pump_tests_are_folder_backed` locks the parent/child layout, prevents representative lifecycle, pointer route, and metrics/dirty-domain tests from moving back into the parent, preserves all 14 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI runtime window event ABI child folder split

runtime_15_ui_runtime_window_event_abi_children_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production runtime window event ABI routing unchanged and only splits the oversized ABI test child owner. `ui/tests/runtime_ui_window_event_routes/abi.rs` now owns only child module mounting; runtime event batch and adapter error coverage lives in `ui/tests/runtime_ui_window_event_routes/abi/batch_adapter.rs`, pointer/wheel/hover/cursor/touch route coverage lives in `ui/tests/runtime_ui_window_event_routes/abi/pointer_window_routes.rs`, and keyboard/gamepad route coverage lives in `ui/tests/runtime_ui_window_event_routes/abi/keyboard_gamepad_routes.rs`.

`runtime_15_ui_runtime_window_event_abi_children_are_folder_backed` locks the parent/child layout, prevents representative batch/adapter, pointer/window, and keyboard/gamepad tests from moving back into the parent, preserves all 13 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI widget text input keyboard test folder split

runtime_15_ui_widget_text_input_keyboard_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production text input dispatch, selection, clipboard, newline, and IME behavior unchanged and only splits the oversized keyboard text-input test owner. `ui/tests/widget_text_input_keyboard.rs` now owns shared dispatch/text/IME/surface fixtures plus child module mounting; basic edit, read-only, grapheme, and delete coverage lives in `ui/tests/widget_text_input_keyboard/basic_editing.rs`; caret movement and selection navigation coverage lives in `ui/tests/widget_text_input_keyboard/selection_navigation.rs`; word movement/deletion/select-all and escape coverage lives in `ui/tests/widget_text_input_keyboard/word_shortcuts.rs`; clipboard plus newline coverage lives in `ui/tests/widget_text_input_keyboard/clipboard_newline.rs`; and text/IME composition coverage lives in `ui/tests/widget_text_input_keyboard/text_ime.rs`.

`runtime_15_ui_widget_text_input_keyboard_tests_are_folder_backed` locks the parent/child layout, prevents representative basic editing, selection/navigation, word shortcut, clipboard/newline, and text/IME tests from moving back into the parent, preserves all 52 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI focus navigation test folder split

runtime_15_ui_focus_navigation_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production focus, navigation, modal, popup, text, and IME routing behavior unchanged and only splits the oversized focus-navigation test owner. `ui/tests/focus_navigation.rs` now owns shared focus/modal/popup/navigation fixtures plus child module mounting; focus state and focused input route coverage lives in `ui/tests/focus_navigation/focus_state.rs`; focusable contract and property mutation coverage lives in `ui/tests/focus_navigation/property_mutation.rs`; tab and directional navigation coverage lives in `ui/tests/focus_navigation/tab_directional.rs`; and modal/popup focus trap coverage lives in `ui/tests/focus_navigation/modal_popup.rs`.

`runtime_15_ui_focus_navigation_tests_are_folder_backed` locks the parent/child layout, prevents representative focus state, property mutation, tab/directional navigation, and modal/popup tests from moving back into the parent, preserves all 16 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Editor UI 08 popup focus owner validation

editor_ui_08_popup_focus_owner_validation_passed

`ui/surface/focus.rs` now treats popup/modal autofocus as an input-owner validation step, not a request to force focus onto the popup root. `open_mui_modal_focus_scope(...)` only returns an autofocus target when a descendant is both focusable and a valid input owner under the current visibility tree. If the popup has no such descendant, the popup opens without stealing focus. If the requested target is invalid, `enforced_mui_modal_focus_target(...)` falls back to the first valid descendant rather than selecting a hidden or collapsed control.

This closes the Workbench module-dispatch failure where inactive/collapsed module controls could expose a focus candidate that later failed owner validation and surfaced as `MissingNode`. The behavior is covered by `ui/tests/focus_navigation/modal_popup.rs`: `widget_popup_without_focusable_descendants_opens_without_stealing_focus` and `widget_popup_under_hidden_ancestor_opens_without_focus_error`, plus the focused `widget_popup` direct binary run.

## Runtime 15 M4 UI template style slot-contract owner split

runtime_15_ui_template_style_slot_contract_owner_split_static_passed_cargo_timeout_no_result

Runtime 15 M4 keeps style plan construction, selector path matching, rule merge order, MUI `sx` precedence, component root class dispatch, selector states, and shared attribute helpers unchanged while splitting slot contract application into a child owner. `ui/template/asset/compiler/style_apply.rs` now owns the style application pipeline and class-dispatch surface; `ui/template/asset/compiler/style_apply/slot_contract.rs` owns root/child slot props merge, slot component/class projection, Skeleton child metadata routing, and owner slot utility class routing across layout, form, selection, collection, MUI X, surface, and navigation families.

`runtime_15_ui_template_style_slot_contract_is_child_owner` locks the parent/child layout, prevents representative slot-contract helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence; the focused core-min Cargo check timed out after 120 seconds with no result and is not counted as passing.

## Runtime 15 M4 UI v2 style runtime-state owner split

runtime_15_ui_v2_style_runtime_state_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps UI v2 style resolver/index entry points, style rule collection, token/theme resolution, selector path DTOs, and selector matching unchanged while splitting runtime-state projection into a child owner. `ui/v2/style.rs` now owns `UiV2StyleResolver`, `UiV2RuntimeStyleIndex`, rule merge/token resolution, selector path construction, and selector matching; `ui/v2/style/runtime_state.rs` owns static/runtime pseudo-state extraction, resolved painter state aliases, retained runtime-state attribute projection, dirty-delta classification, and dirty flag merging.

`runtime_15_ui_v2_style_runtime_state_is_child_owner` locks the parent/child layout, prevents representative runtime-state helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred because external cargo/rustc lanes were already active during the slice.

## Runtime 15 M4 UI v2 style token-resolution owner split

runtime_15_ui_v2_style_token_resolution_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps UI v2 style resolver/index entry points, rule collection, runtime rule merge, selector path DTOs, and selector matching in `ui/v2/style.rs` while splitting token/theme resolution into a child owner. `ui/v2/style/tokens.rs` now owns style block/token source merge, style token path cleanup, recursive document token resolution, theme role normalization, `UiStyleColor` projection, and RGBA hex formatting.

`runtime_15_ui_v2_style_token_resolution_is_child_owner` locks the parent/child layout, prevents token/theme helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M4 UI accessibility extract state owner split

runtime_15_ui_accessibility_extract_state_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps accessibility snapshot traversal, relation-target pruning, name/description resolution, role inference, child filtering, action defaults, bounds, visibility, reference parsing, and diagnostic construction in `ui/accessibility/extract.rs` while splitting state projection into `ui/accessibility/extract/state.rs`. The child owner now owns expanded/open, disabled, selected, pressed, checked, value text, text-selection, component-state conversion, TOML attribute conversion, and byte-offset clamping helpers.

`runtime_15_ui_accessibility_extract_state_is_child_owner` locks the parent/child layout, prevents representative state-projection helpers and `UiValue` conversion from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone implementation cadence.

## Runtime 15 M4 UI component catalog editor-showcase helper owner split

runtime_15_ui_component_catalog_editor_showcase_helper_owner_split_static_passed_cargo_timeout_no_result

Runtime 15 M4 keeps the editor showcase registry, descriptor list, descriptor assembly entry point, and representative component catalog coverage in `ui/component/catalog/editor_showcase.rs` while splitting reusable descriptor construction into `ui/component/catalog/editor_showcase/descriptor_builders.rs`. The child owner now owns base descriptor setup, layout-role/default-template projection, palette metadata, fallback policy, option/slot/value prop schema builders, and TOML layout helper construction; the original M4 `helpers.rs` owner name was later hard-cut to the responsibility name by the Runtime 15 M2 naming slice.

`runtime_15_ui_component_catalog_editor_showcase_helpers_are_child_owner` locks the parent/child layout, prevents descriptor helper and palette/fallback construction from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; focused Cargo timed out after 305 seconds with no diagnostic result and is not counted as passing.

## Runtime 15 M2 UI editor showcase descriptor builders module naming hard cutover

runtime_15_ui_editor_showcase_descriptor_builders_naming_hard_cutover_static_passed_cargo_deferred

Runtime 15 M2 deletes the retired `ui/component/catalog/editor_showcase/helpers.rs` owner name and hard-cuts the descriptor construction owner to `ui/component/catalog/editor_showcase/descriptor_builders.rs`. `ui/component/catalog/editor_showcase.rs` now mounts only `mod descriptor_builders;` and imports descriptor construction, layout role/default template projection, palette metadata, fallback policy, option/slot/value prop schema builders, and TOML layout helpers from the responsibility-named owner.

`runtime_15_ui_editor_showcase_descriptor_builders_use_owner_name` locks the old file absence, the new owner/module entry/caller import shape, the synchronized M4 production-budget guard path, and the Runtime 15/status/docs mirrors. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone implementation cadence.

## Runtime 15 M4 UI surface event-routing owner split

runtime_15_ui_surface_event_routing_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps `UiSurface` state, runtime-state style entry points, frame/debug snapshots, property mutation, reflector snapshots, and focus-path queries in `ui/surface/surface.rs` while splitting event flow into two folder-backed child owners. `ui/surface/surface/event_routing.rs` now owns pointer capture/release, input/window dispatch adapters, pointer route construction, pointer dispatch side effects, navigation routing, and activation-phase derivation. `ui/surface/surface/pointer_component_events.rs` owns route-derived hovered/pressed/focused component-state dirtying, component event reports, focus event reports, and requested damage frames.

`runtime_15_ui_surface_event_routing_is_child_owner` locks the parent/child layout, prevents event-routing helpers and pointer component event/report generation from moving back into the parent, keeps all three production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.


## Runtime 15 M4 UI surface property mutation metadata dirty owner split

runtime_15_ui_surface_property_mutation_metadata_dirty_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps property mutation request/report construction, the `mutate_tree_property(...)` entry point, visibility/input/state mutation, binding report construction, template attribute sync, visibility/input value parsing, and state dirty marking in `ui/surface/property_mutation.rs` while splitting metadata dirty-domain classification into a child owner. `ui/surface/property_mutation/metadata_dirty.rs` now owns metadata attribute dirty classification, render and virtualized dirty helpers, MUI overlay/feedback/transition/customization predicates, virtualized range predicates, and layout metadata predicates.

`runtime_15_ui_surface_property_mutation_metadata_dirty_is_child_owner` locks the parent/child layout, prevents metadata dirty helpers from moving back into the parent, and keeps both production owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.
## Runtime 15 M4 UI surface render feedback command/color owner split

runtime_15_ui_surface_render_feedback_command_color_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps feedback component-kind detection, Alert/AlertTitle/Tooltip/Toast command layout, metadata text/icon/size extraction, border/radius parsing, and public render entry points in `ui/surface/render/feedback.rs` while splitting reusable color and primitive-command construction into child owners. `ui/surface/render/feedback/colors.rs` now owns AlertTone, feedback color constants, visual-state-aware alert/tooltip/toast color selection, and style override fallback. `ui/surface/render/feedback/commands.rs` owns quad/text/icon `UiRenderCommand` DTO construction. `ui/surface/render/feedback/state.rs` continues to own painter family state resolution.

`runtime_15_ui_surface_render_feedback_commands_are_child_owners` locks the parent/child layout, prevents color constants, color helpers, and primitive command constructors from moving back into the parent, keeps all four feedback render owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.

## Runtime 15 M4 UI surface default-interactions keyboard/timer owner split

runtime_15_ui_surface_default_interactions_keyboard_timer_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps pointer default-action routing, button/toggle/disclosure helpers, shared binding report construction, widget behavior predicates, and component event token matching in `ui/surface/surface/default_interactions.rs` while splitting keyboard and timer semantics into two child owners. `ui/surface/surface/default_interactions/keyboard.rs` now owns keyboard-triggered default component actions, semantic keyboard actions/text, keyboard behavior eligibility, and semantic action/event-kind mapping. `ui/surface/surface/default_interactions/timers.rs` owns typeahead timeout, submenu hover delay, tooltip timer derivation, menu-role detection, tooltip id extraction, and timer-expired component event report construction.

`runtime_15_ui_surface_default_interactions_keyboard_timers_are_child_owners` locks the parent/child layout, prevents keyboard and timer helpers from moving back into the parent, keeps all three production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.

## Runtime 15 M4 UI surface table column helper owner split

runtime_15_ui_surface_table_column_helper_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps table pointer routing, column resize/sort event flow, row selection dispatch, virtual scroll dispatch, table mutation, and shared owner predicates in `ui/surface/surface/default_interactions/table/mod.rs` while splitting reusable column helpers into `ui/surface/surface/default_interactions/table/columns.rs`. The child owner now owns column resize/sort metadata classification, column field/width/min-width lookup, sort direction/model helpers, row sort comparison, column matching, and resize drag-token encoding/decoding.

`runtime_15_ui_surface_table_column_helpers_are_child_owner` locks the parent/child layout, prevents column helper constants and helper functions from moving back into the parent, keeps `mod.rs`, `columns.rs`, `selection.rs`, and `virtualization.rs` under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo is deferred while external cargo/rustc lanes remain active.

## Runtime 15 M2 UI table sortingMode server literal allowed-context sync

runtime_15_ui_table_sorting_mode_server_literal_allowed_context_static_passed_cargo_deferred

Runtime 15 M2 keeps the DataGrid/Table API literal `sortingMode = "server"` as a non-network server naming allowed context after the M4 table column helper split moved the production read to `ui/surface/surface/default_interactions/table/columns.rs`. The table behavior stays unchanged: `table_uses_client_sorting(...)` treats the external UI API value as the signal to skip local row sorting.

`runtime_15_ui_table_sorting_mode_server_literal_stays_allowed_context` locks the columns owner, the Python `non_network_server_naming.py` allowlist, the Rust `runtime_non_network_server_naming_is_classified_by_owner` guard, and the Runtime 15/status/docs mirrors. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone implementation cadence.

## Runtime 15 M4 UI template document validation owner split

runtime_15_ui_template_document_validation_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps `UiAssetDocumentRuntimeExt`, style rule/sheet mutation semantics, node traversal order, child mount lookup, parent lookup, and tree mutation APIs unchanged while splitting document validation into a child owner. `ui/template/asset/document.rs` now owns the runtime extension trait implementation and tree/style mutation entry points; `ui/template/asset/document/validation.rs` owns node id authority checks, duplicate subtree validation, stylesheet/style-rule id checks, and selector parse validation.

`runtime_15_ui_template_document_validation_is_child_owner` locks the parent/child layout, prevents representative validation helpers and `UiSelector::parse` from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M4 UI template MUI X DataGrid class owner split

runtime_15_ui_template_mui_x_data_grid_class_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps template style application order, selector matching, slot-props projection, generic MUI class suppression, and non-DataGrid MUI X component classes unchanged while splitting the oversized MUI X class owner. `ui/template/asset/compiler/style_apply/mui_x_classes.rs` now owns component-family dispatch plus MaterialTreeView, Date/Time Pickers, Charts, AgentChat, and Chat component class routing; `ui/template/asset/compiler/style_apply/mui_x_classes/data_grid.rs` owns DataGrid root utility classes and DataGrid slot utility class projection for column headers, rows, cells, toolbar/footer, and overlay slots.

`runtime_15_ui_template_mui_x_data_grid_classes_are_child_owner` locks the parent/child layout, prevents representative DataGrid helpers from moving back into the parent, keeps both production owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M4 UI text layout engine visual-order owner split

runtime_15_ui_text_layout_engine_visual_order_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps the production owner split: `ui/text/layout_engine.rs` owns crate-internal layout entry points, source-run wrapping, ellipsis construction, alignment, and test-module mount; `ui/text/layout_engine/visual_order.rs` owns the visual token/cluster/fragment adapter. The 2026-07-10 Text 02/03 hard cut replaced that child's former low-fidelity ASCII/RTL-block and neutral-direction scaffold with the shared `graphics/text/shaping/bidi.rs` UAX#9 paragraph/line owner. Auto/Mixed direction, isolate levels, post-wrap L1/L2 ordering, and odd-level mirroring now derive from one owner while logical source ranges remain intact.

`runtime_15_ui_text_layout_engine_visual_order_is_child_owner` locks the parent/child layout, prevents representative adapter helpers from moving back into the parent, requires the shared UAX#9 calls, and keeps both production owners under the Runtime 15 file budget. The hard cut passed an exact-owner 3/3 harness and runtime library check; full monolithic lib-test acceptance remains a later milestone gate.

## Runtime 15 M4 UI layout arrange grid/masonry owner split

runtime_15_ui_layout_arrange_grid_masonry_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps layout arrange entry points, Taffy fallback routing, non-grid Zircon fallback families, scroll virtualization window planning, wrap content sizing, and subtree hiding unchanged while splitting GridBox and MasonryBox fallback arrangement into a child owner. `ui/layout/pass/arrange.rs` now owns the arrange dispatcher plus Free/Canvas/Container/Overlay, SizeBox, BlockBox, linear, ScrollableBox, and WrapBox flow; `ui/layout/pass/arrange/grid_masonry.rs` owns GridBox placement, dimension and cell-frame helpers, MasonryBox column selection and content-size computation, masonry child outer-height lookup, and grid/masonry recursive child arrangement.

`runtime_15_ui_layout_arrange_grid_masonry_is_child_owner` locks the parent/child layout, prevents grid and masonry helpers from moving back into the parent, and keeps both production owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M4 UI dispatch input manager test owner split

runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred

Runtime 15 M4 keeps `UiInputManager` routing, window input pump aggregation, pointer table exposure, and component timer behavior unchanged while splitting the inline test owner. `ui/dispatch/input_manager/manager.rs` now owns the production dispatch API, timer arming/drain logic, active pointer helpers, and timestamp helpers; `ui/dispatch/input_manager/manager/tests.rs` owns submenu hover, popup typeahead, Toast auto-hide, Tooltip timer coverage, and the test-only surface fixtures.

`runtime_15_ui_dispatch_input_manager_tests_are_child_owner` locks the parent/child layout, prevents the seven moved tests from moving back into `manager.rs`, keeps both owners under the Runtime 15 file budget, and records the status-output expectation in the Runtime 15 M4 row data. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M3 UI runtime input manager test folder split

runtime_15_ui_runtime_input_manager_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production input manager routing, timer, pointer, touch, popup, and focus behavior unchanged and only splits the oversized runtime input manager test owner. `ui/tests/runtime_input_manager.rs` now owns shared route matrix, double-click, popup, input metadata, pointer/touch/keyboard/popup event, and window metadata fixtures plus child module mounting; window batch and tick coverage lives in `ui/tests/runtime_input_manager/window_timer.rs`; route order and policy naming coverage lives in `ui/tests/runtime_input_manager/route_order.rs`; capture, popup stack, preview, focus path, and default-action route matrix coverage lives in `ui/tests/runtime_input_manager/route_matrix.rs`; and double-click, touch synthesis/cancel, and multi-pointer capture isolation coverage lives in `ui/tests/runtime_input_manager/touch_pointer.rs`.

`runtime_15_ui_runtime_input_manager_tests_are_folder_backed` locks the parent/child layout, prevents representative window/timer, route-order, route-matrix, double-click/touch, and multi-pointer tests from moving back into the parent, preserves all 15 migrated parent tests, and keeps every owner touched by this split under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI runtime input ownership test folder split

runtime_15_ui_runtime_input_ownership_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production input ownership, pointer capture, high-precision dispatch, drag/drop, popup, tooltip, and input-method behavior unchanged and only splits the oversized runtime input ownership test owner. `ui/tests/runtime_input_ownership.rs` now owns shared pointer capture, input event, drag/drop, popup/tooltip, and input-method fixtures plus child module mounting; input-method ownership coverage lives in `ui/tests/runtime_input_ownership/input_method.rs`; hidden/disabled owner validation coverage lives in `ui/tests/runtime_input_ownership/owner_validation.rs`; high-precision dispatch and reply-step coverage lives in `ui/tests/runtime_input_ownership/high_precision_dispatch.rs`; drag/drop lifecycle coverage lives in `ui/tests/runtime_input_ownership/drag_drop.rs`; popup/tooltip transient input coverage lives in `ui/tests/runtime_input_ownership/popup_tooltip.rs`; and route trace plus analog suppression coverage lives in `ui/tests/runtime_input_ownership/route_trace.rs`.

`runtime_15_ui_runtime_input_ownership_tests_are_folder_backed` locks the parent/child layout, prevents representative input-method, owner validation, high-precision dispatch, drag/drop, popup/tooltip, and route-trace tests from moving back into the parent, preserves all 16 migrated parent tests, keeps every owner touched by this split under the Runtime 15 file budget, and preserves the per-pointer capture API assertions. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI event routing test folder split

runtime_15_ui_event_routing_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps the production UI dispatch and surface routing owners unchanged and only splits the oversized event-routing test owner. `ui/tests/event_routing.rs` now owns shared fixtures, helper construction, and child module mounting; behavior tests live in `ui/tests/event_routing/pointer_state.rs`, `ui/tests/event_routing/component_events.rs`, `ui/tests/event_routing/dispatch_effects.rs`, and `ui/tests/event_routing/shared_input.rs`.

`runtime_15_ui_event_routing_tests_are_folder_backed` locks the parent/child layout, prevents representative pointer/component/dispatch/shared-input tests from moving back into the parent, preserves all 27 event-routing tests, and keeps every UI event-routing test owner under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI shared core input visibility child folder split

runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps shared-core input, visibility, focus, hit-testing, and layout behavior unchanged and only splits the near-budget input visibility test owner. `ui/tests/shared_core/input_visibility.rs` now owns only child module mounting; pointer dispatch/capture coverage lives in `ui/tests/shared_core/input_visibility/pointer_routes.rs`, hit-testing/render/hit-grid visibility coverage lives in `ui/tests/shared_core/input_visibility/hit_visibility.rs`, collapsed visibility layout coverage lives in `ui/tests/shared_core/input_visibility/collapsed_layout.rs`, and focus/scroll candidate visibility coverage lives in `ui/tests/shared_core/input_visibility/focus_candidates.rs`.

`runtime_15_ui_shared_core_input_visibility_children_are_folder_backed` locks the input visibility parent/child layout, prevents representative pointer, hit-visibility, collapsed-layout, and focus-candidate tests from moving back into the parent, preserves all 9 input visibility tests, and keeps the parent plus all four child owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M3 UI shared core scroll mutation child folder split

runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps shared-core scroll, pointer dispatch, property mutation, template metadata, and reflection behavior unchanged and only splits the near-budget scroll mutation test owner. `ui/tests/shared_core/scroll_mutation.rs` now owns only child module mounting; virtual window and scroll layout invalidation coverage lives in `ui/tests/shared_core/scroll_mutation/virtual_scroll.rs`, pointer block/passthrough/capture and scroll-wheel route coverage lives in `ui/tests/shared_core/scroll_mutation/pointer_routes.rs`, and runtime property mutation plus reflector snapshot coverage lives in `ui/tests/shared_core/scroll_mutation/property_mutation.rs`.

`runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed` locks the scroll mutation parent/child layout, prevents representative virtual-scroll, pointer-route, and property-mutation tests from moving back into the parent, preserves all 10 scroll mutation tests, and keeps the parent plus all three child owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M3 UI shared core layout surface child folder split

runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps shared-core layout, render extract, overlay, and spacer behavior unchanged and only splits the near-budget layout surface test owner. `ui/tests/shared_core/layout_surface.rs` now owns only child module mounting; axis solving, layout invalidation, intrinsic measurement, and free-layout container coverage lives in `ui/tests/shared_core/layout_surface/layout_measurement.rs`, visual command extraction coverage lives in `ui/tests/shared_core/layout_surface/render_extract.rs`, and overlay plus spacer coverage lives in `ui/tests/shared_core/layout_surface/container_overlays.rs`.

`runtime_15_ui_shared_core_layout_surface_children_are_folder_backed` locks the layout surface parent/child layout, prevents representative layout, render-extract, and overlay tests from moving back into the parent, preserves all 11 layout surface tests, and keeps the parent plus all three child owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

## Runtime 15 M3 UI shared core guard child-owner split

runtime_15_ui_shared_core_guard_child_owner_split_static_passed_cargo_deferred

Runtime 15 M3 keeps UI shared-core runtime behavior unchanged and only splits the structure-convention guard owner. `structure_convention/test_file_budget/ui_shared_core.rs` now owns child module mounting plus `runtime_15_ui_shared_core_guard_child_owners_are_folder_backed`; the existing root, layout-surface, input-visibility, and scroll-mutation guard checks live in `structure_convention/test_file_budget/ui_shared_core/root.rs`, `structure_convention/test_file_budget/ui_shared_core/layout_surface.rs`, `structure_convention/test_file_budget/ui_shared_core/input_visibility.rs`, and `structure_convention/test_file_budget/ui_shared_core/scroll_mutation.rs`.

The new guard locks parent/child ownership, prevents moved guard definitions from returning to the parent, and keeps each focused guard owner below the 400-line budget. This is static structure evidence only; Cargo remains deferred while external cargo lanes remain active.

## Runtime 15 M3 UI runtime input reply routes test folder split

runtime_15_ui_runtime_input_reply_routes_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production input reply routing unchanged and only splits the oversized runtime input reply route parent test owner. `ui/tests/runtime_input_reply_routes.rs` now owns shared fixtures, input event helpers, and child module mounting; the parent-owned tests moved into `ui/tests/runtime_input_reply_routes/route_trace_routes.rs`, `ui/tests/runtime_input_reply_routes/pointer_bubble_routes.rs`, and `ui/tests/runtime_input_reply_routes/focus_text_accessibility_routes.rs`.

`runtime_15_ui_runtime_input_reply_routes_tests_are_folder_backed` locks the parent/child layout, prevents representative route-trace/pointer-bubble/focus-text/accessibility tests from moving back into the parent, preserves all 13 moved parent tests, and keeps the parent plus the three new child owners under the Runtime 15 file budget. Existing `keyboard_navigation_routes.rs` and `tree_view_pointer_routes.rs` remain separate oversized child-owner work. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI runtime input reply route child folder split

runtime_15_ui_runtime_input_reply_route_children_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production input reply routing unchanged and splits only the oversized reply-route child test owners. `ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs` now owns keyboard navigation shared fixtures plus child module mounting; focus path, semantic action route, timers-disabled, and directional navigation tests live in `keyboard_navigation_routes/focus_path.rs`, `keyboard_navigation_routes/semantic_actions.rs`, `keyboard_navigation_routes/timers_disabled.rs`, and `keyboard_navigation_routes/directional.rs`. `ui/tests/runtime_input_reply_routes/tree_view_pointer_routes.rs` now owns tree-view shared fixtures plus child module mounting; selection, drag/reorder, and virtualization tests live in `tree_view_pointer_routes/selection.rs`, `tree_view_pointer_routes/drag_reorder.rs`, and `tree_view_pointer_routes/virtualization.rs`.

`runtime_15_ui_runtime_input_reply_route_children_are_folder_backed` locks the two child-parent layouts, prevents representative keyboard/tree tests from moving back into their parent files, preserves all 15 keyboard-navigation tests and all 9 tree-view pointer tests, and keeps both parents plus all seven new child owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI runtime input reply table pointer route folder split

## Runtime 15 M3 UI runtime input reply route guard child-owner split

runtime_15_ui_runtime_input_reply_route_guard_child_owner_split_static_passed_cargo_deferred

Runtime 15 M3 keeps UI runtime input reply behavior unchanged and only splits the structure-convention guard owner. `structure_convention/test_file_budget/ui_runtime_input_reply_routes.rs` now owns child module mounting plus `runtime_15_ui_runtime_input_reply_route_guard_child_owners_are_folder_backed`; the existing root route, keyboard/tree child route, and table pointer guard checks live in `structure_convention/test_file_budget/ui_runtime_input_reply_routes/root.rs`, `structure_convention/test_file_budget/ui_runtime_input_reply_routes/route_children.rs`, and `structure_convention/test_file_budget/ui_runtime_input_reply_routes/table_pointer.rs`.

The new guard locks parent/child ownership, prevents moved guard definitions from returning to the parent, and keeps each focused guard owner below the 400-line budget. This is static structure evidence only; Cargo remains deferred while external cargo/rustc lanes remain active.

runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production input reply routing unchanged and splits only the table pointer route test owner. `ui/tests/runtime_input_reply_routes/table_pointer_routes.rs` now owns table pointer shared fixtures, input event helpers, table metadata assertions, and child module mounting; column resize, sorting, row selection, and virtualization tests live in `ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs`, `ui/tests/runtime_input_reply_routes/table_pointer_routes/sorting.rs`, `ui/tests/runtime_input_reply_routes/table_pointer_routes/selection.rs`, and `ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs`.

`runtime_15_ui_runtime_input_reply_table_pointer_routes_are_folder_backed` locks the table pointer parent/child layout, prevents representative resize/sort/selection/virtualization tests from moving back into the parent file, preserves all 11 table pointer route tests, and keeps the parent plus all four child owners under the Runtime 15 file budget. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## Runtime 15 M3 UI architecture test folder split

runtime_15_ui_architecture_tests_folder_split_static_passed_cargo_deferred

Runtime 15 M3 keeps production UI architecture code unchanged and splits only the oversized Runtime 09 absorption guard owner. `tests/runtime_absorption/ui_architecture.rs` now owns shared repository-scan helpers and child module mounting; architecture-boundary guards live in `tests/runtime_absorption/ui_architecture/architecture_boundaries.rs`, legacy/debt rename guards live in `tests/runtime_absorption/ui_architecture/legacy_renames.rs`, and the mirror-doc audit guard lives in `tests/runtime_absorption/ui_architecture/mirror_docs.rs`.

`runtime_15_ui_architecture_tests_are_folder_backed` locks the parent/child layout, prevents representative architecture/legacy/mirror tests from moving back into the parent, preserves all 18 Runtime 09 UI architecture absorption guards, and keeps the parent plus all three child owners under the Runtime 15 file budget. `ui_architecture_boundary.py` also reads the three child guard sources so the existing Runtime 09 mirror audit remains aware of the folder-backed layout. This is static structure evidence only; Cargo remains deferred by the Runtime 15 milestone testing cadence.

## M1.2 Layout Engine Backend Name Cutover

runtime_09_m1_2_layout_engine_backend_name_cutover_static_passed_cargo_pending

The layout engine contract now names the runtime fallback backend directly as `UiLayoutEngineBackend::Zircon`. The public capability constructor is `UiLayoutEngineCapability::zircon()`, and route reports expose `zircon_selected_count`. `zircon_runtime/src/ui/layout/pass/engine.rs` consumes those names directly when recording Taffy-native routes, explicit fallbacks, and Zircon-owned routes.

`runtime_09_layout_engine_backend_name_cutover_reduces_ui_layout_debt` keeps the old `LegacyZircon`, `legacy_zircon`, and `legacy_selected_count` API names from returning. This is a hard cutover, not a compatibility alias layer.

## M1.2 Surface Default Interaction Fallback Naming

runtime_09_m1_2_surface_default_interaction_fallback_renamed_static_passed_cargo_pending

The surface default interaction open-state helper now names its compatibility property list `fallback_properties` in `surface/surface/default_interactions.rs`. `default_open_boolean_value(...)` still resolves authored metadata first, then the same component-state property, then retained/component-state fallback aliases, and finally the canonical runtime open flag before applying the supplied default.

`runtime_09_surface_default_interaction_fallback_rename_reduces_ui_surface_debt` keeps the old `legacy_properties` and `legacy_property` local wording from returning. This closes the Runtime 09 production UI `legacy` scan bucket without changing default interaction behavior.

## M2.1 Layout Backend Authority

runtime_09_m2_1_taffy_bridge_pass_order_static_passed_cargo_pending

`zircon_runtime::ui::layout::taffy_bridge` is now folder-backed. `taffy_bridge/compute.rs` owns Taffy tree construction, fractional rounding policy, `TaffyTree::new()`, and `compute_layout` through `compute_taffy_child_frames(...)`. `layout/pass/taffy_arrange.rs` no longer imports `taffy::`, creates a `TaffyTree`, or computes layout directly; it builds `TaffyChildLayoutInput` records, delegates backend computation to the bridge, then records native/fallback selection and recurses into `arrange_node(...)`.

`UI_LAYOUT_PASS_ORDER` lives in `layout/pass/pipeline.rs` and is consumed by both full and incremental layout. The order is: responsive style resolution, measurement, backend selection, Taffy bridge arrangement, Zircon fallback arrangement, clip and virtual-window propagation, selection report. Incremental layout now measures all selected layout roots before arranging them, matching the same phase split as full layout.

runtime_09_m2_1_style_mapping_remains_taffy_dto_adapter

`layout/style_mapping.rs` still mentions Taffy because it converts neutral Zircon `UiLayoutStyle` / container DTOs into Taffy style DTOs. That file is not a backend executor and does not own tree construction or pass sequencing. This explicit adapter verdict replaces the older "only `taffy_bridge.rs` may mention Taffy" wording with the stronger runtime boundary: Taffy compute is only reachable through `compute_taffy_child_frames`, while style DTO conversion remains isolated.

## M2.2 Virtualization Scroll Boundary

runtime_09_m2_2_virtualization_scroll_boundary_static_passed_cargo_pending

`layout/scroll.rs` now owns `UiScrollVirtualizationPlan` and `plan_scrollable_virtual_window(...)`. The planner clamps the requested scroll offset against the current content/viewport extents, computes the scroll state's `offset` / `viewport_extent` / `content_extent`, projects the virtual window, and reports whether `visible_range` must be invalidated. `layout/virtualization.rs::compute_virtual_list_window(...)` stays pure arithmetic over offset, viewport extent, item extent, item count, and overscan.

Invalidation timing is now explicit:

- Scroll offset changes recompute the virtual window. `tree/node/scroll.rs::set_scroll_offset(...)` consumes the planner, marks layout/hit/render/input dirty, and ORs `node.dirty.visible_range |= plan.visible_range_changed` so an earlier visible-range dirty mark cannot be cleared by a later scroll operation.
- Viewport and content changes are settled in `layout/pass/arrange.rs`, which consumes the same planner after measurement has produced `content_size` and the parent frame has produced `viewport_extent`. The resulting full or virtual window is cached in `node.layout_cache.virtual_window`.
- Data changes that alter child count or measured content extent flow through layout dirtiness into the arrange pass. Virtualized scroll containers mark `visible_range` when the planned window, viewport extent, or content extent changes. Non-virtualized scroll containers keep a full-window cache and do not report `visible_range` changes for ordinary scroll offset movement.

Behavior coverage is recorded by `scroll_virtualization.rs`: `virtualized_list_only_materializes_visible_window`, `scroll_offset_invalidates_virtualization_window`, and `non_virtualized_scroll_offset_keeps_full_window_dirty_domain`. Package behavior filters are still deferred by the current implementation-first request, but the tests are present and wired into `ui/tests/mod.rs`.

## M3.1 Template Pipeline Boundary

runtime_09_m3_1_template_compile_instance_validate_boundary_static_passed_cargo_pending

The old recursive template path now has an explicit runtime entry in `UiTemplateRuntimePipeline`. `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES` fixes the phase order as:

```text
load -> validate -> instance -> build
```

The stage owners are intentionally narrow. `UiTemplateLoader` owns TOML parse and file IO, `UiTemplateValidator` owns structural contract validation, `UiTemplateInstance::from_validated_document(...)` owns already-validated expansion, and `UiTemplateSurfaceBuilder` owns lazy `UiSurface` construction. `UiTemplateRuntimePipelineError` keeps those phases observable as `Load`, `Validate`, `Instance`, and `Build`, so callers and tests can tell which boundary rejected the document.

The M3.1 failure-path anchors are present in `template_pipeline.rs`: `template_validate_rejects_unknown_component_contract`, `template_instance_failure_surfaces_loader_error`, and `compiled_template_artifact_stays_binary_leaf_dto_not_generated_source`. Broader package behavior execution remains deferred by the current implementation-first request.

Generated output policy is tied to Runtime 02 M4. `UiRuntimeCompiledAssetArtifact::generated_policy()` records `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source`; current compiled template package output is a binary/TOML DTO payload and does not require a generated-source marker. If a future template compiler writes source files, the first line must be `// @generated <generator> - do not edit by hand`, and the generated file may only contain leaf DTO/table/adaptor material, not runtime validation, loading, expansion, or surface mutation behavior.

## V2 Verdict

v2-replacement-mainline

The v2 UI path is the replacement mainline for authored runtime/editor UI assets, not dead code and not a second unconstrained runtime contract.

Runtime 10 M2.2 now mirrors this verdict through `runtime_10_m2_2_ui_v2_contract_sync_static_passed_cargo_pending`: interface `ui/v2` owns the v2 asset/compiled/style DTOs, runtime `ui/v2` consumes those DTOs for loader/cache/compiler/instancer behavior, and `UiComponentApiVersion` remains the interface-owned component contract version. Runtime validation continues to report `UiComponentContractDiagnosticCode::ApiMismatch` via `actual.is_compatible_with(required)`; the current Runtime 10 structural mirror records `ui_v2_contract_sync_anchors = 9/9` and keeps `v2-replacement-mainline` mutually linked with Runtime 09.

The source-profile split is:

- `.zui` is the production component asset suffix for imported project/editor component assets.
- `.v2.ui.toml` remains valid for v2 view/style/runtime fixture/editor chrome assets that are loaded through the v2 cache path, but it is not the general production component importer path.
- recursive `UiTemplateNode` and old template document paths are legacy/migration/test-only surfaces until M3 proves their remaining owners and deletion conditions.

Current evidence:

- `zircon_runtime_interface::ui::v2` contains the neutral v2 DTO schema.
- `zircon_runtime::ui::v2` contains the runtime loader/compiler/cache/style/surface-builder/surface-tree implementation.
- Runtime UI fixture/manager test-support docs describe `UiV2PrototypeStoreFileCache -> UiV2SurfaceBuilder -> surface_tree -> UiSurface`.
- Editor view/chrome assets and runtime fixtures already consume v2 assets.
- Asset importer/plugin registry docs enforce `.zui` for production component assets while keeping explicit v2 fixture/view paths.

Migration route:

1. Keep `zircon_runtime_interface::ui::v2` as the neutral contract layer.
2. Keep `zircon_runtime::ui::v2` as the sole runtime implementation layer for v2 cache, compilation, style resolution, and surface construction.
3. Keep editor usage on asset/cache/projection APIs; editor code may not grow a parallel runtime builder.
4. Delete or isolate old recursive template paths only after M3 records owner files, generated-file rules, fixture exceptions, and failure-path tests.

Deletion conditions for non-v2 runtime paths:

- production project/editor component importers do not accept old recursive `.ui.toml` component documents;
- component catalog and editor shell projection use v2 assets or `.zui` component assets;
- runtime fixtures and test-support preview manager have no fallback through `UiTemplateTreeBuilder` or old `UiTemplateSurfaceBuilder`;
- migration fixtures are named and isolated from production asset registration;
- template compile/instance/validate failure paths have explicit tests and generated output markers where they write files.

## Static Acceptance

This Runtime 09 record contains the M0 documentation/status pass, the M1.1 normalized input route authority note, the M1.2 local navigation, pointer reply, pointer-capture fallback, table row-label fallback, template component-name fallback, property visibility flag, responsive MUI visibility flag, accessibility open-state fallback, layout engine backend name, and surface default interaction fallback cutovers, the M2.1 Taffy bridge/pass-order authority cutover, the M2.2 virtualization/scroll boundary implementation, and the M3.1 template pipeline/generated-policy boundary. Package Cargo was run only as focused static/type checks in this lane; full UI behavior filters are still deferred per the current implementation-first request. The accepted static evidence is:

- current owner map covers all 18 scanned UI top-level entries;
- current `surface/` scan is recorded as 20 entries rather than the stale 2026-06-12 value;
- `legacy` full-tree and production-file baselines are recorded separately after the Runtime 09 M1.2 navigation, pointer reply, pointer-capture fallback, table row-label fallback, template component-name fallback, property visibility flag, responsive MUI visibility flag, accessibility open-state fallback, layout engine backend name, and surface default interaction fallback cutovers;
- `taffy` production-file baseline is refreshed after M2.1 to record the bridge-directory and pass-order owner shape;
- `UiScrollVirtualizationPlan` and `plan_scrollable_virtual_window(...)` are recorded as the Runtime 09 M2.2 owner boundary for scroll offset, viewport/content extent, and virtual-window invalidation;
- `UiTemplateRuntimePipeline`, `UI_TEMPLATE_RUNTIME_PIPELINE_STAGES`, and `UiTemplateRuntimePipelineError` are recorded as the Runtime 09 M3.1 template load/validate/instance/build boundary;
- `runtime_09_m3_1_binary_leaf_dto_artifact_not_generated_source` records that current compiled template artifacts are binary DTO payloads rather than generated source; future generated source must use `// @generated <generator> - do not edit by hand`;
- v2 is explicitly classified as replacement mainline with a source-profile split and deletion conditions.
- `runtime_absorption::ui_architecture` now guards the module count, baseline scan values, v2 runtime/interface module shape, the route authority note, the direct pointer/navigation owner verdict, the navigation reply rename, the pointer reply rename, the pointer-capture fallback rename, the table row-label fallback rename, the template component-name fallback rename, the property visibility flag rename, the responsive MUI visibility flag rename, the accessibility open-state fallback rename, the layout engine backend name cutover, the surface default interaction fallback rename, the Taffy bridge/pass-order authority, the virtualization/scroll invalidation planner, the template pipeline/generated-policy boundary, and the plan/index anchors.
- 2026-07-01 Runtime 09 UI entry map audit sync records `expected_ui_entry_count = 19` and `expected_surface_entry_count = 21`: `ui/platform_input/` is a current top-level runtime UI owner, `ui/surface/property_mutation/` is the current surface child owner beside `property_mutation.rs`, `has_pointer_capture_or_unindexed_fallback_for_owner` remains the mirror doc anchor for the pointer-capture fallback cutover, and `ui_architecture_boundary` reports `missing_doc_anchors = []` with `risks = []`. This only syncs static structure evidence; behavior and Cargo gates remain deferred.
- 2026-07-10 Runtime Text adds two narrow direct surface leaves: `ui/surface/text_geometry.rs` owns shaped caret/range frames and `ui/surface/text_shape.rs` owns direct shaped-line projection. The Runtime 09 surface entry map is therefore 23; these leaves reuse shared text/graphics owners and do not create another UI architecture path.
- `ui_architecture_boundary` mirrors the same static facts while `ui_architecture_markdown.py` owns Markdown rendering: `ui_architecture_boundary.py` remains the 541-line audit/risk owner, `ui_architecture_markdown.py` is the 110-line renderer, `expected_source_file_count = 52`, `expected_ui_entry_count = 18`, `expected_surface_entry_count = 20`, `legacy_full_hits = 54`, `expected_legacy_full_hits = 54`, `legacy_production_hits = 0`, `expected_legacy_production_hits = 0`, `legacy_production_file_count = 0`, `expected_legacy_production_file_count = 0`, `taffy_production_hits = 175`, `expected_taffy_production_hits = 175`, `taffy_production_file_count = 10`, `expected_taffy_production_file_count = 10`, `runtime_v2_anchor_count = 10`, `interface_v2_anchor_count = 9`, `guard_anchor_count = 19`, `cargo_gate_anchor_count = 7`, `doc_anchor_count = 61`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_09_ui_architecture_mirror_docs_match_structure_audit_counts` keeps this document aligned with Runtime 09, the runtime index, the M0 review, runtime-interface convergence, and the Python audit. This is static structure evidence only.
- `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation` keeps Runtime 09 on the `ui/input/naming_boundary/layout/template` owner/Cargo gate until editor UI owner coordination and the declared Cargo filters provide real evidence.

## 2026-07-01 Runtime 15 UI Structure Mirror Follow-Up

Current Runtime 15 UI guard anchors are mirrored here for the structure sweep: `Runtime 15 M4 UI dispatch input manager test owner split`, `runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred`, `ui/dispatch/input_manager/manager/tests.rs`, `runtime_15_ui_dispatch_input_manager_tests_are_child_owner`, `Runtime 15 M4 UI v2 style token-resolution owner split`, `runtime_15_ui_v2_style_token_resolution_owner_split_static_passed_cargo_deferred`, `ui/v2/style/tokens.rs`, `runtime_15_ui_v2_style_token_resolution_is_child_owner`, `Runtime 15 M3 UI asset MUI web form style test folder split`, `runtime_15_ui_asset_mui_web_form_style_tests_folder_split_static_passed_cargo_deferred`, `ui/tests/asset_mui_web_form_style.rs`, `ui/tests/asset_mui_web_form_style/form_controls.rs`, `runtime_15_ui_asset_mui_web_form_style_tests_are_folder_backed`, `Runtime 15 M3 UI asset surface index test folder split`, `runtime_15_ui_asset_surface_index_tests_folder_split_static_passed_cargo_deferred`, `ui/tests/asset_surface_index.rs`, `ui/tests/asset_surface_index/surface_edges.rs`, `ui/tests/asset_surface_index/dirty_targets.rs`, `runtime_15_ui_asset_surface_index_tests_are_folder_backed`, `ui/tests/runtime_input_reply_routes/keyboard_navigation_routes/focus_path.rs`, `ui/tests/runtime_input_reply_routes/tree_view_pointer_routes/selection.rs`, `Runtime 15 M3 UI runtime input reply table pointer route folder split`, `runtime_15_ui_runtime_input_reply_table_pointer_routes_folder_split_static_passed_cargo_deferred`, `ui/tests/runtime_input_reply_routes/table_pointer_routes.rs`, `ui/tests/runtime_input_reply_routes/table_pointer_routes/resize.rs`, `ui/tests/runtime_input_reply_routes/table_pointer_routes/virtualization.rs`, `runtime_15_ui_runtime_input_reply_table_pointer_routes_are_folder_backed`, `ui/tests/widget_text_input_keyboard/selection_navigation.rs`, and `ui/tests/widget_text_input_keyboard/text_ime.rs`.

Shared-core/focus mirrors for the same sweep: `ui/tests/focus_navigation/modal_popup.rs`, `Runtime 15 M3 UI shared core input visibility child folder split`, `runtime_15_ui_shared_core_input_visibility_child_folder_split_static_passed_cargo_deferred`, `ui/tests/shared_core/input_visibility/hit_visibility.rs`, `ui/tests/shared_core/input_visibility/pointer_routes.rs`, `runtime_15_ui_shared_core_input_visibility_children_are_folder_backed`, `Runtime 15 M3 UI shared core layout surface child folder split`, `runtime_15_ui_shared_core_layout_surface_child_folder_split_static_passed_cargo_deferred`, `ui/tests/shared_core/layout_surface/layout_measurement.rs`, `ui/tests/shared_core/layout_surface/render_extract.rs`, `runtime_15_ui_shared_core_layout_surface_children_are_folder_backed`, `Runtime 15 M3 UI shared core scroll mutation child folder split`, `runtime_15_ui_shared_core_scroll_mutation_child_folder_split_static_passed_cargo_deferred`, `ui/tests/shared_core/scroll_mutation/property_mutation.rs`, `ui/tests/shared_core/scroll_mutation/virtual_scroll.rs`, and `runtime_15_ui_shared_core_scroll_mutation_children_are_folder_backed`.
