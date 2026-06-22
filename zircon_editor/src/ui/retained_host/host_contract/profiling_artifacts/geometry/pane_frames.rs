mod activity_rail;
mod content;
mod pane;
mod surface_frame;
mod template_nodes;

pub(in crate::ui::retained_host::host_contract) use self::activity_rail::collect_activity_rail_buttons;
pub(in crate::ui::retained_host::host_contract) use self::content::{
    floating_window_content_frame, side_dock_content_frame,
};
pub(in crate::ui::retained_host::host_contract) use self::pane::collect_pane_profile_frames;

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) use self::surface_frame::collect_surface_frame_controls;
