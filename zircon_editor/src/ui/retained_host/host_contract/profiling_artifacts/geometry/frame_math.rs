mod center;
mod intersection;
mod named;
mod translation;
mod visibility;

pub(in crate::ui::retained_host::host_contract) use center::{
    frame_rect_center_point, profile_frame_center,
};
pub(in crate::ui::retained_host::host_contract) use intersection::{
    intersect_frames, intersect_profile_frame,
};
pub(in crate::ui::retained_host::host_contract) use named::{
    push_named_frame, push_named_profile_frame,
};
pub(in crate::ui::retained_host::host_contract) use translation::{
    translated, translated_template_frame,
};
pub(in crate::ui::retained_host::host_contract) use visibility::{
    is_visible_frame, is_visible_profile_frame, visible_profile_frame,
};
