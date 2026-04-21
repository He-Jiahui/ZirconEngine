use crate::ui::workbench::layout::{DocumentNode, TabStackLayout};

use super::first_tab_stack::first_tab_stack;

pub(super) fn first_tab_stack_mut(node: &mut DocumentNode) -> &mut TabStackLayout {
    match node {
        DocumentNode::Tabs(stack) => stack,
        DocumentNode::SplitNode { first, second, .. } => {
            if let Some(stack) = first_tab_stack(first) {
                let _ = stack;
                first_tab_stack_mut(first)
            } else {
                first_tab_stack_mut(second)
            }
        }
    }
}
