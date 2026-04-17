use std::collections::BTreeMap;

use crate::layout::DocumentNode;
use crate::view::{ViewHost, ViewInstanceId};

pub(super) fn collect_document_hosts(
    node: &DocumentNode,
    placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
    make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
) {
    fn visit(
        node: &DocumentNode,
        path: &mut Vec<usize>,
        placements: &mut BTreeMap<ViewInstanceId, ViewHost>,
        make_host: impl Fn(Vec<usize>) -> ViewHost + Copy,
    ) {
        match node {
            DocumentNode::Tabs(stack) => {
                let host = make_host(path.clone());
                for instance_id in &stack.tabs {
                    placements.insert(instance_id.clone(), host.clone());
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                path.push(0);
                visit(first, path, placements, make_host);
                path.pop();
                path.push(1);
                visit(second, path, placements, make_host);
                path.pop();
            }
        }
    }

    let mut path = Vec::new();
    visit(node, &mut path, placements, make_host);
}
