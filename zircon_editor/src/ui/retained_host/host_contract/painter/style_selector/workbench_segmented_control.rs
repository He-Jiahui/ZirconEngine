use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_SEGMENT_IDLE_BACKGROUND:
    [u8; 4] = [29, 35, 39, 255];
const WORKBENCH_SEGMENT_GROUP_LABEL_COLOR: [u8; 4] = [161, 172, 178, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchSegmentedControlKind {
    SegmentedControl,
    Tab,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchSegmentedControlStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub selected_surface: [u8; 4],
    pub selected_border: [u8; 4],
    pub selected_border_width: f32,
    pub selected_underline: [u8; 4],
    pub selected_underline_height: f32,
    pub selected_text: [u8; 4],
    pub idle_text: [u8; 4],
    pub group_label: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_segmented_control_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
) -> WorkbenchSegmentedControlStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tab);
    WorkbenchSegmentedControlStyle {
        background: control_background(node, kind, state),
        border: control_border(kind, state),
        border_width: control_border_width(kind),
        selected_surface: selected_segment_surface_color(state),
        selected_border: selected_segment_border_color(state),
        selected_border_width: selected_segment_border_width(node),
        selected_underline: selected_segment_underline_color(node, state),
        selected_underline_height: selected_segment_underline_height(node),
        selected_text: selected_text_color(state),
        idle_text: idle_text_color(state),
        group_label: group_label_color(node, state),
        state,
    }
}

fn control_background(
    node: &TemplatePaneNodeData,
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            Some(PALETTE.surface_disabled)
        }
        UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => match kind {
            WorkbenchSegmentedControlKind::SegmentedControl => {
                Some(WORKBENCH_SEGMENT_IDLE_BACKGROUND)
            }
            WorkbenchSegmentedControlKind::Tab => {
                resolved_style_color(node.button_style.element.background_color.as_ref())
            }
        },
    }
}

fn control_border(
    kind: WorkbenchSegmentedControlKind,
    state: UiPainterResolvedState,
) -> Option<[u8; 4]> {
    match kind {
        WorkbenchSegmentedControlKind::Tab => None,
        WorkbenchSegmentedControlKind::SegmentedControl => Some(match state {
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.border_disabled
            }
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.accent,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => PALETTE.border,
        }),
    }
}

fn control_border_width(kind: WorkbenchSegmentedControlKind) -> f32 {
    match kind {
        WorkbenchSegmentedControlKind::SegmentedControl => 1.0,
        WorkbenchSegmentedControlKind::Tab => 0.0,
    }
}

fn selected_text_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text
    }
}

fn idle_text_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text_muted
    }
}

fn selected_segment_surface_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        PALETTE.surface_disabled
    } else {
        PALETTE.surface_selected
    }
}

fn selected_segment_border_color(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        PALETTE.border_disabled
    } else {
        PALETTE.accent
    }
}

fn selected_segment_border_width(node: &TemplatePaneNodeData) -> f32 {
    if node.has_selected_segment_border_width {
        finite_non_negative(node.selected_segment_border_width).unwrap_or(0.0)
    } else {
        1.0
    }
}

fn selected_segment_underline_height(node: &TemplatePaneNodeData) -> f32 {
    finite_non_negative(node.selected_segment_underline_height).unwrap_or(0.0)
}

fn selected_segment_underline_color(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        PALETTE.text_disabled
    } else if node.selected_segment_underline_color.a > 0 {
        [
            node.selected_segment_underline_color.r,
            node.selected_segment_underline_color.g,
            node.selected_segment_underline_color.b,
            node.selected_segment_underline_color.a,
        ]
    } else {
        PALETTE.accent
    }
}

fn finite_non_negative(value: f32) -> Option<f32> {
    value.is_finite().then_some(value.max(0.0))
}

fn group_label_color(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_segmented_state(state) {
        return PALETTE.text_disabled;
    }
    let base = if node.label_color.a > 0 {
        [
            node.label_color.r,
            node.label_color.g,
            node.label_color.b,
            node.label_color.a,
        ]
    } else {
        WORKBENCH_SEGMENT_GROUP_LABEL_COLOR
    };
    color_with_brightness(base, node.label_brightness)
}

fn color_with_brightness(mut color: [u8; 4], brightness: f32) -> [u8; 4] {
    let brightness = if brightness.is_finite() && brightness > 0.0 {
        brightness
    } else {
        1.0
    };
    for channel in &mut color[0..3] {
        *channel = ((*channel as f32 * brightness).round()).clamp(0.0, 255.0) as u8;
    }
    color
}

fn is_unavailable_segmented_state(state: UiPainterResolvedState) -> bool {
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
    fn segmented_and_tab_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.checked = true;
        node.selected = true;
        node.button_style.loading = true;
        node.label_color = Color::from_rgb_u8(161, 172, 178);
        node.selected_segment_underline_height = 1.0;
        node.selected_segment_underline_color = Color::from_argb_u8(255, 53, 199, 208);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(29, 35, 39, 255)));

        let segmented = select_workbench_segmented_control_style(
            &node,
            WorkbenchSegmentedControlKind::SegmentedControl,
        );

        assert_eq!(segmented.state, UiPainterResolvedState::Loading);
        assert_eq!(segmented.background, Some(PALETTE.surface_disabled));
        assert_eq!(segmented.border, Some(PALETTE.border_disabled));
        assert_eq!(segmented.selected_surface, PALETTE.surface_disabled);
        assert_eq!(segmented.selected_border, PALETTE.border_disabled);
        assert_eq!(segmented.selected_underline, PALETTE.text_disabled);
        assert_eq!(segmented.selected_text, PALETTE.text_disabled);
        assert_eq!(segmented.idle_text, PALETTE.text_disabled);
        assert_eq!(segmented.group_label, PALETTE.text_disabled);

        let tab =
            select_workbench_segmented_control_style(&node, WorkbenchSegmentedControlKind::Tab);

        assert_eq!(tab.state, UiPainterResolvedState::Loading);
        assert_eq!(tab.background, Some(PALETTE.surface_disabled));
        assert_eq!(tab.border, None);
        assert_eq!(tab.selected_underline, PALETTE.text_disabled);
        assert_eq!(tab.selected_text, PALETTE.text_disabled);
        assert_eq!(tab.idle_text, PALETTE.text_disabled);
    }
}
