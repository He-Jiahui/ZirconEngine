use super::palette::WorkbenchChromePalette;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chrome_separator(
    normal: [u8; 4],
    state: UiPainterResolvedState,
    palette: &WorkbenchChromePalette,
) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            palette.border_disabled
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => palette.border,
        UiPainterResolvedState::Hovered => palette.border,
        UiPainterResolvedState::Normal => normal,
    }
}
