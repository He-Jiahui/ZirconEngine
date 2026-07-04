use super::super::palette::dialog_palette;
use super::super::variants::variant_contains_any;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dialog_action_color(
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_text
    } else {
        palette.action
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn cancel_action_color(
    unavailable: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable {
        palette.disabled_text
    } else {
        palette.body
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn confirm_action_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    confirm_enabled: bool,
) -> [u8; 4] {
    let palette = dialog_palette();
    if unavailable || !confirm_enabled {
        palette.disabled_text
    } else if variant_contains_any(node, &["destructive"]) {
        palette.error
    } else {
        palette.action
    }
}
