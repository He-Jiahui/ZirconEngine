use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::resolved_style_color;
use super::super::theme::PALETTE;
use super::painter_state_for_node;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

const ICON_NORMAL: [u8; 4] = [164, 174, 180, 255];
const ICON_MUTED: [u8; 4] = [132, 146, 154, 255];
const ICON_PANEL_SURFACE: [u8; 4] = [31, 37, 41, 255];
const ICON_PANEL_BORDER: [u8; 4] = [48, 57, 63, 255];

pub(in crate::ui::retained_host::host_contract::painter) const WORKBENCH_ICON_PANEL_RADIUS: f32 =
    6.0;
const WORKBENCH_ICON_RAIL_RADIUS: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::painter) enum WorkbenchIconButtonContext {
    Toolbar,
    Rail,
    Panel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract::painter) struct WorkbenchIconButtonStyle {
    pub background: Option<[u8; 4]>,
    pub border: Option<[u8; 4]>,
    pub border_width: f32,
    pub radius: f32,
    pub glyph: [u8; 4],
    pub state: UiPainterResolvedState,
}

pub(in crate::ui::retained_host::host_contract::painter) fn select_workbench_icon_button_style(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
) -> WorkbenchIconButtonStyle {
    let state = painter_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);
    let danger = is_danger_icon(node);

    WorkbenchIconButtonStyle {
        background: icon_background(node, context, state, danger),
        border: icon_border(node, context, state, danger),
        border_width: icon_border_width(context, state),
        radius: icon_radius(node, context),
        glyph: icon_glyph_color(node, context, state, danger),
        state,
    }
}

fn icon_background(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_icon_button_state(state) {
        return (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.surface_disabled);
    }
    if danger && context == WorkbenchIconButtonContext::Panel {
        return Some(PALETTE.error_container);
    }
    match state {
        UiPainterResolvedState::Selected | UiPainterResolvedState::Checked => {
            Some(PALETTE.surface_selected)
        }
        UiPainterResolvedState::Pressed => Some(PALETTE.surface_pressed),
        UiPainterResolvedState::Focused
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => Some(PALETTE.surface_hover),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.surface_disabled)
        }
        UiPainterResolvedState::Normal => {
            if context == WorkbenchIconButtonContext::Panel {
                declared_icon_button_background(node).or(Some(ICON_PANEL_SURFACE))
            } else {
                None
            }
        }
    }
}

fn icon_border(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> Option<[u8; 4]> {
    if is_unavailable_icon_button_state(state) {
        return (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.border_disabled);
    }
    if danger && context == WorkbenchIconButtonContext::Panel {
        return declared_icon_button_border(node).or(Some(PALETTE.error));
    }
    match state {
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered => Some(PALETTE.focus_ring),
        UiPainterResolvedState::Hovered => Some(PALETTE.border),
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
            (context == WorkbenchIconButtonContext::Panel).then_some(PALETTE.border_disabled)
        }
        UiPainterResolvedState::Normal => {
            if context == WorkbenchIconButtonContext::Panel {
                declared_icon_button_border(node).or(Some(ICON_PANEL_BORDER))
            } else {
                None
            }
        }
    }
}

fn icon_border_width(context: WorkbenchIconButtonContext, state: UiPainterResolvedState) -> f32 {
    if context == WorkbenchIconButtonContext::Panel || state != UiPainterResolvedState::Normal {
        1.0
    } else {
        0.0
    }
}

fn icon_radius(node: &TemplatePaneNodeData, context: WorkbenchIconButtonContext) -> f32 {
    let declared = node.button_style.element.corner_radius;
    if declared.is_finite() && declared > 0.0 {
        return declared;
    }
    if context == WorkbenchIconButtonContext::Panel
        && node.corner_radius.is_finite()
        && node.corner_radius > WORKBENCH_ICON_PANEL_RADIUS
    {
        return node.corner_radius;
    }
    match context {
        WorkbenchIconButtonContext::Rail => WORKBENCH_ICON_RAIL_RADIUS,
        WorkbenchIconButtonContext::Toolbar | WorkbenchIconButtonContext::Panel => {
            WORKBENCH_ICON_PANEL_RADIUS
        }
    }
}

fn icon_glyph_color(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
    state: UiPainterResolvedState,
    danger: bool,
) -> [u8; 4] {
    if is_unavailable_icon_button_state(state) {
        PALETTE.text_disabled
    } else if danger {
        PALETTE.error
    } else {
        match state {
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => PALETTE.focus_ring,
            UiPainterResolvedState::Hovered => PALETTE.text,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                PALETTE.text_disabled
            }
            UiPainterResolvedState::Normal => declared_icon_color(node).unwrap_or_else(|| {
                if context == WorkbenchIconButtonContext::Rail {
                    ICON_MUTED
                } else {
                    ICON_NORMAL
                }
            }),
        }
    }
}

fn is_unavailable_icon_button_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

fn declared_icon_button_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

fn declared_icon_button_border(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
}

fn declared_icon_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    (node.icon_color.a > 0).then_some([
        node.icon_color.r,
        node.icon_color.g,
        node.icon_color.b,
        node.icon_color.a,
    ])
}

fn is_danger_icon(node: &TemplatePaneNodeData) -> bool {
    let key = format!(
        "{} {} {}",
        node.control_id.as_str(),
        node.icon_name.as_str(),
        node.validation_level.as_str()
    )
    .to_ascii_lowercase();
    key.contains("delete")
        || key.contains("trash")
        || key.contains("danger")
        || key.contains("error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::Color;
    use zircon_runtime_interface::ui::style::{UiRgbaColor, UiStyleColor};

    #[test]
    fn icon_button_loading_state_uses_unavailable_visuals() {
        let mut node = TemplatePaneNodeData::default();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.checked = true;
        node.selected = true;
        node.button_style.loading = true;
        node.control_id = "WorkbenchDeleteIconButton".into();
        node.icon_name = "trash".into();
        node.validation_level = "danger".into();
        node.icon_color = Color::from_rgb_u8(239, 112, 102);
        node.button_style.element.background_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(63, 25, 28, 255)));
        node.button_style.element.border_color =
            Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(239, 112, 102, 255)));

        let panel = select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Panel);

        assert_eq!(panel.state, UiPainterResolvedState::Loading);
        assert_eq!(panel.background, Some(PALETTE.surface_disabled));
        assert_eq!(panel.border, Some(PALETTE.border_disabled));
        assert_eq!(panel.border_width, 1.0);
        assert_eq!(panel.glyph, PALETTE.text_disabled);

        let toolbar =
            select_workbench_icon_button_style(&node, WorkbenchIconButtonContext::Toolbar);

        assert_eq!(toolbar.state, UiPainterResolvedState::Loading);
        assert_eq!(toolbar.background, None);
        assert_eq!(toolbar.border, None);
        assert_eq!(toolbar.border_width, 1.0);
        assert_eq!(toolbar.glyph, PALETTE.text_disabled);
    }
}
