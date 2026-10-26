use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_details_edit,
    project_material_graph_details_selection, validate_material_graph_with_diagnostics,
    MaterialGraphDetailsEdit, MaterialGraphDetailsTarget, MaterialGraphDiagnosticTarget,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node, MaterialGraphParameterAsset as Value,
};
use zircon_runtime_interface::resource::{AssetUuid, ResourceKind};

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

#[test]
fn material_graph_details_name_edit_borrows_exact_chinese_and_long_request_without_lookup() {
    let graph = graph();
    let original = graph.to_toml_string().unwrap();
    let name = format!("  材质名称_{}  ", "保留作者文字".repeat(40));
    let plan =
        plan_material_graph_details_edit(&graph, MaterialGraphDetailsEdit::Name(&name), |_| {
            panic!("a Name edit does not resolve resources")
        })
        .unwrap();
    assert!(std::ptr::eq(plan.source(), &graph));
    assert!(plan.changes_source());
    let MaterialGraphDetailsEdit::Name(before) = plan.previous_value() else {
        unreachable!()
    };
    let MaterialGraphDetailsEdit::Name(after) = plan.edit() else {
        unreachable!()
    };
    assert!(std::ptr::eq(before, graph.name.as_str()));
    assert!(std::ptr::eq(after, name.as_str()));
    assert_eq!(graph.to_toml_string().unwrap(), original);
}

#[test]
fn material_graph_details_empty_name_is_explicit_and_equal_text_is_a_noop() {
    let graph = graph();
    let same_text = graph.name.clone();
    let unchanged = plan_material_graph_details_edit(
        &graph,
        MaterialGraphDetailsEdit::Name(&same_text),
        |_| panic!("equal text does not resolve resources"),
    )
    .unwrap();
    assert!(!unchanged.changes_source());
    let empty =
        plan_material_graph_details_edit(&graph, MaterialGraphDetailsEdit::Name(""), |_| {
            panic!("an empty name does not resolve resources")
        })
        .unwrap();
    assert!(empty.changes_source());
    assert_eq!(empty.edit(), MaterialGraphDetailsEdit::Name(""));
    assert_eq!(graph.name, same_text);
}

#[test]
fn material_graph_details_shader_edit_resolves_full_pair_once_even_with_repairable_graph() {
    let mut graph = graph();
    graph.nodes = vec![
        node("duplicate", Node::Add),
        node("duplicate", Node::Multiply),
    ];
    let original = graph.to_toml_string().unwrap();
    let proposed = AssetReference::new(
        AssetUuid::from_stable_label("details shader existing test identity"),
        AssetUri::parse("res://shaders/actual_shader_without_extension").unwrap(),
    );
    let mut calls = 0;
    let plan = plan_material_graph_details_edit(
        &graph,
        MaterialGraphDetailsEdit::Shader(Some(&proposed)),
        |reference| {
            calls += 1;
            assert!(std::ptr::eq(reference, &proposed));
            assert_eq!(reference.uuid, proposed.uuid);
            assert_eq!(reference.locator, proposed.locator);
            Ok(ResourceKind::Shader)
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert!(plan.changes_source());
    assert_eq!(
        plan.previous_value(),
        MaterialGraphDetailsEdit::Shader(graph.shader.as_ref())
    );
    assert_eq!(graph.to_toml_string().unwrap(), original);
}

#[test]
fn material_graph_details_equal_shader_pair_and_absent_reference_are_noops_without_lookup() {
    let mut graph = graph();
    let same_pair = graph.shader.clone().unwrap();
    let plan = plan_material_graph_details_edit(
        &graph,
        MaterialGraphDetailsEdit::Shader(Some(&same_pair)),
        |_| panic!("an unchanged pair needs no resolver, even if currently unavailable"),
    )
    .unwrap();
    assert!(!plan.changes_source());
    graph.shader = None;
    let plan =
        plan_material_graph_details_edit(&graph, MaterialGraphDetailsEdit::Shader(None), |_| {
            panic!("an absent reference needs no resolver")
        })
        .unwrap();
    assert!(!plan.changes_source());
}

#[test]
fn material_graph_details_shader_uuid_and_locator_changes_each_require_current_resolution() {
    let graph = graph();
    let original = graph.shader.as_ref().unwrap();
    let mut moved = original.clone();
    moved.locator = AssetUri::parse("res://shaders/moved.wgsl").unwrap();
    let mut replaced = original.clone();
    replaced.uuid = AssetUuid::from_stable_label("details different persistent shader");
    for reference in [&moved, &replaced] {
        let mut calls = 0;
        let plan = plan_material_graph_details_edit(
            &graph,
            MaterialGraphDetailsEdit::Shader(Some(reference)),
            |actual| {
                calls += 1;
                assert_eq!(actual, reference);
                Ok(ResourceKind::Shader)
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        assert!(plan.changes_source());
    }
    assert_eq!(graph.shader.as_ref().unwrap(), original);
}

#[test]
fn material_graph_details_shader_rejects_wrong_kind_and_resolution_error_without_partial_write() {
    let graph = graph();
    let before = graph.to_toml_string().unwrap();
    let proposed =
        AssetReference::from_locator(AssetUri::parse("res://shaders/misleading.wgsl").unwrap());
    for resolution in [
        Ok(ResourceKind::Texture),
        Ok(ResourceKind::Material),
        Ok(ResourceKind::MaterialGraph),
        Err("the UUID/locator pair is not in this project".to_owned()),
    ] {
        let mut calls = 0;
        let errors = plan_material_graph_details_edit(
            &graph,
            MaterialGraphDetailsEdit::Shader(Some(&proposed)),
            |_| {
                calls += 1;
                resolution.clone()
            },
        )
        .unwrap_err();
        assert_eq!(calls, 1);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert_eq!(graph.to_toml_string().unwrap(), before);
    }
}

#[test]
fn material_graph_details_shader_clear_preserves_authored_none_through_codec_and_compile() {
    let mut source = graph();
    source.nodes = vec![
        node("output", Node::Output),
        scalar("roughness", "粗糙度", 0.3),
    ];
    source.links = vec![link("roughness", "roughness")];
    source.parameters.insert("孤立".into(), Value::Scalar(-0.0));
    let plan =
        plan_material_graph_details_edit(&source, MaterialGraphDetailsEdit::Shader(None), |_| {
            panic!("clearing an authored reference does not resolve the old shader")
        })
        .unwrap();
    assert!(plan.changes_source());
    assert!(matches!(
        plan.previous_value(),
        MaterialGraphDetailsEdit::Shader(Some(_))
    ));
    let MaterialGraphDetailsEdit::Shader(after) = plan.edit() else {
        unreachable!()
    };
    // Exercise the candidate against the real asset codec/compiler. This test
    // copy is not an implementation of the host's document/history transaction.
    let mut edited = source.clone();
    edited.shader = after.cloned();
    let reopened = MaterialGraphAsset::from_toml_str(&edited.to_toml_string().unwrap()).unwrap();
    assert!(reopened.shader.is_none());
    assert_eq!(reopened.nodes, source.nodes);
    assert_eq!(reopened.links, source.links);
    let Some(Value::Scalar(orphan)) = reopened.parameters.get("孤立") else {
        unreachable!()
    };
    assert!(orphan.is_sign_negative());
    let material = compile_material_graph(&reopened).unwrap();
    assert_eq!(material.roughness, 0.3);
    assert!(source.shader.is_some());
}

#[test]
fn material_graph_details_name_and_shader_candidates_survive_real_codec_with_source_identity() {
    let mut source = graph();
    source.nodes = vec![
        node("output", Node::Output),
        scalar("roughness", "粗糙度", 0.1),
    ];
    source.links = vec![link("roughness", "roughness")];
    source
        .parameters
        .insert("粗糙度".into(), Value::Scalar(0.72));
    let before = source.to_toml_string().unwrap();
    let name = "  新材质 / 名称  ";
    let shader = AssetReference::new(
        AssetUuid::from_stable_label("details retained shader identity"),
        AssetUri::parse("res://shaders/moved.wgsl").unwrap(),
    );
    let name_plan =
        plan_material_graph_details_edit(&source, MaterialGraphDetailsEdit::Name(name), |_| {
            panic!("Name edits never resolve resources")
        })
        .unwrap();
    let shader_plan = plan_material_graph_details_edit(
        &source,
        MaterialGraphDetailsEdit::Shader(Some(&shader)),
        |_| Ok(ResourceKind::Shader),
    )
    .unwrap();
    let mut edited = source.clone();
    let MaterialGraphDetailsEdit::Name(after_name) = name_plan.edit() else {
        unreachable!()
    };
    let MaterialGraphDetailsEdit::Shader(after_shader) = shader_plan.edit() else {
        unreachable!()
    };
    edited.name = after_name.into();
    edited.shader = after_shader.cloned();
    let reopened = MaterialGraphAsset::from_toml_str(&edited.to_toml_string().unwrap()).unwrap();
    assert_eq!(reopened.name, name);
    assert_eq!(reopened.shader, Some(shader));
    assert_eq!(reopened.uri, source.uri);
    assert_eq!(reopened.nodes, source.nodes);
    assert_eq!(reopened.links, source.links);
    assert_eq!(reopened.parameters, source.parameters);
    assert_eq!(compile_material_graph(&reopened).unwrap().roughness, 0.72);
    assert_eq!(source.to_toml_string().unwrap(), before);
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
