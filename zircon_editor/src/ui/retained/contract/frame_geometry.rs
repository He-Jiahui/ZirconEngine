#[path = "frame_geometry/hit.rs"]
mod hit;
#[cfg(test)]
#[path = "frame_geometry/tests/cases.rs"]
mod tests;
#[path = "frame_geometry/union.rs"]
mod union;
#[path = "frame_geometry/visibility.rs"]
mod visibility;

pub(in crate::ui::retained_host::host_contract) use hit::contains_point;
pub(in crate::ui::retained_host::host_contract) use union::{union_frame, union_optional_frames};
pub(in crate::ui::retained_host::host_contract) use visibility::visible_frame;
