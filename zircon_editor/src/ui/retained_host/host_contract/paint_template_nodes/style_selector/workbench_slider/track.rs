use super::super::super::template_style_color::resolved_style_color;
use super::colors::declared_color;
use super::palette::WorkbenchSliderPalette;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;

pub(super) fn slider_track_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if unavailable {
        palette.track_disabled
    } else {
        resolved_style_color(node.button_style.element.background_color.as_ref())
            .unwrap_or(palette.track)
    }
}

pub(super) fn slider_fill_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
    palette: &WorkbenchSliderPalette,
) -> [u8; 4] {
    if unavailable {
        palette.text_disabled
    } else if matches!(node.validation_level.as_str(), "warning") {
        palette.warning
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        palette.error
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else {
        palette.fill
    }
}

pub(super) fn slider_tick_color(unavailable: bool, palette: &WorkbenchSliderPalette) -> [u8; 4] {
    if unavailable {
        palette.border_disabled
    } else {
        palette.tick
    }
}
