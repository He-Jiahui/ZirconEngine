#[path = "text_focus/activation.rs"]
mod activation;
#[path = "text_focus/clear.rs"]
mod clear;

pub(in crate::ui::retained_host::host_contract) use self::activation::focus_template_node_text_input;
pub(super) use self::clear::clear_focused_text_input_on_primary_press;
