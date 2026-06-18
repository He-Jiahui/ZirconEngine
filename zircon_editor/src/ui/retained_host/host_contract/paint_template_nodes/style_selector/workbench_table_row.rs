use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_ROW_BG: [u8; 4] =
    [13, 17, 20, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_BG: [u8; 4] =
    [12, 16, 19, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_TAIL_BG: [u8; 4] =
    [14, 18, 21, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SELECTED_BG: [u8;
    4] = [13, 65, 73, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HOVER_BG: [u8; 4] =
    [24, 44, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_SEPARATOR: [u8; 4] =
    [28, 36, 41, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_ACTION_MUTED: [u8;
    4] = [116, 130, 137, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TABLE_HEADER_TEXT: [u8;
    4] = [170, 181, 186, 255];
const WORKBENCH_TABLE_TAIL_VALUE_TEXT: [u8; 4] = [170, 181, 186, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTableRowStyle
{
    pub background: [u8; 4],
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub separator: [u8; 4],
    pub action: [u8; 4],
    pub state: UiPainterResolvedState,
    text: [u8; 4],
    muted_text: [u8; 4],
    tail_value_text: [u8; 4],
    header: bool,
    tail: bool,
}

impl WorkbenchTableRowStyle {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_for_cell(
        self,
        index: usize,
    ) -> [u8; 4] {
        if is_unavailable_table_row_state(self.state) {
            PALETTE.text_disabled
        } else if self.header {
            WORKBENCH_TABLE_HEADER_TEXT
        } else if self.tail && index == 3 {
            self.tail_value_text
        } else if index >= 2 {
            self.muted_text
        } else {
            self.text
        }
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_table_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTableRowStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::TableRow);
    let marked = node.selected || node.checked;
    let header = is_table_header(node);
    let tail = is_table_tail(node);

    WorkbenchTableRowStyle {
        background: table_row_background(node, state, marked, header, tail),
        border: table_row_border(state),
        border_width: table_row_border_width(state),
        separator: WORKBENCH_TABLE_SEPARATOR,
        action: table_row_action_color(state),
        state,
        text: PALETTE.text,
        muted_text: PALETTE.text_muted,
        tail_value_text: declared_value_color(node).unwrap_or(WORKBENCH_TABLE_TAIL_VALUE_TEXT),
        header,
        tail,
    }
}

fn table_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
    header: bool,
    tail: bool,
) -> [u8; 4] {
    if is_unavailable_table_row_state(state) {
        PALETTE.surface_disabled
    } else if marked {
        declared_background_color(node).unwrap_or(WORKBENCH_TABLE_SELECTED_BG)
    } else if state == UiPainterResolvedState::Pressed {
        PALETTE.surface_pressed
    } else if is_hot(state) {
        WORKBENCH_TABLE_HOVER_BG
    } else if header {
        WORKBENCH_TABLE_HEADER_BG
    } else {
        declared_background_color(node).unwrap_or_else(|| {
            if tail {
                WORKBENCH_TABLE_TAIL_BG
            } else {
                WORKBENCH_TABLE_ROW_BG
            }
        })
    }
}

fn table_row_border(state: UiPainterResolvedState) -> Option<[u8; 4]> {
    if is_unavailable_table_row_state(state) {
        return None;
    }
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(PALETTE.focus_ring)
}

fn table_row_border_width(state: UiPainterResolvedState) -> f32 {
    if table_row_border(state).is_some() {
        1.0
    } else {
        0.0
    }
}

fn table_row_action_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_table_row_state(state) {
        PALETTE.text_disabled
    } else {
        WORKBENCH_TABLE_ACTION_MUTED
    }
}

fn declared_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

fn declared_value_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    (node.value_color.a > 0).then_some([
        node.value_color.r,
        node.value_color.g,
        node.value_color.b,
        node.value_color.a,
    ])
}

fn is_table_header(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTableHeader"
}

fn is_table_tail(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchTableTail"
}

fn is_hot(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

fn is_unavailable_table_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::Color;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn table_row_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.control_id = "WorkbenchTableSelected".into();
        node.hovered = true;
        node.selected = true;
        node.button_style.loading = true;
        node.value_color = Color::from_rgb_u8(170, 181, 186);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

        let style = select_workbench_table_row_style(&node);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.background, PALETTE.surface_disabled);
        assert_eq!(style.border, None);
        assert_eq!(style.border_width, 0.0);
        assert_eq!(style.action, PALETTE.text_disabled);
        assert_eq!(style.text_for_cell(0), PALETTE.text_disabled);
        assert_eq!(style.text_for_cell(3), PALETTE.text_disabled);
    }
}
