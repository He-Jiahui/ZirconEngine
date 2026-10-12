//! 基础按钮配方只按交互状态与按钮类型取色；特殊命令、标签和声明色由外层选择器随后决定。

#[path = "../../../../../contract/paint/selector/button/states/interactive.rs"]
mod interactive;
#[path = "states/normal.rs"]
mod normal;
#[path = "../../../../../contract/paint/selector/button/states/unavailable.rs"]
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
    focus_uses_hover_surface: bool,
) -> WorkbenchButtonStyle {
    match interaction {
        ButtonInteractionState::Disabled | ButtonInteractionState::Loading => {
            unavailable_button_style(interaction)
        }
        ButtonInteractionState::Normal => normal_button_style(kind, interaction),
        ButtonInteractionState::Hover => hover_button_style(kind, interaction),
        ButtonInteractionState::Pressed => pressed_button_style(kind, interaction),
        ButtonInteractionState::Focused => {
            focused_button_style(kind, interaction, focus_uses_hover_surface)
        }
    }
}
