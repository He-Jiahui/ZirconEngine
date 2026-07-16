use super::super::metrics::workbench_button_border_width;
use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::super::palette::workbench_button_palette;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn normal_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let button_palette = workbench_button_palette();
    let border_width = workbench_button_border_width();
    match kind {
        WorkbenchButtonKind::Primary => WorkbenchButtonStyle {
            surface: button_palette.surface_primary_rest,
            border: button_palette.border,
            border_width,
            text: button_palette.text,
            glyph: button_palette.text,
            interaction,
        },
        WorkbenchButtonKind::Secondary => WorkbenchButtonStyle {
            surface: button_palette.surface_base,
            border: button_palette.border,
            border_width,
            text: button_palette.text,
            glyph: button_palette.text,
            interaction,
        },
        WorkbenchButtonKind::Tertiary => WorkbenchButtonStyle {
            surface: button_palette.transparent_surface,
            border: button_palette.transparent_surface,
            border_width,
            text: button_palette.text_muted,
            glyph: button_palette.text_muted,
            interaction,
        },
        WorkbenchButtonKind::Danger => WorkbenchButtonStyle {
            surface: button_palette.surface_base,
            border: button_palette.border,
            border_width,
            text: button_palette.danger_text,
            glyph: button_palette.danger_text,
            interaction,
        },
    }
}
