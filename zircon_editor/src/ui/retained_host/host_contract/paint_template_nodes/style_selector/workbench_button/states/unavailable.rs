use super::super::metrics::workbench_button_border_width;
use super::super::model::WorkbenchButtonStyle;
use super::super::palette::workbench_button_palette;
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
    let button_palette = workbench_button_palette();
    WorkbenchButtonStyle {
        surface: button_palette.disabled_surface,
        border: button_palette.disabled_border,
        border_width: workbench_button_border_width(),
        text: button_palette.disabled_text,
        glyph: button_palette.disabled_text,
        interaction,
    }
}
