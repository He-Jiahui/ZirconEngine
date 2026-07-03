mod background;
mod border;
mod danger;
mod declared;
mod glyph;
mod radius;

use super::super::resolved_state_for_node;
use super::super::workbench_command::{workbench_command_visual_role, WorkbenchCommandVisualRole};
use super::model::{WorkbenchIconButtonContext, WorkbenchIconButtonStyle};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use background::icon_background;
use border::{icon_border, icon_border_width};
use danger::is_danger_icon;
use glyph::icon_glyph_color;
use radius::icon_radius;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

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
        border_width: icon_border_width(context, state),
        radius: icon_radius(node, context),
        glyph: icon_glyph_color(node, context, state, danger),
        state,
    };

    match workbench_command_visual_role(node) {
        WorkbenchCommandVisualRole::PrimaryImport => primary_import_icon_button_style(style),
        WorkbenchCommandVisualRole::None | WorkbenchCommandVisualRole::MutedProminent => style,
    }
}

fn primary_import_icon_button_style(
    mut style: WorkbenchIconButtonStyle,
) -> WorkbenchIconButtonStyle {
    let surface = match style.state {
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading => return style,
        UiPainterResolvedState::Normal => PALETTE.accent,
        UiPainterResolvedState::Pressed
        | UiPainterResolvedState::Focused
        | UiPainterResolvedState::Selected
        | UiPainterResolvedState::Checked
        | UiPainterResolvedState::Open
        | UiPainterResolvedState::Dragging
        | UiPainterResolvedState::DropHovered
        | UiPainterResolvedState::Hovered => PALETTE.focus_ring,
    };
    style.background = Some(surface);
    style.border = Some(surface);
    style.border_width = 1.0;
    style.glyph = PALETTE.shell_background;
    style
}
