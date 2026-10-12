#[path = "../../../../../contract/paint/divider/geometry/label_bounds/align.rs"]
mod align;
#[path = "../../../../../contract/paint/divider/geometry/label_bounds/horizontal.rs"]
mod horizontal;
#[path = "../../../../../contract/paint/divider/geometry/label_bounds/vertical.rs"]
mod vertical;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use horizontal::horizontal_label_bounds;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use vertical::vertical_label_bounds;
