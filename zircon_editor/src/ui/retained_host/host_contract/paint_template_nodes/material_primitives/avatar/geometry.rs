// 根框、文本与默认图标的几何入口集中在此；根框圆角供图像遮罩和表面边框消费，回退子框尺寸另由 child 决定。
mod child;
mod frame;
mod metrics;
mod radius;
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use child::avatar_fallback_child_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::avatar_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radius::avatar_corner_radius;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::avatar_text_frame;
