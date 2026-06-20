use super::colors::declared_color;
use super::palette::WORKBENCH_SLIDER_TEXT;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_label_color(
    node: &TemplatePaneNodeData,
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        declared_color(node.label_color).unwrap_or(WORKBENCH_SLIDER_TEXT)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn slider_value_text(
    unavailable: bool,
) -> [u8; 4] {
    if unavailable {
        PALETTE.text_disabled
    } else {
        WORKBENCH_SLIDER_TEXT
    }
}
