use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_node_deletion,
    validate_material_graph_with_diagnostics, MaterialGraphDiagnosticTarget,
    MaterialGraphPresentation,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Value,
};

#[test]
fn material_graph_node_deletion_empty_and_root_only_selections_are_no_ops() {
    let mut graph = graph();
    graph.nodes = vec![node("output", Node::Output), scalar("value", "Gain", 0.2)];
    graph.links.push(link("value", "output"));
    for selection in [vec![], vec!["output"]] {
        let plan = plan_material_graph_node_deletion(&graph, &selection).unwrap();
        assert!(!plan.can_delete());
        assert!(plan.removed_nodes().is_empty());
        assert!(plan.removed_links().is_empty());
        assert_eq!(plan.protected_nodes().len(), selection.len());
        assert!(std::ptr::eq(plan.source(), &graph));
    }
    let plan = plan_material_graph_node_deletion(&graph, &["output"]).unwrap();
    assert!(std::ptr::eq(plan.protected_nodes()[0], &graph.nodes[0]));
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.links.len(), 1);
}

#[test]
fn material_graph_node_deletion_mixed_selection_keeps_root_and_original_source_order() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("first", "Shared", 0.1),
        node("output", Node::Output),
        scalar("kept", "Shared", 0.3),
        node("last", Node::Multiply),
    ];
    graph.links = vec![
        link("last", "output"),
        link("kept", "output"),
        link("first", "last"),
    ];
    graph.parameters.insert("Shared".into(), Value::Scalar(0.9));
    let before = graph.clone();
    let plan = plan_material_graph_node_deletion(&graph, &["last", "output", "first"]).unwrap();
    assert!(plan.can_delete());
    assert_eq!(
        plan.removed_nodes()
            .iter()
            .map(|(index, node)| (*index, node.id.as_str()))
            .collect::<Vec<_>>(),
        [(0, "first"), (3, "last")]
    );
    assert_eq!(
        plan.removed_links()
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>(),
        [0, 2]
    );
    assert!(std::ptr::eq(plan.protected_nodes()[0], &graph.nodes[1]));
    assert!(std::ptr::eq(plan.removed_nodes()[0].1, &graph.nodes[0]));
    assert!(std::ptr::eq(plan.removed_links()[1].1, &graph.links[2]));
    assert_eq!(graph, before);
}

#[test]
fn material_graph_node_deletion_rejects_stale_duplicate_and_empty_selection_atomically() {
    let mut graph = graph();
    graph.nodes = vec![node("output", Node::Output), node("math", Node::Add)];
    let before = graph.clone();
    for selection in [
        vec!["math", "missing"],
        vec!["math", "math"],
        vec![""],
        vec![" \t"],
    ] {
        let errors = plan_material_graph_node_deletion(&graph, &selection).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert_eq!(graph, before);
    }
}

#[test]
fn material_graph_node_deletion_rejects_ambiguous_source_ids_even_outside_selection() {
    let mut graph = graph();
    graph.nodes = vec![
        node("math", Node::Add),
        node("other", Node::Multiply),
        node("other", Node::Add),
    ];
    let before = graph.clone();
    assert!(plan_material_graph_node_deletion(&graph, &["math"]).is_err());
    assert_eq!(graph, before);
    graph.nodes[2].id = " \t".into();
    assert!(plan_material_graph_node_deletion(&graph, &["math"]).is_err());
}

#[test]
fn material_graph_node_deletion_uses_exact_case_whitespace_and_cjk_node_keys() {
    let mut graph = graph();
    graph.nodes = vec![
        node("Key", Node::Add),
        node("key", Node::Multiply),
        node(" key ", Node::Add),
        scalar("节点_材质参数_很长的稳定标识符", "显示名不作节点身份", 0.3),
    ];
    let plan =
        plan_material_graph_node_deletion(&graph, &[" key ", "节点_材质参数_很长的稳定标识符"])
            .unwrap();
    assert_eq!(
        plan.removed_nodes()
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert!(plan_material_graph_node_deletion(&graph, &["KEY"]).is_err());
    assert!(plan_material_graph_node_deletion(&graph, &["显示名不作节点身份"]).is_err());
}

#[test]
fn material_graph_node_deletion_keeps_all_incident_link_occurrences_without_double_removal() {
    let mut graph = graph();
    graph.nodes = vec![
        node("a", Node::Add),
        node("b", Node::Multiply),
        node("kept", Node::Output),
    ];
    graph.links = vec![
        link("missing", "kept"),
        link("a", "b"),
        link("a", "b"),
        link("a", "a"),
        link("b", "kept"),
        link("dangling", "a"),
        link("kept", "unrelated-missing"),
    ];
    let before = graph.clone();
    let plan = plan_material_graph_node_deletion(&graph, &["a", "b"]).unwrap();
    assert_eq!(
        plan.removed_links()
            .iter()
            .map(|(index, _)| *index)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    for (index, edge) in plan.removed_links() {
        assert!(std::ptr::eq(*edge, &graph.links[*index]));
    }
    assert_ne!(
        plan.removed_links()[0].1 as *const _,
        plan.removed_links()[1].1 as *const _
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_node_deletion_allows_repair_without_erasing_unrelated_diagnostics() {
    let mut graph = graph();
    graph.nodes = vec![
        node("a", Node::Add),
        node("b", Node::Multiply),
        scalar("bad-default", "Invalid", f32::NAN),
    ];
    graph.links = vec![link("a", "b"), link("b", "a")];
    graph
        .parameters
        .insert("Invalid".into(), Value::Scalar(f32::INFINITY));
    let before = graph.to_toml_string().unwrap();
    let errors = validate_material_graph_with_diagnostics(&graph);
    assert!(!errors.is_empty());
    let plan = plan_material_graph_node_deletion(&graph, &["a"]).unwrap();
    assert_eq!(plan.removed_nodes().len(), 1);
    assert_eq!(plan.removed_links().len(), 2);
    assert_eq!(validate_material_graph_with_diagnostics(&graph), errors);
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_node_deletion_preserves_shared_and_orphan_override_presence() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("removed", "Shared", 0.2),
        scalar("kept", "Shared", 0.7),
    ];
    graph
        .parameters
        .insert("Shared".into(), Value::Scalar(-0.0));
    graph
        .parameters
        .insert("Orphan".into(), Value::Vector([0.0; 4]));
    let before = graph.clone();
    let plan = plan_material_graph_node_deletion(&graph, &["removed"]).unwrap();
    assert!(plan.can_delete());
    assert_eq!(plan.source().parameters, before.parameters);
    let mut after = graph.clone();
    after.nodes.remove(plan.removed_nodes()[0].0);
    let reopened = MaterialGraphAsset::from_toml_str(&after.to_toml_string().unwrap()).unwrap();
    let row = MaterialGraphPresentation::new(&reopened)
        .unwrap()
        .parameter_declarations()
        .next()
        .unwrap();
    assert_eq!(row.node().id, "kept");
    let Some(Value::Scalar(value)) = row.override_value() else {
        unreachable!()
    };
    assert!(value.is_sign_negative());
    assert_eq!(
        reopened.parameters.get("Orphan"),
        Some(&Value::Vector([0.0; 4]))
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_node_deletion_codec_and_inverse_data_restore_real_pbr_compile() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        scalar("unconnected", "Unused", 0.5),
        node(
            "color",
            Node::VectorParameter {
                name: "Tint".into(),
                default: [0.3, 0.4, 0.5, 1.0],
            },
        ),
        scalar("roughness", "粗糙度", 0.83),
    ];
    graph.links = vec![
        MaterialGraphLinkAsset {
            from_node: "color".into(),
            from_pin: "value".into(),
            to_node: "output".into(),
            to_pin: "base_color".into(),
        },
        MaterialGraphLinkAsset {
            from_node: "roughness".into(),
            from_pin: "value".into(),
            to_node: "output".into(),
            to_pin: "roughness".into(),
        },
    ];
    graph
        .parameters
        .insert("粗糙度".into(), Value::Scalar(0.125));
    let saved = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    let before_material = compile_material_graph(&reopened).unwrap();
    assert_eq!(before_material.roughness, 0.125);
    let plan =
        plan_material_graph_node_deletion(&reopened, &["roughness", "output", "unconnected"])
            .unwrap();
    let mut after = reopened.clone();
    for (index, _) in plan.removed_links().iter().rev() {
        after.links.remove(*index);
    }
    for (index, _) in plan.removed_nodes().iter().rev() {
        after.nodes.remove(*index);
    }
    let mut after = MaterialGraphAsset::from_toml_str(&after.to_toml_string().unwrap()).unwrap();
    assert_eq!(after.nodes.len(), 2);
    assert_eq!(after.nodes[0].id, "output");
    assert_eq!(after.nodes[1].id, "color");
    assert_eq!(after.parameters, reopened.parameters);
    let after_material = compile_material_graph(&after).unwrap();
    assert_ne!(after_material.roughness, before_material.roughness);
    assert_eq!(after_material.base_color, before_material.base_color);
    for (index, node) in plan.removed_nodes() {
        after.nodes.insert(*index, (*node).clone());
    }
    for (index, edge) in plan.removed_links() {
        after.links.insert(*index, (*edge).clone());
    }
    assert_eq!(after.to_toml_string().unwrap(), saved);
    let restored_material = compile_material_graph(&after).unwrap();
    assert_eq!(restored_material.roughness, before_material.roughness);
    assert_eq!(restored_material.base_color, before_material.base_color);
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/deletion.material_graph.toml").unwrap(),
        name: "Delete candidate / 删除候选".into(),
        shader: Some(AssetReference::from_locator(
            AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        )),
        nodes: Vec::new(),
        links: Vec::new(),
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position: [0.0, 0.0],
        kind,
    }
}

fn scalar(id: &str, name: &str, default: f32) -> MaterialGraphNodeAsset {
    node(
        id,
        Node::ScalarParameter {
            name: name.into(),
            default,
        },
    )
}

fn link(from_node: &str, to_node: &str) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: from_node.into(),
        from_pin: "value".into(),
        to_node: to_node.into(),
        to_pin: "a".into(),
    }
}
