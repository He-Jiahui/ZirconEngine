use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SURFACE: [u8; 4] =
    [23, 28, 32, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BORDER: [u8; 4] =
    [37, 45, 50, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_TITLE: [u8; 4] =
    [208, 217, 221, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_BODY: [u8; 4] =
    [168, 179, 184, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_ICON: [u8; 4] =
    [37, 156, 167, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TOOLTIP_SHADOW: [u8; 4] =
    [0, 0, 0, 96];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTooltipStyle {
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub title: [u8; 4],
    pub body: [u8; 4],
    pub arrow: [u8; 4],
    pub icon: [u8; 4],
    pub shadow: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_tooltip_style(
    node: &TemplatePaneNodeData,
) -> WorkbenchTooltipStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Tooltip);
    let mut style = tooltip_state_style(state);

    if !is_unavailable_tooltip_state(state) {
        if let Some(surface) =
            resolved_style_color(node.button_style.element.background_color.as_ref())
        {
            style.surface = surface;
            if declared_color(node.value_color).is_none() {
                style.arrow = surface;
            }
        }
        if let Some(border) = resolved_style_color(node.button_style.element.border_color.as_ref())
        {
            style.border = border;
        }
        if let Some(title) =
            resolved_style_color(node.button_style.element.foreground_color.as_ref())
        {
            style.title = title;
        }
        if let Some(body) = declared_color(node.label_color) {
            style.body = body;
        }
        if let Some(icon) = declared_color(node.icon_color) {
            style.icon = icon;
        }
        if let Some(arrow) = declared_color(node.value_color) {
            style.arrow = arrow;
        }
    }

    style
}

fn tooltip_state_style(state: UiPainterResolvedState) -> WorkbenchTooltipStyle {
    match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WorkbenchTooltipStyle {
                surface: PALETTE.surface_disabled,
                border: PALETTE.border_disabled,
                title: PALETTE.text_disabled,
                body: PALETTE.text_disabled,
                arrow: PALETTE.surface_disabled,
                icon: PALETTE.text_disabled,
                shadow: [0, 0, 0, 48],
                state,
            }
        }
        UiPainterResolvedState::Pressed | UiPainterResolvedState::Focused => {
            let mut style = tooltip_normal_style(state);
            style.border = PALETTE.focus_ring;
            style.icon = PALETTE.focus_ring;
            style.title = PALETTE.text;
            style
        }
        UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => {
            let mut style = tooltip_normal_style(state);
            style.border = PALETTE.border;
            style.icon = PALETTE.accent;
            style
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => tooltip_normal_style(state),
    }
}

fn is_unavailable_tooltip_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn tooltip_normal_style(state: UiPainterResolvedState) -> WorkbenchTooltipStyle {
    WorkbenchTooltipStyle {
        surface: WORKBENCH_TOOLTIP_SURFACE,
        border: WORKBENCH_TOOLTIP_BORDER,
        title: WORKBENCH_TOOLTIP_TITLE,
        body: WORKBENCH_TOOLTIP_BODY,
        arrow: WORKBENCH_TOOLTIP_SURFACE,
        icon: WORKBENCH_TOOLTIP_ICON,
        shadow: WORKBENCH_TOOLTIP_SHADOW,
        state,
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn tooltip_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.button_style.loading = true;
        node.value_color = Color::from_rgb_u8(23, 28, 32);
        node.label_color = Color::from_rgb_u8(168, 179, 184);
        node.icon_color = Color::from_rgb_u8(37, 156, 167);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(23, 28, 32, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(37, 45, 50, 255)));
        node.button_style.element.foreground_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(208, 217, 221, 255)));

        let style = select_workbench_tooltip_style(&node);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, PALETTE.surface_disabled);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.title, PALETTE.text_disabled);
        assert_eq!(style.body, PALETTE.text_disabled);
        assert_eq!(style.arrow, PALETTE.surface_disabled);
        assert_eq!(style.icon, PALETTE.text_disabled);
    }
}
