#[path = "pane_frames/activity_rail.rs"]
mod activity_rail;
#[path = "pane_frames/content.rs"]
mod content;
#[path = "pane_frames/pane.rs"]
mod pane;
#[path = "pane_frames/surface_frame.rs"]
mod surface_frame;
#[path = "pane_frames/nodes.rs"]
pub(super) mod template_nodes;

pub(in crate::ui::retained_host::host_contract) use self::activity_rail::collect_activity_rail_buttons;
pub(in crate::ui::retained_host::host_contract) use self::content::{
    floating_window_content_frame, side_dock_content_frame,
};
pub(in crate::ui::retained_host::host_contract) use self::pane::collect_pane_profile_frames;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use self::surface_frame::collect_surface_frame_controls;
