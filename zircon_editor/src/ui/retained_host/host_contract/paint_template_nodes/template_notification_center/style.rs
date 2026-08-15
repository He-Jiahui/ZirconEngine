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
mod tests {
    use super::super::super::super::paint_theme::PALETTE;
    use super::*;

    fn row() -> TemplatePaneOptionData {
        TemplatePaneOptionData::default()
    }

    #[test]
    fn palette_projects_each_notification_role_from_the_host_theme() {
        let mut host = PALETTE;
        host.popup = [1, 2, 3, 4];
        host.border = [5, 6, 7, 8];
        host.text = [9, 10, 11, 12];
        host.text_muted = [13, 14, 15, 16];
        host.surface_inset = [17, 18, 19, 20];
        host.accent_soft = [21, 22, 23, 24];
        host.surface_disabled = [25, 26, 27, 28];
        host.focus_ring = [29, 30, 31, 32];
        host.accent = [33, 34, 35, 36];
        host.error = [37, 38, 39, 40];
        host.success = [41, 42, 43, 44];
        host.warning = [45, 46, 47, 48];

        let palette = notification_center_palette_from_host(host);

        assert_eq!(palette.panel_surface, [1, 2, 3, 4]);
        assert_eq!(palette.panel_border, [5, 6, 7, 8]);
        assert_eq!(palette.header_text, [9, 10, 11, 12]);
        assert_eq!(palette.muted_text, [13, 14, 15, 16]);
        assert_eq!(palette.row_surface, [17, 18, 19, 20]);
        assert_eq!(palette.row_unread_surface, [21, 22, 23, 24]);
        assert_eq!(palette.row_disabled_surface, [25, 26, 27, 28]);
        assert_eq!(palette.row_border, [5, 6, 7, 8]);
        assert_eq!(palette.row_focus_border, [29, 30, 31, 32]);
        assert_eq!(palette.accent, [33, 34, 35, 36]);
        assert_eq!(palette.error, [37, 38, 39, 40]);
        assert_eq!(palette.success, [41, 42, 43, 44]);
        assert_eq!(palette.warning, [45, 46, 47, 48]);
    }

    #[test]
    fn notification_row_state_priority_keeps_disabled_and_selected_explicit() {
        let palette = current_notification_center_palette();
        let disabled = TemplatePaneOptionData {
            disabled: true,
            unread: true,
            focused: true,
            ..row()
        };
        let selected = TemplatePaneOptionData {
            selected: true,
            focused: true,
            ..row()
        };

        assert_eq!(
            row_background(&disabled, palette),
            palette.row_disabled_surface
        );
        assert_eq!(row_border(&selected, palette), palette.accent);
        assert_eq!(
            severity_color("warning", palette),
            palette.warning,
            "semantic tones must not use local RGB literals"
        );
    }
}
