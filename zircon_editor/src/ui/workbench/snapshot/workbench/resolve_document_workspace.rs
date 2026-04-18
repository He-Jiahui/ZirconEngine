use std::collections::HashMap;

use crate::layout::DocumentNode;
use crate::view::{ViewDescriptor, ViewDescriptorId, ViewInstance, ViewInstanceId};

use super::{resolve_view_tab, DocumentWorkspaceSnapshot};

pub(crate) fn resolve_document_workspace(
    node: &DocumentNode,
    instances: &HashMap<ViewInstanceId, ViewInstance>,
    descriptors: &HashMap<ViewDescriptorId, ViewDescriptor>,
) -> DocumentWorkspaceSnapshot {
    match node {
        DocumentNode::SplitNode {
            axis,
            ratio,
            first,
            second,
        } => DocumentWorkspaceSnapshot::Split {
            axis: *axis,
            ratio: *ratio,
            first: Box::new(resolve_document_workspace(first, instances, descriptors)),
            second: Box::new(resolve_document_workspace(second, instances, descriptors)),
        },
        DocumentNode::Tabs(stack) => DocumentWorkspaceSnapshot::Tabs {
            tabs: stack
                .tabs
                .iter()
                .map(|instance_id| resolve_view_tab(instance_id, instances, descriptors))
                .collect(),
            active_tab: stack.active_tab.clone(),
        },
    }
}
