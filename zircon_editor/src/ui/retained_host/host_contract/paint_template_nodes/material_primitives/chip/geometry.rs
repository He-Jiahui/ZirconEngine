mod delete;
mod frame;
mod label;
mod leading;
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use delete::chip_delete_icon_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::{
    chip_corner_radius, chip_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use label::chip_label_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use leading::{
    chip_avatar_frame, chip_icon_frame,
};
