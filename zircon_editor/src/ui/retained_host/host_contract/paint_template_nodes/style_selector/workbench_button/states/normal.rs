use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::super::palette::{
    DANGER_BORDER, DANGER_SURFACE, DANGER_TEXT, OUTLINED_BORDER, OUTLINED_SURFACE, OUTLINED_TEXT,
    PRIMARY_SURFACE, PRIMARY_TEXT, TERTIARY_TEXT,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn normal_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    match kind {
        WorkbenchButtonKind::Primary => WorkbenchButtonStyle {
            surface: PRIMARY_SURFACE,
            border: [36, 154, 166, 255],
            border_width: 1.0,
            text: PRIMARY_TEXT,
            glyph: PRIMARY_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Secondary => WorkbenchButtonStyle {
            surface: OUTLINED_SURFACE,
            border: OUTLINED_BORDER,
            border_width: 1.0,
            text: OUTLINED_TEXT,
            glyph: OUTLINED_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Tertiary => WorkbenchButtonStyle {
            surface: PALETTE.surface_inset,
            border: PALETTE.border,
            border_width: 1.0,
            text: TERTIARY_TEXT,
            glyph: TERTIARY_TEXT,
            interaction,
        },
        WorkbenchButtonKind::Danger => WorkbenchButtonStyle {
            surface: DANGER_SURFACE,
            border: DANGER_BORDER,
            border_width: 1.0,
            text: DANGER_TEXT,
            glyph: DANGER_TEXT,
            interaction,
        },
    }
}
