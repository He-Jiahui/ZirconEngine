use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchListRowStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub text: [u8; 4],
    pub adornment: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_list_row_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchListRowStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::ListRow);
    let marked = node.checked || node.selected;
    WorkbenchListRowStyle {
        background: list_row_background(node, state, marked),
        border: list_row_border(state),
        border_width: list_row_border_width(state),
        text: list_row_text_color(node, state, marked),
        adornment: list_row_adornment_color(node, state, marked),
        state,
    }
}

fn list_row_background(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_list_row_state(state) {
        None
    } else if marked {
        declared_background_color(node).or(Some(PALETTE.surface_selected))
    } else {
        match state {
            UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
            UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
            UiPainterResolvedState::Disabled
            | UiPainterResolvedState::Loading
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => None,
        }
    }
}

fn list_row_border(state: UiPainterResolvedState) -> Option<[u8; 4]> {
    matches!(
        state,
        UiPainterResolvedState::Focused
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
    .then_some(PALETTE.focus_ring)
}

fn list_row_border_width(state: UiPainterResolvedState) -> f32 {
    if list_row_border(state).is_some() {
        1.0
    } else {
        0.0
    }
}

fn list_row_text_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if marked {
        PALETTE.text
    } else {
        PALETTE.text_muted
    }
}

fn list_row_adornment_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    marked: bool,
) -> [u8; 4] {
    if is_unavailable_list_row_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if marked {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}

fn declared_background_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn is_unavailable_list_row_state(state: UiPainterResolvedState) -> bool {
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
    fn list_row_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.selected = true;
        node.checked = true;
        node.button_style.loading = true;
        node.value_color = Color::from_rgb_u8(53, 199, 208);
        node.icon_color = Color::from_rgb_u8(122, 230, 240);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(13, 65, 73, 255)));

        let style = select_workbench_list_row_style(&node);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.background, None);
        assert_eq!(style.border, None);
        assert_eq!(style.text, PALETTE.text_disabled);
        assert_eq!(style.adornment, PALETTE.text_disabled);
    }
}
