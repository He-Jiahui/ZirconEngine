mod interactive;
mod normal;
mod unavailable;

use super::model::{WorkbenchButtonKind, WorkbenchButtonStyle};
use interactive::{focused_button_style, hover_button_style, pressed_button_style};
use normal::normal_button_style;
use unavailable::unavailable_button_style;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use unavailable::is_unavailable_button_interaction;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn base_button_style(
    kind: WorkbenchButtonKind,
    interaction: ButtonInteractionState,
) -> WorkbenchButtonStyle {
    match interaction {
        ButtonInteractionState::Disabled | ButtonInteractionState::Loading => {
            unavailable_button_style(interaction)
        }
        ButtonInteractionState::Normal => normal_button_style(kind, interaction),
        ButtonInteractionState::Hover => hover_button_style(kind, interaction),
        ButtonInteractionState::Pressed => pressed_button_style(kind, interaction),
        ButtonInteractionState::Focused => focused_button_style(kind, interaction),
    }
}
