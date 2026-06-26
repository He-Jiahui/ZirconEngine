use super::super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use super::super::palette::{
    DANGER_SURFACE_HOVER, DANGER_SURFACE_PRESSED, OUTLINED_SURFACE_HOVER, OUTLINED_SURFACE_PRESSED,
    PRIMARY_SURFACE_HOVER, PRIMARY_SURFACE_PRESSED,
};
use super::normal::normal_button_style;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn hover_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = PRIMARY_SURFACE_HOVER;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = OUTLINED_SURFACE_HOVER;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = PALETTE.surface_hover;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = DANGER_SURFACE_HOVER;
        }
    }
    style
}

pub(super) fn pressed_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = normal_button_style(kind, interaction);
    match kind {
        WorkbenchButtonKind::Primary => {
            style.surface = PRIMARY_SURFACE_PRESSED;
        }
        WorkbenchButtonKind::Secondary => {
            style.surface = OUTLINED_SURFACE_PRESSED;
        }
        WorkbenchButtonKind::Tertiary => {
            style.surface = PALETTE.popup;
            style.text = PALETTE.text;
            style.glyph = PALETTE.text;
        }
        WorkbenchButtonKind::Danger => {
            style.surface = DANGER_SURFACE_PRESSED;
            style.border = PALETTE.error;
        }
    }
    style
}

pub(super) fn focused_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    let mut style = hover_button_style(kind, interaction);
    style.border = if kind == WorkbenchButtonKind::Danger {
        PALETTE.error
    } else {
        PALETTE.focus_ring
    };
    style
}
