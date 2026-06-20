mod command;
mod geometry;
mod identity;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use command::push_template_image_command;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::leading_icon_size;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use identity::{
    is_icon_node, is_icon_only_node,
};
