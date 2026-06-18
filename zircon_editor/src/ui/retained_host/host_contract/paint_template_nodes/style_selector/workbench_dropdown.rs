use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState, UiStyleColor};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_SURFACE: [u8; 4] =
    [16, 22, 26, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_HOVER_SURFACE:
    [u8; 4] = [20, 27, 31, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_OPEN_SURFACE:
    [u8; 4] = [15, 24, 28, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_DISABLED_SURFACE:
    [u8; 4] = [25, 29, 34, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_BORDER: [u8; 4] =
    [50, 63, 71, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_FOCUS_BORDER:
    [u8; 4] = [31, 152, 161, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_TEXT: [u8; 4] =
    [205, 216, 221, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_DROPDOWN_PLACEHOLDER:
    [u8; 4] = [122, 134, 142, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchDropdownStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub chevron: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_dropdown_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchDropdownStyle {
    let state = resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::Dropdown);
    let mut style = WorkbenchDropdownStyle {
        surface: dropdown_surface(node, state),
        border: dropdown_border(node, state),
        text: dropdown_text(node, state, label_is_placeholder),
        chevron: dropdown_chevron(node, state),
        state,
    };
    style = apply_visual_brightness(style, node.label_brightness);
    style
}

fn dropdown_surface(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    let color = match state {
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open => WORKBENCH_DROPDOWN_OPEN_SURFACE,
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => WORKBENCH_DROPDOWN_HOVER_SURFACE,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WORKBENCH_DROPDOWN_DISABLED_SURFACE
        }
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => WORKBENCH_DROPDOWN_SURFACE,
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}

fn dropdown_border(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    let color = if is_unavailable_dropdown_state(state) {
        PALETTE.border_disabled
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open => WORKBENCH_DROPDOWN_FOCUS_BORDER,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.border_disabled
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => WORKBENCH_DROPDOWN_BORDER,
        }
    };
    if is_unavailable_dropdown_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}

fn dropdown_text(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
    label_is_placeholder: bool,
) -> [u8; 4] {
    if is_unavailable_dropdown_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.value_color) {
        color
    } else if label_is_placeholder {
        WORKBENCH_DROPDOWN_PLACEHOLDER
    } else {
        WORKBENCH_DROPDOWN_TEXT
    }
}

fn dropdown_chevron(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_dropdown_state(state) {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.icon_color) {
        color
    } else if matches!(
        state,
        UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
    ) {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}

fn declared_color(color: Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn is_unavailable_dropdown_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn declared_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}

fn apply_visual_brightness(
    style: WorkbenchDropdownStyle,
    brightness: f32,
) -> WorkbenchDropdownStyle {
    if is_unavailable_dropdown_state(style.state) {
        return style;
    }
    if !brightness.is_finite() || brightness <= 0.0 || (brightness - 1.0).abs() < 0.001 {
        return style;
    }
    let brightness = brightness.clamp(0.0, 4.0);
    WorkbenchDropdownStyle {
        surface: scaled_color(style.surface, brightness),
        border: scaled_color(style.border, brightness),
        text: scaled_color(style.text, brightness),
        chevron: scaled_color(style.chevron, brightness),
        state: style.state,
    }
}

fn scaled_color(color: [u8; 4], brightness: f32) -> [u8; 4] {
    [
        scaled_channel(color[0], brightness),
        scaled_channel(color[1], brightness),
        scaled_channel(color[2], brightness),
        color[3],
    ]
}

fn scaled_channel(value: u8, brightness: f32) -> u8 {
    (f32::from(value) * brightness).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn dropdown_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.popup_open = true;
        node.selected = true;
        node.validation_level = "danger".into();
        node.button_style.loading = true;
        node.label_brightness = 1.8;
        node.value_color = Color::from_rgb_u8(205, 216, 221);
        node.icon_color = Color::from_rgb_u8(128, 234, 255);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(15, 101, 116, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

        let style = select_workbench_dropdown_style(&node, false);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, WORKBENCH_DROPDOWN_DISABLED_SURFACE);
        assert_eq!(style.border, PALETTE.border_disabled);
        assert_eq!(style.text, PALETTE.text_disabled);
        assert_eq!(style.chevron, PALETTE.text_disabled);
    }
}
