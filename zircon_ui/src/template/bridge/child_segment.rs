use crate::template::UiTemplateNode;

pub(super) fn child_segment(node: &UiTemplateNode, index: usize) -> String {
    let raw = node
        .control_id
        .as_deref()
        .or(node.component.as_deref())
        .unwrap_or("node");
    let sanitized = raw
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ' ' | ':' | '#' => '_',
            _ => ch,
        })
        .collect::<String>();
    format!("{sanitized}_{index}")
}
