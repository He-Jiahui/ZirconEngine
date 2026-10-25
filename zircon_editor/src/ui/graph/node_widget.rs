use super::{GraphNodeView, GraphSelection};

/// Domain identity for an attachment row. Geometry, instance paths, selection and focus must come
/// from the actual retained consumer; equal labels never identify an attachment or its host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphAttachmentPresentation<NodeId> {
    pub attachment_id: String,
    pub host_node_id: NodeId,
    pub display_name: String,
    pub category: String,
}

impl<NodeId: Clone> GraphAttachmentPresentation<NodeId> {
    pub fn from_view(node: &GraphNodeView<NodeId>) -> Vec<Self> {
        node.attachments
            .iter()
            .map(|attachment| Self {
                attachment_id: attachment.id.clone(),
                host_node_id: node.id.clone(),
                display_name: attachment.display_name.clone(),
                category: attachment.category.clone(),
            })
            .collect()
    }
}

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
