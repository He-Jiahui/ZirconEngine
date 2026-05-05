mod diagnostics_overlay;
mod frame;
mod geometry;
mod primitives;
mod render_commands;
mod template_nodes;
mod text;
mod visual_assets;
mod workbench;

pub(super) use frame::HostRgbaFrame;
pub(super) use diagnostics_overlay::{
    debug_refresh_overlay_frame, top_bar_frame as fallback_debug_top_bar_frame, union_frames,
};
pub(super) use workbench::{paint_host_frame, repaint_host_frame_region};

#[cfg(test)]
pub(crate) use render_commands::paint_runtime_render_commands_for_test;
