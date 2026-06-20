use super::super::tokens::{DIALOG_ACTION, DIALOG_BODY, DIALOG_DISABLED_TEXT, DIALOG_ERROR};
use super::super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_action_color(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_ACTION
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cancel_action_color(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        DIALOG_DISABLED_TEXT
    } else {
        DIALOG_BODY
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn confirm_action_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    confirm_enabled: bool,
) -> [u8; 4] {
    if unavailable || !confirm_enabled {
        DIALOG_DISABLED_TEXT
    } else if variant_contains_any(node, &["destructive"]) {
        DIALOG_ERROR
    } else {
        DIALOG_ACTION
    }
}
