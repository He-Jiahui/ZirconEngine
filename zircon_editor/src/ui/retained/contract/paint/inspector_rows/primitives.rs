#[path = "primitives/field.rs"]
mod field;
#[path = "primitives/labels.rs"]
mod labels;
#[path = "primitives/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use field::push_field;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::{
    push_label, push_nested_label,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::push_text;
