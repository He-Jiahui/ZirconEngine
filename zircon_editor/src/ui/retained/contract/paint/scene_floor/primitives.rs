#[path = "primitives/alpha.rs"]
mod alpha;
#[path = "../../../../contract/paint/primitives/scene_floor_inset_rect.rs"]
mod inset_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use alpha::color_with_alpha_factor;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use inset_rect::push_inset_rect;
