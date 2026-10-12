// 根框、文本与默认图标的几何入口集中在此；根框圆角供图像遮罩和表面边框消费，回退子框尺寸另由 child 决定。
#[path = "geometry/child.rs"]
mod child;
#[path = "geometry/frame.rs"]
mod frame;
#[path = "../../avatar/geometry/metrics.rs"]
mod metrics;
#[path = "../../avatar/geometry/radius.rs"]
mod radius;
#[path = "geometry/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use child::avatar_fallback_child_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use frame::avatar_frame;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radius::avatar_corner_radius;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::avatar_text_frame;
