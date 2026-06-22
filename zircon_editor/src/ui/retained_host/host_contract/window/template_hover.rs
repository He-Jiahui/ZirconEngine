mod nodes;
mod panes;
mod rows;

use self::nodes::apply_template_hover_to_nodes;
use self::panes::{apply_template_hover_to_dock_panes, apply_template_hover_to_floating_panes};
use super::super::data::{HostPaneInteractionStateData, HostWindowPresentationData};

pub(in crate::ui::retained_host::host_contract) fn apply_template_hover_to_presentation(
    presentation: &mut HostWindowPresentationData,
    interaction: &HostPaneInteractionStateData,
) {
    if interaction.hovered_template_control_id.is_empty() {
        return;
    }
    apply_template_hover_to_nodes(&mut presentation.workbench_window_nodes, interaction);
    apply_template_hover_to_dock_panes(presentation, interaction);
    apply_template_hover_to_floating_panes(presentation, interaction);
}
