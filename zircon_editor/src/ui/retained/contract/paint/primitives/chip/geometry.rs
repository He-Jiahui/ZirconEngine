#[path = "geometry/delete.rs"]
mod delete;
#[path = "geometry/frame.rs"]
mod frame;
#[path = "geometry/label.rs"]
mod label;
#[path = "geometry/leading.rs"]
mod leading;
#[path = "geometry/metrics.rs"]
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use delete::chip_delete_icon_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::{
    chip_corner_radius, chip_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use label::chip_label_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use leading::{
    chip_avatar_frame, chip_icon_frame,
};
