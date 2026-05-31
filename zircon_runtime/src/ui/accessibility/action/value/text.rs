use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityNode, component::UiValue, event_ui::UiNodeId,
    surface::UiTextRange,
};

use crate::ui::surface::{text_input_constraints_for_node, UiSurface};

use super::super::text_state::text_input_is_read_only;

pub(super) struct TextInputSetValue {
    pub(super) value: UiValue,
    pub(super) constraint_note: Option<&'static str>,
}

pub(super) struct TextInputSetValueRejection {
    pub(super) code: &'static str,
    pub(super) reason: &'static str,
}

pub(super) fn prepare_text_input_set_value(
    surface: &UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    value: UiValue,
) -> Result<TextInputSetValue, TextInputSetValueRejection> {
    if text_input_is_read_only(surface, target) {
        return Err(TextInputSetValueRejection {
            code: "read_only",
            reason: "text input is read-only",
        });
    }

    let mut value = value;
    let mut constraint_note = None;
    if let UiValue::String(text) = &value {
        let current_text = snapshot_node.state.value.as_deref().unwrap_or_default();
        let sanitized = text_input_constraints_for_node(surface, target).sanitize_replacement(
            current_text,
            UiTextRange {
                start: 0,
                end: current_text.len(),
            },
            text,
        );
        if sanitized != *text {
            constraint_note = Some("accessibility_text_value_sanitized");
            value = UiValue::String(sanitized);
        }
    }

    Ok(TextInputSetValue {
        value,
        constraint_note,
    })
}
