use crate::layout::DocumentNode;
use crate::view::ViewInstanceId;

pub(in crate::core::host::manager) fn active_tab_from_document(
    node: &DocumentNode,
) -> Option<ViewInstanceId> {
    match node {
        DocumentNode::Tabs(stack) => stack.active_tab.clone(),
        DocumentNode::SplitNode { first, second, .. } => {
            active_tab_from_document(first).or_else(|| active_tab_from_document(second))
        }
    }
}
