use super::super::super::data::TemplatePaneNodeData;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TREE_ROW_TEXT_NORMAL:
    [u8; 4] = [168, 178, 183, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TREE_ROW_TEXT_SELECTED:
    [u8; 4] = [204, 232, 234, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TREE_ROW_ICON_MUTED: [u8;
    4] = [143, 163, 172, 255];
pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_TREE_ROW_ACTION: [u8; 4] =
    [156, 173, 182, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchTreeRowStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub text: [u8; 4],
    pub icon: [u8; 4],
    pub secondary: [u8; 4],
    pub action: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_tree_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTreeRowStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::TreeRow);
    let marked = node.selected || node.checked;

    WorkbenchTreeRowStyle {
        background: tree_row_background(state, marked),
        border: tree_row_border(state, marked),
        border_width: tree_row_border_width(state, marked),
        text: tree_row_primary_color(state, marked),
        icon: tree_row_icon_color(state, marked),
        secondary: tree_row_secondary_color(state, marked),
        action: tree_row_action_color(state, marked),
        state,
    }
}

fn tree_row_background(state: UiPainterResolvedState, marked: bool) -> Option<[u8; 4]> {
    if state == UiPainterResolvedState::Disabled {
        None
    } else if marked {
        Some(PALETTE.surface_selected)
    } else if state == UiPainterResolvedState::Pressed {
        Some(PALETTE.surface_pressed)
    } else if is_hot(state) {
        Some(PALETTE.surface_hover)
    } else {
        None
    }
}

fn tree_row_border(state: UiPainterResolvedState, marked: bool) -> Option<[u8; 4]> {
    (state != UiPainterResolvedState::Disabled && (marked || is_focus_or_press(state)))
        .then_some(PALETTE.focus_ring)
}

fn tree_row_border_width(state: UiPainterResolvedState, marked: bool) -> f32 {
    if tree_row_border(state, marked).is_some() {
        1.0
    } else {
        0.0
    }
}

fn tree_row_primary_color(state: UiPainterResolvedState, marked: bool) -> [u8; 4] {
    if state == UiPainterResolvedState::Disabled {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_TEXT_NORMAL
    }
}

fn tree_row_icon_color(state: UiPainterResolvedState, marked: bool) -> [u8; 4] {
    if state == UiPainterResolvedState::Disabled {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_ICON_MUTED
    }
}

fn tree_row_secondary_color(state: UiPainterResolvedState, marked: bool) -> [u8; 4] {
    if state == UiPainterResolvedState::Disabled {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        PALETTE.text_muted
    }
}

fn tree_row_action_color(state: UiPainterResolvedState, marked: bool) -> [u8; 4] {
    if state == UiPainterResolvedState::Disabled {
        PALETTE.text_disabled
    } else if marked {
        WORKBENCH_TREE_ROW_TEXT_SELECTED
    } else {
        WORKBENCH_TREE_ROW_ACTION
    }
}

fn is_focus_or_press(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

fn is_hot(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}
