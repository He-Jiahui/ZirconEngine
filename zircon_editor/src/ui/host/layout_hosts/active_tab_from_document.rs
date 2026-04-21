use crate::ui::workbench::layout::DocumentNode;
use crate::ui::workbench::view::ViewInstanceId;

pub(in crate::ui::host) fn active_tab_from_document(node: &DocumentNode) -> Option<ViewInstanceId> {
    match node {
        DocumentNode::Tabs(stack) => stack.active_tab.clone(),
        DocumentNode::SplitNode { first, second, .. } => {
            active_tab_from_document(first).or_else(|| active_tab_from_document(second))
        }
    }
}
