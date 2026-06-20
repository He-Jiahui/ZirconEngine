mod hit;
#[cfg(test)]
mod tests;
mod union;
mod visibility;

pub(in crate::ui::retained_host::host_contract) use hit::contains_point;
pub(in crate::ui::retained_host::host_contract) use union::{union_frame, union_optional_frames};
pub(in crate::ui::retained_host::host_contract) use visibility::visible_frame;
