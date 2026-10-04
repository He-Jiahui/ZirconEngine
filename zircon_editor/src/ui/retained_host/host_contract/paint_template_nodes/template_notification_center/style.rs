//! 将宿主palette映射到通知面板角色，并按投影的未读、选中、焦点和严重性挑选颜色。
//! 视觉优先级不改变通知的实际已读或选中状态；输入和清除仍归交互owner。

use super::super::super::data::TemplatePaneOptionData;
use super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct NotificationCenterPalette
{
    pub panel_surface: [u8; 4],
    pub panel_border: [u8; 4],
    pub header_text: [u8; 4],
    pub muted_text: [u8; 4],
    pub row_surface: [u8; 4],
    pub row_unread_surface: [u8; 4],
    pub row_disabled_surface: [u8; 4],
    pub row_border: [u8; 4],
    pub row_focus_border: [u8; 4],
    pub accent: [u8; 4],
    pub error: [u8; 4],
    pub success: [u8; 4],
    pub warning: [u8; 4],
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn current_notification_center_palette(
) -> NotificationCenterPalette {
    notification_center_palette_from_host(current_host_palette())
}

fn notification_center_palette_from_host(
    palette: HostMaterialPalette,
) -> NotificationCenterPalette {
    NotificationCenterPalette {
        panel_surface: palette.popup,
        panel_border: palette.border,
        header_text: palette.text,
        muted_text: palette.text_muted,
        row_surface: palette.surface_inset,
        row_unread_surface: palette.accent_soft,
        row_disabled_surface: palette.surface_disabled,
        row_border: palette.border,
        row_focus_border: palette.focus_ring,
        accent: palette.accent,
        error: palette.error,
        success: palette.success,
        warning: palette.warning,
    }
}

pub(super) fn row_background(
    option: &TemplatePaneOptionData,
    palette: NotificationCenterPalette,
) -> [u8; 4] {
    if option.disabled {
        palette.row_disabled_surface
    } else if option.unread {
        palette.row_unread_surface
    } else {
        palette.row_surface
    }
}

pub(super) fn row_border(
    option: &TemplatePaneOptionData,
    palette: NotificationCenterPalette,
) -> [u8; 4] {
    if option.selected {
        palette.accent
    } else if option.focused {
        palette.row_focus_border
    } else {
        palette.row_border
    }
}

pub(super) fn title_color(
    option: &TemplatePaneOptionData,
    palette: NotificationCenterPalette,
) -> [u8; 4] {
    if option.disabled {
        palette.muted_text
    } else {
        palette.header_text
    }
}

pub(super) fn severity_color(tone: &str, palette: NotificationCenterPalette) -> [u8; 4] {
    match tone {
        "success" => palette.success,
        "warning" => palette.warning,
        "error" => palette.error,
        _ => palette.accent,
    }
}

#[cfg(test)]
#[path = "tests/style.rs"]
mod tests;
