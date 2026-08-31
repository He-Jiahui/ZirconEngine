use super::{GraphNodeView, GraphSelection};

/// Renderer-neutral node projection. The retained UI layer owns widgets; this type ensures all
/// toolkits consume a consistent title, pin count, attachment list, and selection state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodePresentation<NodeId> {
    pub node_id: NodeId,
    pub title: String,
    pub input_count: usize,
    pub output_count: usize,
    pub attachment_labels: Vec<String>,
    pub selected: bool,
}

impl<NodeId> GraphNodePresentation<NodeId>
where
    NodeId: Clone + Ord,
{
    pub fn from_view(node: &GraphNodeView<NodeId>, selection: &GraphSelection<NodeId>) -> Self {
        Self {
            node_id: node.id.clone(),
            title: node.display_name.clone(),
            input_count: node.inputs.len(),
            output_count: node.outputs.len(),
            attachment_labels: node
                .attachments
                .iter()
                .map(|attachment| attachment.display_name.clone())
                .collect(),
            selected: selection.contains(&node.id),
        }
    }
}
