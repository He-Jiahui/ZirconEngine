use std::rc::Rc;

use crate::ui::retained_host::primitives::{ModelRc, VecModel};

use super::super::super::data::{HostPaneInteractionStateData, TemplatePaneNodeData};
use super::rows::apply_template_row_hover;

pub(super) fn apply_template_hover_to_nodes(
    nodes: &mut ModelRc<TemplatePaneNodeData>,
    interaction: &HostPaneInteractionStateData,
) -> bool {
    let mut changed = false;
    let hovered = &interaction.hovered_template_control_id;
    let values: Vec<_> = (0..nodes.row_count())
        .filter_map(|row| nodes.row_data(row))
        .map(|mut node| {
            if node.control_id.as_str() == hovered.as_str() && !node.hovered {
                node.hovered = true;
                changed = true;
            }
            if node.control_id.as_str() == hovered.as_str() {
                changed |= apply_template_row_hover(&mut node, interaction);
            }
            node
        })
        .collect();
    if changed {
        *nodes = ModelRc::from(Rc::new(VecModel::from(values)));
    }
    changed
}
