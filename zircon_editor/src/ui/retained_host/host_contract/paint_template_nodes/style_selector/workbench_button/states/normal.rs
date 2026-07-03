use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn normal_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let palette = current_host_palette();
    match kind {
        WorkbenchButtonKind::Primary => WorkbenchButtonStyle {
            surface: palette.surface_pressed,
            border: palette.border,
            border_width: 1.0,
            text: palette.text,
            glyph: palette.text,
            interaction,
        },
        WorkbenchButtonKind::Secondary => WorkbenchButtonStyle {
            surface: palette.surface_pressed,
            border: palette.border,
            border_width: 1.0,
            text: palette.text,
            glyph: palette.text,
            interaction,
        },
        WorkbenchButtonKind::Tertiary => WorkbenchButtonStyle {
            surface: [0, 0, 0, 0],
            border: [0, 0, 0, 0],
            border_width: 1.0,
            text: palette.text_muted,
            glyph: palette.text_muted,
            interaction,
        },
        WorkbenchButtonKind::Danger => WorkbenchButtonStyle {
            surface: palette.surface_pressed,
            border: palette.border,
            border_width: 1.0,
            text: palette.error,
            glyph: palette.error,
            interaction,
        },
    }
}
