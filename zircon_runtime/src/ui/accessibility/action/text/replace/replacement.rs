use zircon_runtime_interface::ui::{accessibility::UiAccessibilityNode, event_ui::UiNodeId};

use crate::ui::surface::{text_input_constraints_for_node, UiSurface};

use super::super::super::text_state::text_selection_range;

pub(super) struct SelectedTextReplacement {
    pub(super) text: String,
    pub(super) caret_offset: usize,
    pub(super) constraint_note: Option<&'static str>,
}

pub(super) fn selected_text_replacement(
    surface: &UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    replacement: &str,
) -> SelectedTextReplacement {
    let current_text = snapshot_node.state.value.as_deref().unwrap_or_default();
    let selected_range =
        text_selection_range(current_text, snapshot_node.state.text_selection.as_ref());
    let sanitized = text_input_constraints_for_node(surface, target).sanitize_replacement(
        current_text,
        selected_range,
        replacement,
    );
    let constraint_note =
        (sanitized != replacement).then_some("accessibility_replace_selected_text_sanitized");
    let mut text = String::with_capacity(
        current_text.len() - (selected_range.end - selected_range.start) + sanitized.len(),
    );
    text.push_str(&current_text[..selected_range.start]);
    text.push_str(&sanitized);
    text.push_str(&current_text[selected_range.end..]);
    let caret_offset = selected_range.start + sanitized.len();
    SelectedTextReplacement {
        text,
        caret_offset,
        constraint_note,
    }
}
