use crate::ui::template::{UiTemplateInstance, UiTemplateTreeBuilder};
use crate::ui::tree::UiRuntimeTreeAccessExt;
use zircon_runtime_interface::ui::{event_ui::UiTreeId, template::UiTemplateNode};

#[test]
fn template_tree_builder_inserts_deep_template_chain_with_explicit_stack() {
    const NODE_COUNT: usize = 4096;
    let instance = UiTemplateInstance {
        root: deep_template_chain(NODE_COUNT),
    };

    let result = UiTemplateTreeBuilder::build_tree(UiTreeId::new("deep.template.chain"), &instance);
    // Recursive drop of the transitional template tree is not part of this stack-safety signal.
    std::mem::forget(instance);
    let tree = result.unwrap();

    assert_eq!(tree.roots.len(), 1);
    assert_eq!(tree.nodes.len(), NODE_COUNT);

    let mut depth = 1usize;
    let mut node = tree.node(tree.roots[0]).unwrap();
    while let Some(child_id) = node.children.first() {
        assert_eq!(node.children.len(), 1);
        depth += 1;
        node = tree.node(*child_id).unwrap();
    }

    assert_eq!(depth, NODE_COUNT);
    assert_eq!(
        node.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.control_id.as_deref()),
        Some("DeepLeaf")
    );
}

fn deep_template_chain(node_count: usize) -> UiTemplateNode {
    assert!(node_count > 0);
    let mut node = UiTemplateNode {
        component: Some("P".to_string()),
        control_id: Some("DeepLeaf".to_string()),
        ..Default::default()
    };

    for _ in 1..node_count {
        node = UiTemplateNode {
            component: Some("P".to_string()),
            children: vec![node],
            ..Default::default()
        };
    }

    node
}
