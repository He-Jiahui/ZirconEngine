mod child;
mod frame;
mod metrics;
mod radius;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use child::avatar_fallback_child_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::avatar_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radius::avatar_corner_radius;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::avatar_text_frame;
