use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::resolved_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_ICON_COLOR: [u8;
    4] = [149, 164, 172, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_ICON_MUTED: [u8;
    4] = [105, 121, 130, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_RIGHT_BORDER: [u8;
    4] = [36, 44, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_NO_ERRORS_FILL:
    [u8; 4] = [88, 184, 102, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_STATUS_MARK_ON_LIGHT:
    [u8; 4] = [8, 18, 18, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum WorkbenchStatusSignalKind
{
    Ready,
    Success,
    Warning,
    Info,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusSignalStyle
{
    pub icon_fill: [u8; 4],
    pub text: [u8; 4],
    pub mark: [u8; 4],
    pub state: UiPainterResolvedState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusChipStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub state: UiPainterResolvedState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchStatusIconButtonStyle
{
    pub background: [u8; 4],
    pub border: [u8; 4],
    pub glyph: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_signal_style(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
) -> WorkbenchStatusSignalStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);

    WorkbenchStatusSignalStyle {
        icon_fill: status_signal_icon_fill(node, kind, state),
        text: status_signal_text_color(node, kind, state),
        mark: status_signal_mark_color(node, state),
        state,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_chip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusChipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Generic);

    WorkbenchStatusChipStyle {
        background: status_chip_background(state),
        border: status_chip_border(state),
        text: status_chip_text_color(node, state),
        state,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_status_icon_button_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchStatusIconButtonStyle {
    let state =
        resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);

    WorkbenchStatusIconButtonStyle {
        background: status_icon_button_background(state),
        border: status_icon_button_border(state),
        glyph: status_icon_glyph_color(state),
        state,
    }
}

fn status_signal_icon_fill(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = declared_color(node.label_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => PALETTE.success,
        WorkbenchStatusSignalKind::Success => WORKBENCH_STATUS_NO_ERRORS_FILL,
        WorkbenchStatusSignalKind::Warning => PALETTE.warning,
        WorkbenchStatusSignalKind::Info => PALETTE.info,
    }
}

fn status_signal_mark_color(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        PALETTE.text_disabled
    } else {
        declared_color(node.icon_color).unwrap_or(WORKBENCH_STATUS_MARK_ON_LIGHT)
    }
}

fn status_signal_text_color(
    node: &TemplatePaneNodeData,
    kind: WorkbenchStatusSignalKind,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = declared_color(node.value_color) {
        return color;
    }
    match kind {
        WorkbenchStatusSignalKind::Ready => PALETTE.text,
        WorkbenchStatusSignalKind::Success
        | WorkbenchStatusSignalKind::Warning
        | WorkbenchStatusSignalKind::Info => PALETTE.text_muted,
    }
}

fn status_chip_background(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.surface_disabled
        }
        UiPainterResolvedState::Pressed => PALETTE.surface_pressed,
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            PALETTE.surface_selected
        }
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => PALETTE.surface_hover,
        UiPainterResolvedState::Normal => PALETTE.surface_inset,
    }
}

fn status_chip_border(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.border_disabled
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            WORKBENCH_STATUS_RIGHT_BORDER
        }
    }
}

fn status_chip_text_color(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_status_state(state) {
        PALETTE.text_disabled
    } else {
        declared_color(node.value_color).unwrap_or(PALETTE.text_muted)
    }
}

fn status_icon_button_background(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.surface_disabled
        }
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            PALETTE.surface_selected
        }
        UiPainterResolvedState::Pressed => PALETTE.surface_pressed,
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => PALETTE.surface_hover,
        UiPainterResolvedState::Normal => PALETTE.surface_inset,
    }
}

fn status_icon_button_border(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            PALETTE.border_disabled
        }
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered | UiPainterResolvedState::Normal => {
            WORKBENCH_STATUS_RIGHT_BORDER
        }
    }
}

fn status_icon_glyph_color(state: UiPainterResolvedState) -> [u8; 4] {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => PALETTE.text_disabled,
        UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
        UiPainterResolvedState::Hovered => WORKBENCH_STATUS_ICON_COLOR,
        UiPainterResolvedState::Normal => WORKBENCH_STATUS_ICON_MUTED,
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn is_unavailable_status_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_signal_unavailable_states_mute_icon_text_and_mark() {
        let mut disabled = TemplatePaneNodeData::default();
        disabled.disabled = true;
        disabled.hovered = true;
        disabled.label_color = Color::from_rgb_u8(242, 195, 86);
        disabled.value_color = Color::from_rgb_u8(135, 146, 153);
        disabled.icon_color = Color::from_rgb_u8(17, 24, 26);

        let disabled_style =
            select_workbench_status_signal_style(&disabled, WorkbenchStatusSignalKind::Warning);
        assert_eq!(disabled_style.state, UiPainterResolvedState::Disabled);
        assert_eq!(disabled_style.icon_fill, PALETTE.text_disabled);
        assert_eq!(disabled_style.text, PALETTE.text_disabled);
        assert_eq!(disabled_style.mark, PALETTE.text_disabled);

        let mut loading = TemplatePaneNodeData::default();
        loading.hovered = true;
        loading.button_style.loading = true;
        loading.label_color = Color::from_rgb_u8(88, 184, 102);
        loading.value_color = Color::from_rgb_u8(143, 154, 160);
        loading.icon_color = Color::from_rgb_u8(8, 18, 18);

        let loading_style =
            select_workbench_status_signal_style(&loading, WorkbenchStatusSignalKind::Success);
        assert_eq!(loading_style.state, UiPainterResolvedState::Loading);
        assert_eq!(loading_style.icon_fill, PALETTE.text_disabled);
        assert_eq!(loading_style.text, PALETTE.text_disabled);
        assert_eq!(loading_style.mark, PALETTE.text_disabled);
    }
}
