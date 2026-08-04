use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::super::palette::workbench_button_palette;
use super::normal::normal_button_style;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn hover_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let button_palette = workbench_button_palette();
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = button_palette.surface_primary_hover;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = button_palette.surface_hover;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = button_palette.surface_hover;
            style.text = button_palette.text;
            style.glyph = button_palette.text;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = button_palette.surface_hover;
        }
    }
    style
}

pub(super) fn pressed_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let button_palette = workbench_button_palette();
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = button_palette.surface_primary_pressed;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = button_palette.surface_secondary_pressed;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = button_palette.surface_tertiary_pressed;
            style.text = button_palette.text;
            style.glyph = button_palette.text;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = button_palette.surface_danger_pressed;
        }
    }
    style
}

pub(super) fn focused_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
    focus_uses_hover_surface: bool,
) -> WorkbenchButtonStyle {
    let button_palette = workbench_button_palette();
    let mut style = if focus_uses_hover_surface {
        hover_button_style(kind, interaction)
    } else {
        normal_button_style(kind, interaction)
    };
    style.border = button_palette.focus_border;
    style
}
