#[path = "evidence.rs"]
mod evidence;
#[path = "primitives.rs"]
mod material_primitives;
#[path = "material_state_layer.rs"]
mod material_state_layer;
#[path = "mui_x_primitives.rs"]
mod mui_x_primitives;
#[path = "commands.rs"]
mod render_command_conversion;
#[path = "render_commands.rs"]
mod render_commands;
#[path = "sprite_atlas.rs"]
mod sprite_atlas;
#[path = "selector/mod.rs"]
mod style_selector;
#[path = "alert_glyphs.rs"]
mod template_alert_glyphs;
#[path = "alerts.rs"]
mod template_alerts;
#[path = "asset_placeholder_visuals.rs"]
mod template_asset_placeholder_visuals;
#[path = "axis_labels.rs"]
mod template_axis_labels;
#[path = "axis_value_field_style.rs"]
mod template_axis_value_field_style;
#[path = "axis_value_fields.rs"]
mod template_axis_value_fields;
#[path = "button_glyphs.rs"]
mod template_button_glyphs;
#[path = "buttons.rs"]
mod template_buttons;
#[path = "chip_glyphs.rs"]
mod template_chip_glyphs;
#[path = "chips.rs"]
mod template_chips;
#[path = "command_palette.rs"]
mod template_command_palette;
#[path = "dialogs.rs"]
mod template_dialogs;
#[path = "diamond_glyph.rs"]
mod template_diamond_glyph;
#[path = "drag_overlay.rs"]
mod template_drag_overlay;
#[path = "dropdown_glyphs.rs"]
mod template_dropdown_glyphs;
#[path = "dropdown_metrics.rs"]
mod template_dropdown_metrics;
#[path = "dropdowns.rs"]
mod template_dropdowns;
#[path = "field_stepper.rs"]
mod template_field_stepper;
#[path = "fields.rs"]
mod template_fields;
#[path = "icon_assets.rs"]
mod template_icon_assets;
#[path = "icon_button_glyphs.rs"]
mod template_icon_button_glyphs;
#[path = "icon_buttons.rs"]
mod template_icon_buttons;
#[path = "inspector_row_geometry.rs"]
mod template_inspector_row_geometry;
#[path = "inspector_row_glyphs.rs"]
mod template_inspector_row_glyphs;
#[path = "inspector_row_kind.rs"]
mod template_inspector_row_kind;
#[path = "inspector_rows.rs"]
mod template_inspector_rows;
#[path = "list_row_glyphs.rs"]
mod template_list_row_glyphs;
#[path = "list_rows.rs"]
mod template_list_rows;
#[path = "feedback.rs"]
mod template_material_feedback;
#[path = "node_images.rs"]
mod template_node_images;
#[path = "node_labels.rs"]
mod template_node_labels;
#[path = "node_pipeline.rs"]
mod template_node_pipeline;
#[path = "node_surface.rs"]
mod template_node_surface;
#[path = "node_text.rs"]
mod template_node_text;
#[path = "nodes.rs"]
mod template_nodes;
#[path = "notification_center.rs"]
mod template_notification_center;
#[path = "popup_row_adornments.rs"]
mod template_popup_row_adornments;
#[path = "popup_rows.rs"]
mod template_popup_rows;
#[path = "property_axis_values.rs"]
mod template_property_axis_values;
#[path = "property_rows.rs"]
mod template_property_rows;
#[path = "row_metrics.rs"]
mod template_row_metrics;
#[path = "sample_grid.rs"]
mod template_sample_grid;
#[path = "section_title_glyphs.rs"]
mod template_section_title_glyphs;
#[path = "section_titles.rs"]
mod template_section_titles;
#[path = "segmented_control_geometry.rs"]
mod template_segmented_control_geometry;
#[path = "segmented_controls.rs"]
mod template_segmented_controls;
#[path = "selection_control_geometry.rs"]
mod template_selection_control_geometry;
#[path = "selection_controls.rs"]
mod template_selection_controls;
#[path = "settings_window.rs"]
mod template_settings_window;
#[path = "shell_panels.rs"]
mod template_shell_panels;
#[path = "slider_geometry.rs"]
mod template_slider_geometry;
#[path = "sliders.rs"]
mod template_sliders;
#[path = "status_control_geometry.rs"]
mod template_status_control_geometry;
#[path = "status_controls.rs"]
mod template_status_controls;
#[path = "status_glyphs.rs"]
mod template_status_glyphs;
#[path = "style.rs"]
mod template_style;
#[path = "style_color.rs"]
mod template_style_color;
#[path = "table_rows.rs"]
mod template_table_rows;
#[path = "timeline_strip.rs"]
mod template_timeline_strip;
#[path = "tooltip_glyphs.rs"]
mod template_tooltip_glyphs;
#[path = "tooltips.rs"]
mod template_tooltips;
#[path = "tree_row_geometry.rs"]
mod template_tree_row_geometry;
#[path = "tree_row_glyphs.rs"]
mod template_tree_row_glyphs;
#[path = "tree_rows.rs"]
mod template_tree_rows;
#[path = "viewport_scene.rs"]
mod template_viewport_scene;
#[path = "scene_architecture.rs"]
mod template_viewport_scene_architecture;
#[path = "scene_floor.rs"]
mod template_viewport_scene_floor;
#[path = "scene_gizmos.rs"]
mod template_viewport_scene_gizmos;
#[path = "scene_light.rs"]
mod template_viewport_scene_light;
#[path = "scene_props.rs"]
mod template_viewport_scene_props;
#[path = "scene_structure.rs"]
mod template_viewport_scene_structure;
#[path = "scene_surfaces.rs"]
mod template_viewport_scene_surfaces;
#[path = "weight_heatmap.rs"]
mod template_weight_heatmap;
#[path = "visual_assets.rs"]
mod visual_assets;

pub(crate) use evidence::PaintEvidenceScope;
pub(in crate::ui::retained_host::host_contract) use evidence::{
    active_owner, owner_json, record_native_hierarchy_row,
};
pub(in crate::ui::retained_host::host_contract) use render_commands::draw_runtime_surface_frame;
pub(crate) use sprite_atlas::{
    copy_editor_sprite_atlas_rgba, invalidate_editor_sprite_atlas_cache,
};
pub(super) use template_node_pipeline::{
    draw_template_nodes, draw_template_nodes_with_transform, has_template_nodes,
    TemplateNodePaintTransform,
};
pub(crate) use template_tree_row_geometry::tree_disclosure_rect_for_indent;
pub(super) use template_viewport_scene::is_viewport_fallback_scene_node;
pub(in crate::ui::retained_host) use visual_assets::{
    bind_visual_asset_loader, clear_svg_tree_cache, clear_visual_asset_pixels_cache,
    invalidate_svg_tree_paths, invalidate_visual_asset_pixel_paths, reconcile_svg_tree_sources,
    reconcile_visual_asset_pixel_sources, take_visual_asset_completion, unbind_visual_asset_loader,
    VisualAssetLoadCompletion,
};

#[cfg(test)]
pub(crate) use render_commands::paint_runtime_render_commands_for_test;

pub(crate) use template_node_pipeline::paint_template_nodes_for_evidence_with_background;

#[cfg(test)]
pub(crate) use template_node_pipeline::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
};
#[cfg(test)]
pub(crate) use template_nodes::template_node_command_summary_for_test;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use template_popup_rows::record_popup_menu_rows_for_test;
