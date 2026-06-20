use super::super::model::WorkbenchButtonStyle;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_button_interaction(
    interaction: ButtonInteractionState,
) -> bool {
    matches!(
        interaction,
        ButtonInteractionState::Disabled | ButtonInteractionState::Loading
    )
}

pub(super) fn unavailable_button_style(
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    WorkbenchButtonStyle {
        surface: PALETTE.surface_disabled,
        border: PALETTE.border_disabled,
        border_width: 1.0,
        text: PALETTE.text_disabled,
        glyph: PALETTE.text_disabled,
        interaction,
    }
}
