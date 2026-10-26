use std::collections::BTreeSet;

use zircon_runtime_interface::ui::v2::UiV2AssetDocument;

use super::UiAssetSourceOwner;

pub(super) fn source_owners(
    document: &UiV2AssetDocument,
    node_id: &str,
) -> Vec<UiAssetSourceOwner> {
    let mut owners = Vec::new();
    if let Some(root) = document.root.as_ref() {
        if contains_node(document, &root.node, node_id) {
            owners.push(UiAssetSourceOwner::AssetRoot {
                root_node_id: root.node.clone(),
            });
        }
    }
    for (name, component) in &document.components {
        if contains_node(document, &component.root, node_id) {
            owners.push(UiAssetSourceOwner::Component {
                name: name.clone(),
                root_node_id: component.root.clone(),
            });
        }
    }
    owners
}

fn contains_node(document: &UiV2AssetDocument, root: &str, needle: &str) -> bool {
    let mut pending = vec![root];
    let mut seen = BTreeSet::new();
    while let Some(node_id) = pending.pop() {
        if !seen.insert(node_id) {
            continue;
        }
        let Some(node) = document.nodes.get(node_id) else {
            continue;
        };
        if node_id == needle {
            return true;
        }
        // Component invocations do not transfer declaration ownership to their callers.
        pending.extend(node.children.iter().map(|child| child.node.as_str()));
    }
    false
}
