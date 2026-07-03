use super::super::model::WorkbenchButtonStyle;
use crate::ui::retained_host::host_contract::paint_theme::current_host_palette;
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
    let palette = current_host_palette();
    WorkbenchButtonStyle {
        surface: palette.surface_disabled,
        border: palette.border_disabled,
        border_width: 1.0,
        text: palette.text_disabled,
        glyph: palette.text_disabled,
        interaction,
    }
}
