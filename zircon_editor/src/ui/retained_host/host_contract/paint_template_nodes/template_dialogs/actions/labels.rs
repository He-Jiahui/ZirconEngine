use super::super::super::super::data::TemplatePaneNodeData;
use super::metrics::{DIALOG_ACTION_CHAR_WIDTH, DIALOG_ACTION_MIN_WIDTH};

pub(super) fn action_label(node: &TemplatePaneNodeData, index: usize) -> Option<String> {
    node.actions
        .row_data(index)
        .and_then(|action| non_empty(action.label.as_str()).map(str::to_string))
}

pub(super) fn action_width(text: &str) -> f32 {
    (text.chars().count() as f32 * DIALOG_ACTION_CHAR_WIDTH + 20.0).max(DIALOG_ACTION_MIN_WIDTH)
}

fn non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}
