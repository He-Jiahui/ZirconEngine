#[path = "../../divider/labels/horizontal.rs"]
mod horizontal;
#[path = "labels/text.rs"]
mod text;
#[path = "../../divider/labels/vertical.rs"]
mod vertical;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use horizontal::push_horizontal_divider_label;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::divider_label;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use vertical::push_vertical_divider_label;
