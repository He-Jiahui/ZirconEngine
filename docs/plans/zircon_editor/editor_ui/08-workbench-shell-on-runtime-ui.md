---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workbench_snapshot_access.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/window.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/template_hover.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callback_methods.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/ui_context.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/event_loop/platform_input.rs
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_editor/src/tests/host/retained_window/platform_input_translation.rs
  - docs/zircon_editor/ui/retained_host/host_contract/platform_input.md
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_popup_dismiss/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/classifier.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/path.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/provider.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_geometry_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/classification.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics/target.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_input_semantics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/invalidation.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics/refresh.rs
  - zircon_editor/src/ui/retained_host/host_contract/diagnostics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/request.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw/dispatch_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/redraw_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/asset.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/helpers.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics/route.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_activation_semantics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/classify.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/family.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/roles.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/visual_language.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_component_family_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/bounds.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/dropdown.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/metrics.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/template_popup_layout_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/environment.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/export.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/frame_math.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/hit_samples.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/pane_frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/geometry/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/schema.rs
  - zircon_editor/src/ui/retained_host/host_contract/profiling_artifacts/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/command.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/button.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_close_prompt/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/colors.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_debug_reflector_overlay_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/marker.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/top_bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/union.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_diagnostics_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_frame/recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/pixel_rect.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_geometry/rect_ops.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/image.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_primitives/pixels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_recording.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/blend.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/font.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text/raster.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_text_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_material_feedback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_images.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/draw.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline/test_support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/clip.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/dropdown_popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/menu_popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_pipeline_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_text.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels_tests/property.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_node_labels_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_nodes/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/surface.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_style_color.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_labels_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_field_style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_axis_value_fields_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_field_stepper.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_fields_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_buttons_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chip_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_chips_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tree_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdown_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_dropdowns_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltip_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_tooltips_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_title_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_section_titles_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/adornment.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_list_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_slider_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/toggle.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/marks.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_selection_controls_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/options.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/tabs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_segmented_controls_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_buttons/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_segments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/files.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/tools.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_icon_button_glyph_shapes/visibility.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_kind.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_inspector_row_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_row_adornments.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/adornment.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_popup_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows_tests/axis.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows_tests/component.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_property_axis_values.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/cells.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_table_rows_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_control_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/signals.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/chips.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_controls_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_status_glyphs.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer_tests/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer_tests/ripple.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/material_state_layer_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_shell_panels/separators.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/architecture.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/floor.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/gizmo.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/light.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/props.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/surfaces.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_architecture.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_floor.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_light.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_structure.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_viewport_scene_surfaces.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/render_commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas_tests/resolver.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/sprite_atlas_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/runtime.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/template.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/editor_pages.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/mui.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/svg.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/tint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/candidates.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/mui_icons.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/identity.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/paint.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_sliders_tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench/commands.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench/test_frame.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/bottom.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/document.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/floating_windows.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/fallback.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/panel_header.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/rail.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/side.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/viewport_toolbar.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/menus.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/dock_layer.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/scene_layers/resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/layout.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column/frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/recent_projects.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/style.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/chrome_press.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/close_prompt_hit.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/asset_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/native_panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/template_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/text_focus.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/viewport_button.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize/resize_capture.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize/tab_drag.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/move_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/bar.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/damage.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/frames.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/menu_geometry/popup.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/redraw_result.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/chrome.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/geometry.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/panes.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/routing/workbench.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/scroll_dispatch.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/pane_nodes.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/popup_rows.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node/surface_frame_builder.rs
  - zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/template_node_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/mod.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/runtime_draw_list/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stats.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/stream.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/extraction.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/replay.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/stream_model.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/tests/support.rs
  - zircon_editor/src/ui/retained_host/host_contract/chrome_command_stream/atlas_tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/snapshot.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/diagnostics.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/softbuffer/surface_io.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/workbench_context_menu.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/context_menu.rs
  - zircon_editor/src/ui/retained_host/floating_window_projection.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/workbench/view/view_descriptor.rs
  - zircon_editor/src/ui/workbench/view/dock_policy.rs
  - zircon_editor/src/ui/workbench/window_registry
  - zircon_editor/src/ui/workbench/preset/shell_preset.rs
  - zircon_editor/src/ui/workbench/preset/default_layout.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/core/commands/mod.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
  - zircon_editor/assets/ui/editor/components/workbench/shell
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
plan_sources:
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - .codex/plans/Zircon Editor Workbench Shell VNext.md
  - .codex/plans/JetBrains Hybrid Workbench Shell Spec Implementation Plan.md
  - .codex/plans/GPU Command Stream 接管 Editor UI 渲染计划.md
  - .codex/plans/Drawer_Window_Menu Slate 化推进计划.md
design_references:
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tool-drawers-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-collapsed-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/floating-tool-window-state-spec.png
status: planned
---

# 08 Workbench Shell 全面切到 Runtime UI

## 1. 目标

宿主编辑器窗口从「editor 自管 presentation/painter」切到「runtime UI surface 承载」：workbench shell（top toolbar + main tabs、activity rail、左/右/底 drawer、中央 document workspace、status bar）全部以计划 06 的 L4 组件拼装，布局走 runtime Taffy + docking 接缝（02 M4），输入走统一 input manager（01 M5），样式走 selector（04），渲染继续走 GPU command stream。editor 只保留 workbench/docking/windowing 语义与编辑器业务状态。同时补齐壳级缺口：浮动窗口、完整菜单栏、快捷键、布局持久化、context menu、toast 触发、status bar 扩展。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| runtime `UiSurface` 宿主能力 | `zircon_runtime/src/ui/surface/surface.rs` | `hit_test`（:156）、`surface_frame`（:165）、`mutate_property`（:223）、`reflector_snapshot`（:299）、`focus_path`（:307）、`capture_pointer`（:315）、`apply_dispatch_reply`（:338）、`dispatch_input_event(_with_manager)`（:354/:363）、`dispatch_window_input_pump_event(_with_manager)`（:371/:380）——**宿主接缝 API 基本齐备** |
| L4 shell `.zui` 资产（8 件） | `zircon_editor/assets/ui/editor/components/workbench/shell/` | workbench_top_toolbar、workbench_main_band、workbench_activity_rail、workbench_status_bar、workbench_component_drawer、workbench_scene_tree_panel、workbench_inspector_panel、workbench_viewport_panel |
| view registry | `zircon_editor/src/ui/workbench/view/` | view_descriptor(+builder/id)、view_registry(+descriptor/instance access、instance_mutation)、view_kind、dock_policy、pane_template_spec、pane_route_namespace、preferred_host 共 20 文件 |
| 窗口注册表 | `zircon_editor/src/ui/workbench/window_registry/` + `src/tests/workbench/registry/window_registry.rs` | EditorWindowRegistry 已有骨架与测试 |
| 布局 preset | `zircon_editor/src/ui/workbench/preset/` | default_layout、default_registry、shell_preset、panel_preset、functional_window、design_stack |
| shell 几何 | `zircon_editor/src/ui/workbench/autolayout/` | workbench_shell_geometry、region、constraints（02 M4 接缝对象） |
| 旧投影双轨（待退役） | `zircon_editor/src/ui/retained_host/` | `app/workbench_snapshot_access.rs` 已替代并删除 `HostPresentationCache`；`root_shell_projection.rs`、旧 `host_contract/painter/` 目录、旧 `presenter/command_stream` 与 `presenter/extraction` 路径已删除；剩余 `floating_window_projection.rs`、中立 `host_contract/paint_template_nodes/` 软件绘制族、`host_contract/chrome_command_stream/` 过渡 command stream 与 presenter 后端 |

### 2.2 真实缺口

1. **双轨投影**：editor host contract 中立 paint + `chrome_command_stream/` 与 runtime render extract 仍是两套投影；shell 像素当前由 `paint_*` / `paint_template_nodes/` 路径录制并由 `chrome_command_stream/` 过渡回放，旧 `painter` 命名空间和旧 presenter command-stream 路径已删除。
2. **菜单栏只有快速按钮**；无命令注册表（grep 无 CommandRegistry/keymap 命中）、无快捷键表、无 CommandPalette 数据源。
3. **浮窗仅模板投影**（floating_window_projection.rs），window_registry 未接独立 `UiSurface` + 原生子窗口。
4. **布局持久化不全**：`host/layout_persistence.rs` 与 preset 模块已有骨架（default/builtin 级），project workspace 与 global default 序列化恢复缺失。
5. 全局 context menu、toast 触发链、status bar 实时状态、split tabs 未完成。

## 3. 设计

### 3.1 承载切换（核心硬切换）

- editor 主窗口持有一个 runtime `UiSurface`（新增 `retained_host/shell_surface_host.rs` 作 owner）：shell 树由 L4 组件实例化（§2.1 的 8 件 `.zui`），`UiSurfaceFrame` 驱动布局/命中/提取，GPU command stream 消费 `UiRenderExtract`。
- Workbench shell layout 描述只以 `.zui` 为当前权威；`.ui.toml` / `.v2.ui.toml` 后缀已退役，不再作为当前壳层、抽屉、浮窗或插件 editor view/layout 入口。历史状态表中的旧后缀只作为迁移证据保留。
- **区域硬切顺序与删除清单**（每区域同变更删除旧投影）：

| 区域 | 承载 `.zui` | 同变更删除 |
|------|------------|-----------|
| status bar（M1） | workbench_status_bar.zui | painter 中 status bar 投影段 + presentation_cache 对应区段 |
| activity rail（M2） | workbench_activity_rail.zui | activity_rail_pointer 桥（与 01 M5 协同）+ painter rail 段 |
| main tabs（M2） | workbench_main_band.zui | document_tab_pointer、tab_drag 命中态 + painter tab 段 |
| drawers（M2） | workbench_component_drawer.zui + scene_tree/inspector panel | drawer_header_pointer、drawer_resize 命中态 + painter drawer 段 |
| 全 shell（M3） | workbench_top_toolbar.zui + viewport_panel.zui | `root_shell_projection.rs`、`app/presentation_cache.rs`、旧 `host_contract/painter/` 投影族删除确认、presenter/ 残余 |

- workbench 模型（`WorkbenchLayout`、view registry、EditorState）保留为业务状态层，经数据绑定（route id / `UiSurface::mutate_property`）与 surface 同步——editor 改状态、runtime 改像素。

### 3.2 Docking 与窗口

- docking 拓扑沿用 Shell V1 定稿：固定壳 + 受控 docking 树；只有中心 document workspace 与浮窗允许递归 split；6 个固定 drawer 槽；dock_policy.rs 既有语义沿用。
- FloatingWindow：window_registry 管实例；每个浮窗一个独立 `UiSurface` + 原生子窗口（复用 runtime window 抽象）；drawer ↔ 浮窗互转。
- 布局持久化：preset > project workspace > global default > builtin fallback 四级恢复；序列化 docking 树 + drawer extent + 活动 view。

### 3.3 壳级功能补齐

- **菜单栏**：File/Edit/View/Window/Help 真实菜单树（PopupMenu 多级已有路由），菜单项 = command id + 快捷键标注 + enabled 谓词。
- **命令与快捷键**：editor command registry（id、标题、类别、默认键位）；keymap 资产（TOML）可改绑；input manager 焦点链未消费的按键进 keymap 解析；CommandPalette（06 M3 骨架）按命令注册表搜索执行。
- **Context menu**：右键经 hit path 取最近声明 context-menu provider 的节点，editor 按节点语义出菜单。
- **Toast/通知**：editor 事件（构建完成、导入失败等）→ Toast 队列（06 行为）+ NotificationCenter 历史。
- **Status bar**：左侧状态消息/警告计数，右侧 grid/snap/zoom 等 chips + 任务进度槽，数据绑定实时刷新。

### 3.4 Viewport 接缝

- ViewportPanel 作为 UI 节点持有 runtime 场景纹理（GPU command stream 已支持 surface 合成）；指针事件经 01 的路由进入 viewport 节点后转交 scene 交互路径（picking/gizmo/camera controller），UI 不解释 3D 语义。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_editor/src/ui/retained_host/shell_surface_host.rs
pub struct EditorShellSurfaceHost {
    surface: UiSurface,                        // 现有类型（runtime）
    intent_map: EditorRouteIntentMap,          // 01 M5 类型
    binding_sync: ShellBindingSync,            // workbench 状态 → mutate_property 批
}
impl EditorShellSurfaceHost {
    pub fn instantiate_shell(&mut self, /* prototype store 句柄 */) -> Result<(), EditorShellHostError>;
    pub fn pump_input(&mut self, batch: UiWindowInputPumpBatch) -> Vec<EditorIntent>;   // 01 outcome → intent
    pub fn sync_workbench_state(&mut self, layout: &WorkbenchLayout /* 现有 */);         // 状态差量 → property mutation
}

// 新增 zircon_editor/src/ui/host/commands/{mod.rs, registry.rs, keymap.rs}
pub struct EditorCommandRegistry { commands: Vec<EditorCommandDescriptor> }
pub struct EditorCommandDescriptor {
    pub id: EditorCommandId,                   // "editor.scene.delete" 等稳定 id
    pub title: String,
    pub category: EditorCommandCategory,       // File | Edit | View | Window | Help | Scene | Asset
    pub default_binding: Option<EditorKeyChord>,
    pub enabled_route: Option<String>,         // enabled 谓词的数据绑定路径
}
pub struct EditorKeymap { bindings: Vec<(EditorKeyChord, EditorCommandId)> }
// keymap 资产 TOML（zircon_editor/assets/ui/editor/keymap/default.keymap.toml）：
// [bindings]
// "editor.command_palette" = "Ctrl+Shift+P"
// "editor.scene.delete"    = "Delete"
// "editor.scene.rename"    = "F2"
impl EditorKeymap {
    pub fn resolve(&self, chord: EditorKeyChord) -> Option<EditorCommandId>;   // 焦点链未消费按键进此
}

// 布局持久化（扩展既有 zircon_editor/src/ui/host/layout_persistence.rs）
pub struct WorkbenchLayoutSnapshot {
    pub docking_tree: /* dock_policy 序列化形态 */,
    pub drawer_extents: Vec<(ShellRegionId, f32)>,   // 现有 ShellRegionId
    pub active_views: Vec<ViewInstanceId>,           // 现有类型（view/）
}
pub enum WorkbenchLayoutSource { Preset(String), ProjectWorkspace, GlobalDefault, Builtin }
pub fn restore_layout(/* 四级查找 */) -> (WorkbenchLayoutSnapshot, WorkbenchLayoutSource);

// 浮窗（扩展 workbench/window_registry/）
pub struct EditorFloatingWindow {
    pub view: ViewInstanceId,
    pub surface: UiSurface,                    // 每浮窗独立
    pub native_window: /* runtime window 抽象句柄 */,
}
```

## 5. 模块与文件落点

**新增**：`retained_host/shell_surface_host.rs`、`host/commands/{mod.rs, registry.rs, keymap.rs}`、`assets/ui/editor/keymap/default.keymap.toml`、菜单树 `.zui`/模板（File/Edit/View/Window/Help 内容声明）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `retained_host/app.rs`、`app/host_lifecycle.rs` | 主窗口生命周期挂 EditorShellSurfaceHost；按区域逐步把渲染来源切到 surface extract |
| `workbench/window_registry/` | 浮窗实例持独立 UiSurface + 原生子窗口 |
| `host/layout_persistence.rs`、`workbench/preset/{default_layout, shell_preset}.rs` | 扩展为四级恢复链（序列化 docking 树 + drawer extent + 活动 view） |
| `workbench/view/view_registry*.rs` | 视图激活状态经 binding_sync 同步 |
| `host/module.rs` | EditorModule 接线命令注册表与 keymap 加载 |

**删除（硬切换义务，按区域分批）**：§3.1 删除清单全部条目；M3 已确认 `presentation_cache.rs`、`root_shell_projection.rs`、旧 `host_contract/painter/` 投影族、旧 `presenter/command_stream` 与 `presenter/extraction` 路径物理删除；剩余中立 `chrome_command_stream/` 与 `paint_template_nodes/` 软件绘制族继续按后续切片收束到 runtime extract / GPU command stream 边界（验收项）。

## 6. 管线时序（切换后）

```
winit（editor EventLoop）→ 01 platform_input 翻译 → batch
→ EditorShellSurfaceHost.pump_input → UiSurface dispatch（01 七阶段路由）
→ component events → route_intent → EditorIntent → editor command（undo/redo）
→ workbench 状态变更 → sync_workbench_state → mutate_property → dirty
→ runtime 帧管线（state→motion→layout→text→extract）→ GPU command stream → present
键盘未消费 → EditorKeymap.resolve → command 执行
```

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | EditorShellSurfaceHost 骨架：主窗口挂 UiSurface，实例化最小 shell 树（仅 status bar 区） | shell_surface_host.rs、app.rs | `cargo check -p zircon_editor --lib --locked` | 无删除 |
| M1.S2 | status bar 区域首迁：workbench_status_bar.zui 承载 + 数据绑定（状态消息/chips） | shell_surface_host.rs、status_bar.zui | `cargo test -p zircon_editor --lib status_bar --locked` | 删 painter status bar 段 |
| M1.S3 | 实机：status bar 由 runtime 路径渲染/命中（GPU stream 验收 software_fallback_count=0 沿用） | 实机 | editor 实机 | 删 presentation_cache 对应区段 |
| M2.S1 | activity rail 迁移（与 01 M5.S2 协同删桥） | activity_rail.zui、shell_surface_host | `cargo test -p zircon_editor --lib activity_rail --locked` | 删 rail 桥命中态 + painter 段 |
| M2.S2 | main tabs 迁移：tab 切换/拖拽重排走 runtime（06 TabStrip） | main_band.zui | `cargo test -p zircon_editor --lib document_tab --locked` | 删 tab 桥命中态 + painter 段 |
| M2.S3 | drawers 框架迁移 + docking 接缝接通（02 M4 PaneContentRootConstraint）：开合/改宽/切 tab 全走新路径 | component_drawer.zui、autolayout | `cargo test -p zircon_editor --lib drawer --locked` + 实机 | 删 drawer 桥命中态 + painter 段 |
| M3.S1 | 剩余区域（top toolbar、document workspace、viewport 挂点）迁移 | top_toolbar.zui、viewport_panel.zui | `cargo test -p zircon_editor --lib --locked` | 删 root_shell_projection.rs |
| M3.S2 | 旧路径总删除：painter 投影族、presentation_cache、presenter 残余；全壳实机交互回归 | retained_host/ | `cargo test -p zircon_editor --lib --locked` + `--test integration_contracts --features integration-contracts` + 实机 | **删除确认清单出文档** |
| M4.S1 | EditorCommandRegistry + 默认命令集（File/Edit/View/Window/Help 全菜单项） | host/commands/ | `cargo test -p zircon_editor --lib commands --locked` | 快速按钮旧实现删除 |
| M4.S2 | keymap 资产 + 焦点链未消费按键解析（01 路由 default-action 后接） | keymap.rs、keymap.toml | `cargo test -p zircon_editor --lib keymap --locked` | 无删除 |
| M4.S3 | 菜单栏真实菜单树 + CommandPalette 接命令源（06 M3 骨架）；命令矩阵测试 + 实机快捷键 | 菜单模板、palette | 同上 + 实机 | 无删除 |
| M5.S1 | 浮窗：window_registry 接独立 UiSurface + 原生子窗口；drawer ↔ 浮窗互转 | window_registry/ | `cargo test -p zircon_editor --lib window_registry --locked` | 删 floating_window_projection.rs |
| M5.S2 | 布局持久化四级恢复：序列化 + 启动恢复 + preset 切换 | layout_persistence.rs、preset/ | `cargo test -p zircon_editor --lib layout_persistence --locked` + 重启实机 | 无删除 |
| M6.S1 | context menu：hit path → provider 查找 → 节点语义菜单 | shell_surface_host、菜单 | `cargo test -p zircon_editor --lib context_menu --locked` | 无删除 |
| M6.S2 | toast 触发链 + NotificationCenter 历史（06 M3 组件） | editor 事件 → toast 队列 | `cargo test -p zircon_editor --lib toast --locked` | 无删除 |
| M6.S3 | status bar 实时状态（任务进度槽）+ 实机验收 | binding_sync | 实机 + focused tests | 无删除 |

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`08/2026-07-09-workbench-shell-on-runtime-ui-output-records.md`](08/2026-07-09-workbench-shell-on-runtime-ui-output-records.md)
- 当前失败交接（`open / 待修复`）：[`08/failure-2026-07-11-runtime-diagnostics-physics-state-format.md`](08/failure-2026-07-11-runtime-diagnostics-physics-state-format.md)
- 当前失败交接（`open / 待修复`）：[`08/failure-2026-07-11-retained-window-hard-cutover-expectations.md`](08/failure-2026-07-11-retained-window-hard-cutover-expectations.md)
- fixed 已修复：[componentized-workspace-test-export](../editor/14/fixed-2026-07-12-componentized-workspace-test-export.md)
