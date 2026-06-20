use super::super::super::template_style_color::resolved_style_color;
use super::colors::declared_color;
use super::palette::{
    WORKBENCH_SLIDER_TICK, WORKBENCH_SLIDER_TRACK, WORKBENCH_SLIDER_TRACK_DISABLED,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_track_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        WORKBENCH_SLIDER_TRACK_DISABLED
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or(WORKBENCH_SLIDER_TRACK)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_fill_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else if matches!(node.validation_level.as_str(), "warning") {
        PALETTE.warning
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else {
        PALETTE.accent
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_tick_color(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.border_disabled
    } else {
        WORKBENCH_SLIDER_TICK
    }
}
