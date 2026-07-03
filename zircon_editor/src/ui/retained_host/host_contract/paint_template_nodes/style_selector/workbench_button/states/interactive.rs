use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::normal::normal_button_style;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn hover_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let palette = current_host_palette();
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = palette.surface_hover;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = palette.surface_hover;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = palette.surface_hover;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = palette.surface_hover;
        }
    }
    style
}

pub(super) fn pressed_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let palette = current_host_palette();
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = palette.surface_selected;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = palette.surface;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = palette.popup;
            style.text = palette.text;
            style.glyph = palette.text;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = palette.surface;
        }
    }
    style
}

pub(super) fn focused_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let palette = current_host_palette();
    let mut style = hover_button_style(kind, interaction);
    style.border = palette.focus_ring;
    style
}
