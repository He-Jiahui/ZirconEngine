use super::super::super::data::TemplatePaneOptionData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PANEL_SURFACE: [u8; 4] =
    [17, 24, 29, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const PANEL_BORDER: [u8; 4] =
    [45, 58, 66, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const HEADER_TEXT: [u8; 4] =
    [231, 238, 240, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const MUTED_TEXT: [u8; 4] =
    [127, 143, 149, 255];

const ROW_SURFACE: [u8; 4] = [21, 30, 35, 255];
const ROW_UNREAD_SURFACE: [u8; 4] = [21, 48, 53, 255];
const ROW_FOCUSED_SURFACE: [u8; 4] = [24, 58, 63, 255];
const ROW_DISABLED_SURFACE: [u8; 4] = [37, 44, 49, 255];
const ROW_BORDER: [u8; 4] = [40, 56, 66, 255];
const ACCENT: [u8; 4] = [53, 199, 208, 255];
const ERROR: [u8; 4] = [239, 112, 102, 255];
const SUCCESS: [u8; 4] = [66, 184, 131, 255];
const WARNING: [u8; 4] = [224, 163, 58, 255];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_background(
    option: &TemplatePaneOptionData,
) -> [u8; 4] {
    if option.disabled {
        ROW_DISABLED_SURFACE
    } else if option.focused {
        ROW_FOCUSED_SURFACE
    } else if option.unread {
        ROW_UNREAD_SURFACE
    } else {
        ROW_SURFACE
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn row_border(
    option: &TemplatePaneOptionData,
) -> [u8; 4] {
    if option.selected {
        ACCENT
    } else {
        ROW_BORDER
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn title_color(
    option: &TemplatePaneOptionData,
) -> [u8; 4] {
    if option.disabled {
        MUTED_TEXT
    } else {
        HEADER_TEXT
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn severity_color(
    tone: &str,
) -> [u8; 4] {
    match tone {
        "success" => SUCCESS,
        "warning" => WARNING,
        "error" => ERROR,
        _ => ACCENT,
    }
}
