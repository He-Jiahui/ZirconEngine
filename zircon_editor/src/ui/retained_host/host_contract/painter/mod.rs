mod close_prompt;
mod debug_reflector_overlay;
mod diagnostics_overlay;
mod frame;
mod geometry;
mod material_primitives;
mod material_state_layer;
mod mui_x_primitives;
mod primitives;
mod render_commands;
mod sprite_atlas;
mod style_selector;
mod template_alerts;
mod template_axis_labels;
mod template_axis_value_fields;
mod template_buttons;
mod template_chips;
mod template_command_palette;
mod template_dialogs;
mod template_drag_overlay;
mod template_dropdowns;
mod template_fields;
mod template_icon_buttons;
mod template_inspector_rows;
mod template_list_rows;
mod template_node_labels;
mod template_nodes;
mod template_notification_center;
mod template_popup_rows;
mod template_property_rows;
mod template_section_titles;
mod template_segmented_controls;
mod template_selection_controls;
mod template_shell_panels;
mod template_sliders;
mod template_status_controls;
mod template_style;
mod template_table_rows;
mod template_tooltips;
mod template_tree_rows;
mod template_viewport_scene;
mod template_viewport_scene_architecture;
mod template_viewport_scene_floor;
mod template_viewport_scene_light;
mod template_viewport_scene_structure;
mod template_viewport_scene_surfaces;
mod text;
mod theme;
mod visual_assets;
mod workbench;

pub(super) use debug_reflector_overlay::draw_debug_reflector_overlay;
pub(super) use diagnostics_overlay::{
    debug_refresh_overlay_frame, presentation_top_bar_frame, union_frames,
};
pub(super) use frame::{HostRecordedPaintCommand, HostRecordedPaintKind, HostRgbaFrame};
pub(in crate::ui::retained_host::host_contract) use geometry::frame_from_template;
pub(super) use primitives::{
    draw_rect_clipped, draw_rgba_image_clipped_with_resource_key, draw_rounded_border_clipped,
    draw_rounded_rect_clipped,
};
pub(super) use text::draw_text_with_size_and_style;
pub(super) use workbench::{paint_host_frame, record_host_frame_commands};

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use sprite_atlas::{
    HostPaintAtlasImage, HostPaintImageUvRect,
};

#[cfg(test)]
pub(crate) use render_commands::paint_runtime_render_commands_for_test;
#[cfg(test)]
pub(crate) use template_nodes::{
    paint_template_nodes_for_test, paint_template_nodes_for_test_with_background,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use workbench::repaint_host_frame_region;
