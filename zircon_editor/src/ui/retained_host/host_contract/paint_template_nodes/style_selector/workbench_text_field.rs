use super::super::super::data::TemplatePaneNodeData;
use super::super::super::paint_theme::PALETTE;
use super::super::template_style_color::resolved_style_color;
use super::resolved_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_SURFACE: [u8;
    4] = [16, 22, 26, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_HOVER_SURFACE:
    [u8; 4] = [20, 27, 31, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE:
    [u8; 4] = [15, 24, 28, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_SURFACE:
    [u8; 4] = [36, 41, 45, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_BORDER: [u8;
    4] = [50, 63, 71, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_FOCUSED_BORDER:
    [u8; 4] = [27, 152, 160, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_BORDER:
    [u8; 4] = [48, 56, 62, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_TEXT: [u8; 4] =
    [205, 216, 221, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_PLACEHOLDER:
    [u8; 4] = [122, 134, 142, 255];
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) const WORKBENCH_TEXT_FIELD_DISABLED_TEXT:
    [u8; 4] = [125, 135, 141, 255];

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct WorkbenchTextFieldStyle
{
    pub surface: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub stepper: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_text_field_style(
    node: &TemplatePaneNodeData,
    label_is_placeholder: bool,
) -> WorkbenchTextFieldStyle {
    let mut painter_state = resolved_state_for_node(node);
    if node.control_id.as_str() == "WorkbenchInputFocused" {
        painter_state.focused = true;
    }
    let state = painter_state.resolved_state_for_family(UiPainterFamily::TextField);
    WorkbenchTextFieldStyle {
        surface: text_field_surface(node, state),
        border: text_field_border(node, state),
        text: text_field_text(state, label_is_placeholder),
        stepper: text_field_stepper(state),
        state,
    }
}

fn text_field_surface(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    let color = match state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            WORKBENCH_TEXT_FIELD_DISABLED_SURFACE
        }
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open => WORKBENCH_TEXT_FIELD_FOCUSED_SURFACE,
        UiPainterResolvedState::Hovered
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => WORKBENCH_TEXT_FIELD_HOVER_SURFACE,
        UiPainterResolvedState::Checked
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Normal => WORKBENCH_TEXT_FIELD_SURFACE,
    };
    if is_unavailable_text_field_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}

fn text_field_border(node: &TemplatePaneNodeData, state: UiPainterResolvedState) -> [u8; 4] {
    let color = if is_unavailable_text_field_state(state) {
        WORKBENCH_TEXT_FIELD_DISABLED_BORDER
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed => PALETTE.focus_ring,
            UiPainterResolvedState::Focused | UiPainterResolvedState::Open => {
                WORKBENCH_TEXT_FIELD_FOCUSED_BORDER
            }
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                WORKBENCH_TEXT_FIELD_DISABLED_BORDER
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => WORKBENCH_TEXT_FIELD_BORDER,
        }
    };
    if is_unavailable_text_field_state(state) {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}

fn text_field_text(state: UiPainterResolvedState, label_is_placeholder: bool) -> [u8; 4] {
    if is_unavailable_text_field_state(state) {
        WORKBENCH_TEXT_FIELD_DISABLED_TEXT
    } else if label_is_placeholder {
        WORKBENCH_TEXT_FIELD_PLACEHOLDER
    } else {
        WORKBENCH_TEXT_FIELD_TEXT
    }
}

fn text_field_stepper(state: UiPainterResolvedState) -> [u8; 4] {
    if is_unavailable_text_field_state(state) {
        PALETTE.text_disabled
    } else {
        PALETTE.text_muted
    }
}

fn is_unavailable_text_field_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn declared_style_color(
    color: Option<&zircon_runtime_interface::ui::style::UiStyleColor>,
) -> Option<[u8; 4]> {
    resolved_style_color(color).filter(|color| color[3] > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn text_field_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.validation_level = "error".into();
        node.button_style.loading = true;
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(16, 22, 26, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

        let style = select_workbench_text_field_style(&node, true);

        assert_eq!(style.state, UiPainterResolvedState::Loading);
        assert_eq!(style.surface, WORKBENCH_TEXT_FIELD_DISABLED_SURFACE);
        assert_eq!(style.border, WORKBENCH_TEXT_FIELD_DISABLED_BORDER);
        assert_eq!(style.text, WORKBENCH_TEXT_FIELD_DISABLED_TEXT);
        assert_eq!(style.stepper, PALETTE.text_disabled);
    }
}
