use super::super::super::super::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TreeIconKind {
    Cube,
    PlayerStart,
    Audio,
}

pub(super) fn tree_icon_kind(node: &TemplatePaneNodeData) -> TreeIconKind {
    let id = node.control_id.as_str();
    let label = node.text.as_str();
    if id.contains("Audio") || label.contains("Audio") {
        TreeIconKind::Audio
    } else if id.contains("Player") || label.contains("Player") {
        TreeIconKind::PlayerStart
    } else {
        TreeIconKind::Cube
    }
}

pub(super) fn is_unavailable_tree_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
