mod body;
mod entry;
mod title;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_tooltip_text;

pub(super) fn tooltip_title(node: &super::super::data::TemplatePaneNodeData) -> String {
    let text = node.text.as_str().trim();
    if text.is_empty() {
        "Tooltip".to_string()
    } else {
        text.to_string()
    }
}

pub(super) fn tooltip_body(node: &super::super::data::TemplatePaneNodeData) -> String {
    let text = node.label_text.as_str().trim();
    if text.is_empty() {
        "This is a tooltip".to_string()
    } else {
        text.to_string()
    }
}
