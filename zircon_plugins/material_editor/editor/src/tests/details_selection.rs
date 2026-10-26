use std::collections::BTreeMap;

use super::{
    compile_material_graph, project_material_graph_details_selection,
    validate_material_graph_with_diagnostics, MaterialGraphDetailsTarget,
    MaterialGraphDiagnosticTarget,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Value,
};

#[test]
fn material_graph_details_empty_selection_inspects_original_even_with_ambiguous_nodes() {
    let mut graph = graph();
    graph.nodes = vec![
        node("duplicate", Node::Output),
        node("duplicate", Node::Add),
    ];
    graph
        .parameters
        .insert("Orphan".into(), Value::Scalar(-0.0));
    let before = graph.to_toml_string().unwrap();
    let selection = project_material_graph_details_selection(&graph, &[]).unwrap();
    assert!(std::ptr::eq(selection.source(), &graph));
    assert!(selection.selected_nodes().is_empty());
    assert_eq!(selection.targets().len(), 1);
    assert_graph(&selection.targets()[0], &graph);
    assert!(!selection.has_expression());
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_details_output_inspects_graph_and_keeps_root_identity_despite_compile_error() {
    let mut graph = graph();
    graph.nodes = vec![node("output", Node::Output)];
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert!(!diagnostics.is_empty());
    let selection = project_material_graph_details_selection(&graph, &["output"]).unwrap();
    assert_eq!(selection.selected_nodes().len(), 1);
    assert!(std::ptr::eq(selection.selected_nodes()[0], &graph.nodes[0]));
    assert_eq!(selection.targets().len(), 1);
    assert_graph(&selection.targets()[0], &graph);
    assert!(!selection.has_expression());
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
}

#[test]
fn material_graph_details_expressions_use_node_identity_and_source_order_not_shared_name() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("first", "Shared", 0.1),
        node("output", Node::Output),
        scalar("second", "Shared", 0.9),
        node("unselected", Node::Multiply),
    ];
    let selection = project_material_graph_details_selection(&graph, &["second", "first"]).unwrap();
    assert_eq!(selection.selected_nodes().len(), 2);
    assert!(std::ptr::eq(selection.selected_nodes()[0], &graph.nodes[0]));
    assert!(std::ptr::eq(selection.selected_nodes()[1], &graph.nodes[2]));
    assert_eq!(selection.targets().len(), 2);
    assert_expression(&selection.targets()[0], &graph.nodes[0]);
    assert_expression(&selection.targets()[1], &graph.nodes[2]);
    assert!(selection.has_expression());
    assert!(project_material_graph_details_selection(&graph, &["Shared"]).is_err());
}

#[test]
fn material_graph_details_mixed_selection_keeps_each_root_and_one_graph_target() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("before", "Before", 0.1),
        node("root-a", Node::Output),
        node("math", Node::Add),
        node("root-b", Node::Output),
    ];
    let selection =
        project_material_graph_details_selection(&graph, &["root-b", "math", "root-a", "before"])
            .unwrap();
    assert_eq!(selection.selected_nodes().len(), 4);
    for (selected, source) in selection.selected_nodes().iter().zip(&graph.nodes) {
        assert!(std::ptr::eq(*selected, source));
    }
    assert_eq!(selection.targets().len(), 3);
    assert_expression(&selection.targets()[0], &graph.nodes[0]);
    assert_graph(&selection.targets()[1], &graph);
    assert_expression(&selection.targets()[2], &graph.nodes[2]);
    assert!(selection.has_expression());
}

#[test]
fn material_graph_details_stale_duplicate_or_blank_selection_rejects_whole_projection() {
    let mut graph = graph();
    graph.nodes = vec![scalar("known", "Value", 0.5), node("output", Node::Output)];
    let before = graph.to_toml_string().unwrap();
    for selected in [
        vec!["known", "missing"],
        vec!["known", "known"],
        vec!["known", ""],
        vec!["known", "   "],
        vec!["output", "missing"],
    ] {
        let errors = project_material_graph_details_selection(&graph, &selected).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_details_rejects_ambiguous_unselected_source_and_never_chooses_last_match() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("known", "Value", 0.5),
        node("duplicate", Node::Add),
        node("duplicate", Node::Multiply),
    ];
    assert!(project_material_graph_details_selection(&graph, &["known"]).is_err());
    assert!(project_material_graph_details_selection(&graph, &["duplicate"]).is_err());
    graph.nodes[2].id = " ".into();
    assert!(project_material_graph_details_selection(&graph, &["known"]).is_err());
}

#[test]
fn material_graph_details_exact_keys_preserve_nonfinite_and_wrong_type_values_for_repair() {
    let mut graph = graph();
    let long_key = "参数_保留真实源身份_".repeat(24);
    graph.nodes = vec![
        scalar(" node ", "Gain", 0.1),
        scalar("Node", "Gain", 0.9),
        scalar(&long_key, "粗糙度", f32::NAN),
    ];
    graph
        .parameters
        .insert("粗糙度".into(), Value::Vector([f32::INFINITY; 4]));
    let before = graph.to_toml_string().unwrap();
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert!(!diagnostics.is_empty());
    let selection =
        project_material_graph_details_selection(&graph, &[&long_key, "Node", " node "]).unwrap();
    for (target, source) in selection.targets().iter().zip(&graph.nodes) {
        assert_expression(target, source);
    }
    assert!(project_material_graph_details_selection(&graph, &["node"]).is_err());
    let MaterialGraphDetailsTarget::Expression(last) = &selection.targets()[2] else {
        unreachable!()
    };
    let Node::ScalarParameter { default, .. } = &last.kind else {
        unreachable!()
    };
    assert!(default.is_nan());
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_details_codec_reopen_keeps_author_presence_and_real_pbr_compile() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        scalar("roughness", "粗糙度", 0.5),
        node(
            "color",
            Node::VectorParameter {
                name: "颜色".into(),
                default: [0.1, 0.2, 0.3, 1.0],
            },
        ),
    ];
    graph.links = vec![link("roughness", "roughness"), link("color", "base_color")];
    graph.parameters.insert("粗糙度".into(), Value::Scalar(0.5));
    graph.parameters.insert("孤立".into(), Value::Scalar(-0.0));
    let saved = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    let root = project_material_graph_details_selection(&reopened, &["output"]).unwrap();
    assert_graph(&root.targets()[0], &reopened);
    assert!(std::ptr::eq(
        root.source().parameters.get("粗糙度").unwrap(),
        reopened.parameters.get("粗糙度").unwrap()
    ));
    let Some(Value::Scalar(orphan)) = root.source().parameters.get("孤立") else {
        unreachable!()
    };
    assert!(orphan.is_sign_negative());
    assert_eq!(root.source().shader, graph.shader);
    let expression = project_material_graph_details_selection(&reopened, &["roughness"]).unwrap();
    assert_expression(&expression.targets()[0], &reopened.nodes[1]);
    let material = compile_material_graph(expression.source()).unwrap();
    assert_eq!(material.roughness, 0.5);
    assert_eq!(material.base_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(reopened.to_toml_string().unwrap(), saved);
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/details.material_graph.toml").unwrap(),
        name: "Material Details / 材质属性".into(),
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

fn link(from: &str, to_pin: &str) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: from.into(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: to_pin.into(),
    }
}

fn assert_graph(target: &MaterialGraphDetailsTarget<'_>, source: &MaterialGraphAsset) {
    let MaterialGraphDetailsTarget::Graph(actual) = target else {
        panic!("expected the original graph Details object")
    };
    assert!(std::ptr::eq(*actual, source));
}

fn assert_expression(target: &MaterialGraphDetailsTarget<'_>, source: &MaterialGraphNodeAsset) {
    let MaterialGraphDetailsTarget::Expression(actual) = target else {
        panic!("expected the original expression Details object")
    };
    assert!(std::ptr::eq(*actual, source));
}
