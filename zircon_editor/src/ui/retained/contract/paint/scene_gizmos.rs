#[path = "scene_gizmos/axes.rs"]
mod axes;
#[path = "scene_gizmos/center.rs"]
mod center;
#[path = "scene_gizmos/selection.rs"]
mod selection;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use axes::{
    push_axis_line, push_axis_origin,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use center::push_gizmo_center;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::push_selection_glow;
