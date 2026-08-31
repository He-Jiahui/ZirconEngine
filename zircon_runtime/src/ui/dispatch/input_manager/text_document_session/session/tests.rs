use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    tree::UiTreeNode,
};

use crate::ui::surface::UiSurface;

use super::UiTextDocumentSession;

#[test]
fn detached_owner_closes_its_retained_document() {
    let tree_id = UiTreeId::new("text-document-session.detach");
    let node_id = UiNodeId::new(7);
    let mut surface = UiSurface::new(tree_id.clone());
    surface.tree.nodes.insert(
        node_id,
        UiTreeNode::new(node_id, UiNodePath::new("root/editor")),
    );
    let mut session = UiTextDocumentSession::default();
    session.synchronize_owners(&surface.tree, surface.session_identity());
    session.synchronize_source(&tree_id, node_id, 0, "retained");
    assert_eq!(session.store.report().document_count, 1);

    surface.tree.nodes.remove(&node_id);
    session.synchronize_owners(&surface.tree, surface.session_identity());

    assert!(session.bindings.is_empty());
    assert_eq!(session.store.report().document_count, 0);
}

#[test]
fn retained_grapheme_query_requires_the_synchronized_source_epoch() {
    let tree_id = UiTreeId::new("text-document-session.graphemes");
    let node_id = UiNodeId::new(8);
    let mut session = UiTextDocumentSession::default();
    session.synchronize_source(&tree_id, node_id, 4, "a\u{0301}bc");

    assert_eq!(
        session
            .retained_grapheme_count(&tree_id, node_id, 4, 3..4)
            .expect("the synchronized revision owns the grapheme index"),
        2
    );
    assert_eq!(
        session.retained_grapheme_count(&tree_id, node_id, 3, 3..4),
        Err(super::super::UiTextDocumentSessionError::SourceNotSynchronized)
    );
}
