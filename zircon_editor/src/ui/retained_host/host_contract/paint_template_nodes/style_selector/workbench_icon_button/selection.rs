mod background;
mod border;
mod danger;
mod declared;
mod glyph;
mod radius;

use super::super::resolved_state_for_node;
use super::super::workbench_command::{workbench_command_visual_role, WorkbenchCommandVisualRole};
use super::model::{WorkbenchIconButtonContext, WorkbenchIconButtonStyle};
use super::palette::workbench_icon_button_palette;
use super::state::{icon_button_node_is_hot, icon_button_node_is_selected};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::current_host_metrics;
use background::icon_background;
use border::{icon_border, icon_border_width};
use danger::is_danger_icon;
use glyph::icon_glyph_color;
use radius::icon_radius;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

#[cfg(test)]
pub(super) use border::icon_border_width_from_host;
#[cfg(test)]
pub(super) use radius::icon_radius_from_host;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_icon_button_style(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
) -> WorkbenchIconButtonStyle {
    let state =
        resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);
    let danger = is_danger_icon(node);

    let style = WorkbenchIconButtonStyle {
        background: icon_background(node, context, state, danger),
        border: icon_border(node, context, state, danger),
        border_width: icon_border_width(node, context, state),
        radius: icon_radius(node, context),
        glyph: icon_glyph_color(node, context, state, danger),
        state,
    };

    match workbench_command_visual_role(node) {
        WorkbenchCommandVisualRole::PrimaryImport => primary_import_icon_button_style(node, style),
        WorkbenchCommandVisualRole::None | WorkbenchCommandVisualRole::MutedProminent => style,
    }
}

fn primary_import_icon_button_style(
    node: &TemplatePaneNodeData,
    mut style: WorkbenchIconButtonStyle,
) -> WorkbenchIconButtonStyle {
    let palette = workbench_icon_button_palette();
    let focus_only = matches!(style.state, UiPainterResolvedState::Focused)
        && !icon_button_node_is_selected(node)
        && !icon_button_node_is_hot(node);
    let surface = match style.state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => return style,
        UiPainterResolvedState::Normal => palette.accent,
        UiPainterResolvedState::Focused if focus_only => palette.accent,
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => palette.focus_ring,
    };
    style.background = Some(surface);
    style.border = Some(if focus_only {
        palette.focus_ring
    } else {
        surface
    });
    style.border_width = current_host_metrics().border_width;
    style.glyph = palette.shell_background;
    style
}
