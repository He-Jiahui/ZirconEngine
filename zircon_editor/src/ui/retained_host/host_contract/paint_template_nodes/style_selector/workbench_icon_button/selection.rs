mod background;
mod border;
mod danger;
mod declared;
mod glyph;
mod radius;

use super::super::resolved_state_for_node;
use super::model::{WorkbenchIconButtonContext, WorkbenchIconButtonStyle};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use background::icon_background;
use border::{icon_border, icon_border_width};
use danger::is_danger_icon;
use glyph::icon_glyph_color;
use radius::icon_radius;
use zircon_runtime_interface::ui::style::UiPainterFamily;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn select_workbench_icon_button_style(
    node: &TemplatePaneNodeData,
    context: WorkbenchIconButtonContext,
) -> WorkbenchIconButtonStyle {
    let state =
        resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::IconButton);
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
