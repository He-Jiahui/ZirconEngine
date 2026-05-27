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
mod template_nodes;
mod text;
mod theme;
mod visual_assets;
mod workbench;

pub(super) use debug_reflector_overlay::draw_debug_reflector_overlay;
pub(super) use diagnostics_overlay::{
    debug_refresh_overlay_frame, presentation_top_bar_frame, union_frames,
};
pub(super) use frame::{HostRecordedPaintCommand, HostRecordedPaintKind, HostRgbaFrame};
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
