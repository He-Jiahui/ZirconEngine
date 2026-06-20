mod edge;
mod frame;
mod margin;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use edge::chip_leading_edge;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::{
    chip_avatar_frame, chip_icon_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use margin::{
    chip_leading_margin, chip_negative_slot_margin,
};
