#[path = "../regions/center_status.rs"]
mod center_status;
#[path = "../regions/same_group.rs"]
mod same_group;

pub(super) use self::center_status::{
    center_band_status_damage_frame, center_band_status_with_source_damage_frame,
};
pub(super) use self::same_group::release_same_group_damage_frame;
