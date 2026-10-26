use std::collections::BTreeMap;

use super::{
    compile_material_graph, validate_material_graph_with_diagnostics,
    MaterialGraphDiagnosticTarget, MaterialGraphPresentation,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Value,
};

#[test]
fn material_graph_parameter_declarations_do_not_fabricate_rows_from_override_keys() {
    let mut graph = graph();
    graph.nodes.push(node("output", Node::Output));
    graph.parameters.insert("Orphan".into(), Value::Scalar(0.4));
    let before = graph.clone();
    assert_eq!(
        MaterialGraphPresentation::new(&graph)
            .unwrap()
            .parameter_declarations()
            .count(),
        0
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_parameter_declarations_borrow_source_keys_nodes_and_numeric_defaults() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        node(
            "vector",
            Node::VectorParameter {
                name: "颜色_中文与长参数名称".into(),
                default: [0.1, 0.2, 0.3, 0.4],
            },
        ),
        scalar("scalar", "Roughness", 0.75),
        node("math", Node::Add),
    ];
    let before = graph.clone();
    let rows: Vec<_> = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .parameter_declarations()
        .collect();
    assert_eq!(rows.len(), 2);
    assert!(std::ptr::eq(rows[0].node(), &graph.nodes[1]));
    assert!(std::ptr::eq(rows[1].node(), &graph.nodes[2]));
    let Node::VectorParameter { name, .. } = &graph.nodes[1].kind else {
        unreachable!()
    };
    assert_eq!(rows[0].name().as_ptr(), name.as_ptr());
    assert_eq!(rows[0].name(), name);
    assert_eq!(
        rows[0].default_value(),
        &Value::Vector([0.1, 0.2, 0.3, 0.4])
    );
    assert_eq!(rows[1].effective_value(), Some(&Value::Scalar(0.75)));
    assert!(rows.iter().all(|row| row.override_value().is_none()));
    assert!(graph.links.is_empty());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_parameter_declarations_keep_explicit_default_and_zero_override_presence() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("scalar", "Gain", 0.0),
        node(
            "vector",
            Node::VectorParameter {
                name: "Tint".into(),
                default: [0.0; 4],
            },
        ),
    ];
    graph.parameters.insert("Gain".into(), Value::Scalar(-0.0));
    graph
        .parameters
        .insert("Tint".into(), Value::Vector([0.0; 4]));
    let saved = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    let rows: Vec<_> = MaterialGraphPresentation::new(&reopened)
        .unwrap()
        .parameter_declarations()
        .collect();
    assert!(rows.iter().all(|row| row.override_value().is_some()));
    for row in &rows {
        assert!(std::ptr::eq(
            row.effective_value().unwrap(),
            reopened.parameters.get(row.name()).unwrap()
        ));
    }
    let Some(Value::Scalar(value)) = rows[0].effective_value() else {
        unreachable!()
    };
    assert!(value.is_sign_negative());
}

#[test]
fn material_graph_parameter_declarations_preserve_wrong_type_for_diagnostic_repair() {
    let mut graph = graph();
    graph.nodes = vec![scalar("roughness", "Roughness", 0.5)];
    graph
        .parameters
        .insert("Roughness".into(), Value::Vector([0.2; 4]));
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    let before = graph.clone();
    let row = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .parameter_declarations()
        .next()
        .unwrap();
    assert_eq!(row.default_value(), &Value::Scalar(0.5));
    assert!(row.effective_value().is_none());
    assert!(std::ptr::eq(
        row.override_value().unwrap(),
        graph.parameters.get("Roughness").unwrap()
    ));
    assert!(diagnostics.iter().any(|entry| {
        entry.target
            == MaterialGraphDiagnosticTarget::Parameter {
                name: "Roughness".into(),
            }
            && entry.message.contains("incompatible override type")
    }));
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
    assert_eq!(graph, before);
}

#[test]
fn material_graph_parameter_declarations_keep_duplicate_names_and_different_defaults() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("first", "Shared", 0.1),
        scalar("second", "Shared", 0.9),
        node(
            "vector",
            Node::VectorParameter {
                name: "Shared".into(),
                default: [0.3; 4],
            },
        ),
    ];
    graph.parameters.insert("Shared".into(), Value::Scalar(0.7));
    let rows: Vec<_> = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .parameter_declarations()
        .collect();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].default_value(), &Value::Scalar(0.1));
    assert_eq!(rows[1].default_value(), &Value::Scalar(0.9));
    for (index, row) in rows.iter().enumerate() {
        assert!(std::ptr::eq(row.node(), &graph.nodes[index]));
        assert_eq!(row.name(), "Shared");
    }
    assert_eq!(rows[0].effective_value(), Some(&Value::Scalar(0.7)));
    assert_eq!(rows[1].effective_value(), Some(&Value::Scalar(0.7)));
    assert!(rows[2].effective_value().is_none());
}

#[test]
fn material_graph_parameter_declarations_do_not_casefold_trim_or_remap_renamed_keys() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("upper", "Gain", 0.1),
        scalar("lower", "gain", 0.2),
        scalar("spaces", " Gain ", 0.3),
        scalar("unicode", "增益", 0.4),
    ];
    graph.parameters.insert("gain".into(), Value::Scalar(0.8));
    {
        let rows: Vec<_> = MaterialGraphPresentation::new(&graph)
            .unwrap()
            .parameter_declarations()
            .collect();
        assert_eq!(rows[0].effective_value(), Some(&Value::Scalar(0.1)));
        assert_eq!(rows[1].effective_value(), Some(&Value::Scalar(0.8)));
        assert_eq!(rows[2].name(), " Gain ");
        assert!(rows[2].override_value().is_none());
        assert_eq!(rows[3].name(), "增益");
    }
    let Node::ScalarParameter { name, .. } = &mut graph.nodes[1].kind else {
        unreachable!()
    };
    *name = "Renamed".into();
    let rows: Vec<_> = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .parameter_declarations()
        .collect();
    assert_eq!(rows[1].node().id, "lower");
    assert_eq!(rows[1].name(), "Renamed");
    assert_eq!(rows[1].effective_value(), Some(&Value::Scalar(0.2)));
    assert!(rows[1].override_value().is_none());
    assert_eq!(graph.parameters.get("gain"), Some(&Value::Scalar(0.8)));
}

#[test]
fn material_graph_parameter_declarations_survive_codec_and_follow_real_pbr_compile_and_reset() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        scalar("roughness", "粗糙度", 0.6),
        node(
            "color",
            Node::VectorParameter {
                name: "颜色".into(),
                default: [0.1, 0.2, 0.3, 1.0],
            },
        ),
    ];
    graph.links = vec![link("color", "base_color"), link("roughness", "roughness")];
    graph
        .parameters
        .insert("粗糙度".into(), Value::Scalar(0.125));
    graph
        .parameters
        .insert("颜色".into(), Value::Vector([0.3, 0.4, 0.5, 0.8]));
    let saved = graph.to_toml_string().unwrap();
    let mut reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    {
        let rows: Vec<_> = MaterialGraphPresentation::new(&reopened)
            .unwrap()
            .parameter_declarations()
            .collect();
        assert_eq!(rows[0].name(), "粗糙度");
        assert_eq!(rows[0].effective_value(), Some(&Value::Scalar(0.125)));
        assert_eq!(rows[1].name(), "颜色");
        assert!(rows.iter().all(|row| row.override_value().is_some()));
    }
    let material = compile_material_graph(&reopened).unwrap();
    assert_eq!(material.roughness, 0.125);
    assert_eq!(material.base_color, [0.3, 0.4, 0.5, 0.8]);
    assert_eq!(reopened.to_toml_string().unwrap(), saved);
    reopened.parameters.remove("粗糙度");
    reopened.parameters.remove("颜色");
    let reset = MaterialGraphAsset::from_toml_str(&reopened.to_toml_string().unwrap()).unwrap();
    assert!(MaterialGraphPresentation::new(&reset)
        .unwrap()
        .parameter_declarations()
        .all(|row| row.override_value().is_none()));
    let material = compile_material_graph(&reset).unwrap();
    assert_eq!(material.roughness, 0.6);
    assert_eq!(material.base_color, [0.1, 0.2, 0.3, 1.0]);
    assert_eq!(reset.nodes, graph.nodes);
}

#[test]
fn material_graph_parameter_declarations_keep_nonfinite_sources_and_validation_errors() {
    let mut graph = graph();
    graph.nodes = vec![scalar("bad-default", "Invalid", f32::NAN)];
    graph
        .parameters
        .insert("Invalid".into(), Value::Scalar(f32::INFINITY));
    let before = graph.to_toml_string().unwrap();
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    let row = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .parameter_declarations()
        .next()
        .unwrap();
    let Value::Scalar(default) = row.default_value() else {
        unreachable!()
    };
    assert!(default.is_nan());
    let Some(Value::Scalar(value)) = row.effective_value() else {
        unreachable!()
    };
    assert!(value.is_infinite());
    assert!(diagnostics
        .iter()
        .any(|entry| entry.message.contains("non-finite default")));
    assert!(diagnostics
        .iter()
        .any(|entry| entry.message.contains("non-finite override")));
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/parameters.material_graph.toml").unwrap(),
        name: "Parameter declarations / 参数声明".into(),
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

fn link(from_node: &str, to_pin: &str) -> MaterialGraphLinkAsset {
    MaterialGraphLinkAsset {
        from_node: from_node.into(),
        from_pin: "value".into(),
        to_node: "output".into(),
        to_pin: to_pin.into(),
    }
}
