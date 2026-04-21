use crate::ui::workbench::layout::{DocumentNode, TabStackLayout};

pub(super) fn first_tab_stack(node: &DocumentNode) -> Option<&TabStackLayout> {
    match node {
        DocumentNode::Tabs(stack) => Some(stack),
        DocumentNode::SplitNode { first, second, .. } => {
            first_tab_stack(first).or_else(|| first_tab_stack(second))
        }
    }
}
