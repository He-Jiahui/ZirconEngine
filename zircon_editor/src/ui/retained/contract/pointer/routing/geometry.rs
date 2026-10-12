#[path = "geometry/containment.rs"]
mod containment;
#[path = "geometry/frames.rs"]
mod frames;

pub(in crate::ui::retained_host::host_contract) use self::containment::contains;
pub(in crate::ui::retained_host::host_contract) use self::frames::{
    floating_window_content_frame, side_dock_content_frame, translated,
};
