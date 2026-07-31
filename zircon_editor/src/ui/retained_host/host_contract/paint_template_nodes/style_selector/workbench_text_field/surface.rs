use super::colors::declared_style_color;
use super::palette::workbench_text_field_palette;
use super::state::is_unavailable_text_field_state;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_surface(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    let has_validation_error = matches!(node.validation_level.as_str(), "error" | "danger");
    let color = if is_toolbar_text_field(node) {
        match state {
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.disabled_surface
            }
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open
            | UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
            | UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => palette.toolbar_surface,
        }
    } else {
        match state {
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.disabled_surface
            }
            UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Focused
            | UiPainterResolvedState::Open => palette.focused_surface,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => palette.hover_surface,
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => palette.surface,
        }
    };
    if is_unavailable_text_field_state(state)
        || is_toolbar_text_field(node)
        || has_validation_error
        || !accepts_normal_text_field_override(state)
    {
        color
    } else {
        declared_style_color(node.button_style.element.background_color.as_ref()).unwrap_or(color)
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn text_field_border(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> [u8; 4] {
    let palette = workbench_text_field_palette();
    let has_validation_error = matches!(node.validation_level.as_str(), "error" | "danger");
    let color = if is_unavailable_text_field_state(state) {
        palette.disabled_border
    } else if has_validation_error {
        palette.error
    } else {
        match state {
            UiPainterResolvedState::Pressed => palette.focus_border,
            UiPainterResolvedState::Focused | UiPainterResolvedState::Open => palette.focus_border,
            UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered => palette.hover_border,
            UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => {
                palette.disabled_border
            }
            UiPainterResolvedState::Checked
            | UiPainterResolvedState::Selected
            | UiPainterResolvedState::Normal => palette.border,
        }
    };
    if is_unavailable_text_field_state(state)
        || is_toolbar_text_field(node)
        || has_validation_error
        || !accepts_normal_text_field_override(state)
    {
        color
    } else {
        declared_style_color(node.button_style.element.border_color.as_ref()).unwrap_or(color)
    }
}

fn is_toolbar_text_field(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "SearchEdited" | "AssetBrowserImportPathField"
    ) || matches!(node.role.as_str(), "SearchField")
        || matches!(node.component_role.as_str(), "search-field")
}

fn accepts_normal_text_field_override(state: UiPainterResolvedState) -> bool {
    state == UiPainterResolvedState::Normal
}
