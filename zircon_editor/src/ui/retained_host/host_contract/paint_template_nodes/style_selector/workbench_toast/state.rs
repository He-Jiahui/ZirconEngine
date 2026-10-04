//! Toast 的加载/禁用、按压、焦点和热态先决定外壳及动作基色；后续声明色只在许可通道覆盖。

use super::model::WorkbenchToastStyle;
use super::palette::{
    toast_normal_style_from_palette, workbench_toast_palette, WorkbenchToastPalette,
};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn toast_state_style(
    state: UiPainterResolvedState,
) -> WorkbenchToastStyle {
    toast_state_style_from_palette(state, workbench_toast_palette())
}

fn toast_state_style_from_palette(
    state: UiPainterResolvedState,
    palette: WorkbenchToastPalette,
) -> WorkbenchToastStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => WorkbenchToastStyle {
            surface: palette.disabled_surface,
            border: palette.disabled_border,
            text: palette.disabled_text,
            mark: palette.disabled_text,
            action: palette.disabled_text,
            close: palette.disabled_text,
            state,
        },
        UiPainterResolvedState::Pressed => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.surface = palette.pressed_surface;
            style.border = palette.action;
            style.action = palette.action;
            style
        }
        UiPainterResolvedState::Focused => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.border = palette.focus_border;
            style
        }
        UiPainterResolvedState::Open => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.border = palette.action;
            style.action = palette.action;
            style
        }
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => {
            let mut style = toast_normal_style_from_palette(state, palette);
            style.surface = palette.hover_surface;
            style.border = palette.hover_border;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => toast_normal_style_from_palette(state, palette),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_toast_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
