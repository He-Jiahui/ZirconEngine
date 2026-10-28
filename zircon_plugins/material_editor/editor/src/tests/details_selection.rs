use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_details_edit,
    plan_material_graph_expression_details_batch_edit, plan_material_graph_expression_details_edit,
    project_material_graph_details_selection, validate_material_graph_with_diagnostics,
    MaterialGraphDetailsEdit, MaterialGraphDetailsTarget, MaterialGraphDetailsValue as CommonValue,
    MaterialGraphDiagnosticTarget, MaterialGraphExpressionDetailsBatchEdit as BatchEdit,
    MaterialGraphExpressionDetailsEdit as ExpressionEdit,
    MaterialGraphExpressionDetailsValues as ExpressionValues, MaterialGraphPresentation,
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

#[test]
fn material_graph_expression_details_scalar_default_keeps_borrowed_node_and_map_presence() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("first", "Shared", 0.1),
        scalar("second", "Shared", 0.9),
    ];
    graph.nodes[1].position = [-64.0, 512.0];
    graph.parameters.insert("Shared".into(), Value::Scalar(0.9));
    graph
        .parameters
        .insert("Orphan".into(), Value::Vector([0.0; 4]));
    let before = graph.to_toml_string().unwrap();
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        "second",
        ExpressionEdit::ScalarDefault(0.4),
        |_| panic!("numeric fields do not resolve resources"),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.source(), &graph));
    assert!(std::ptr::eq(plan.node(), &graph.nodes[1]));
    assert_eq!(plan.previous_value(), ExpressionEdit::ScalarDefault(0.9));
    assert_eq!(plan.edit(), ExpressionEdit::ScalarDefault(0.4));
    assert!(plan.changes_source());
    assert_eq!(plan.node().id, "second");
    assert_eq!(plan.node().position, [-64.0, 512.0]);
    assert_eq!(
        plan.source().parameters.get("Shared"),
        Some(&Value::Scalar(0.9))
    );
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_vector_default_keeps_all_channels_and_hdr_values() {
    let mut graph = graph();
    graph.nodes = vec![node(
        "color",
        Node::VectorParameter {
            name: "颜色".into(),
            default: [0.1, 0.2, 0.3, 1.0],
        },
    )];
    let requested = [-2.0, 4.0, 0.25, -0.0];
    let before = graph.to_toml_string().unwrap();
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        "color",
        ExpressionEdit::VectorDefault(requested),
        |_| panic!("numeric fields do not resolve resources"),
    )
    .unwrap();
    assert_eq!(
        plan.previous_value(),
        ExpressionEdit::VectorDefault([0.1, 0.2, 0.3, 1.0])
    );
    let ExpressionEdit::VectorDefault(after) = plan.edit() else {
        unreachable!()
    };
    assert_eq!(after, requested);
    assert!(after[3].is_sign_negative());
    assert!(plan.changes_source());
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_defaults_reject_nonfinite_without_mutating_source() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("scalar", "Scalar", 0.5),
        node(
            "vector",
            Node::VectorParameter {
                name: "Vector".into(),
                default: [0.0; 4],
            },
        ),
    ];
    let before = graph.to_toml_string().unwrap();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for (id, edit) in [
            ("scalar", ExpressionEdit::ScalarDefault(invalid)),
            (
                "vector",
                ExpressionEdit::VectorDefault([0.0, 0.0, invalid, 1.0]),
            ),
        ] {
            let errors = plan_material_graph_expression_details_edit(&graph, id, edit, |_| {
                panic!("no lookup")
            })
            .unwrap_err();
            assert_eq!(errors.len(), 1);
            assert_eq!(
                errors[0].target,
                MaterialGraphDiagnosticTarget::Node { node_id: id.into() }
            );
        }
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_numeric_noops_keep_bit_identity_and_signed_zero() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("scalar", "Scalar", 0.0),
        node(
            "vector",
            Node::VectorParameter {
                name: "Vector".into(),
                default: [0.0; 4],
            },
        ),
    ];
    for (id, unchanged, signed_zero) in [
        (
            "scalar",
            ExpressionEdit::ScalarDefault(0.0),
            ExpressionEdit::ScalarDefault(-0.0),
        ),
        (
            "vector",
            ExpressionEdit::VectorDefault([0.0; 4]),
            ExpressionEdit::VectorDefault([0.0, -0.0, 0.0, 0.0]),
        ),
    ] {
        let noop = plan_material_graph_expression_details_edit(&graph, id, unchanged, |_| {
            panic!("no lookup")
        })
        .unwrap();
        assert!(!noop.changes_source());
        let changed = plan_material_graph_expression_details_edit(&graph, id, signed_zero, |_| {
            panic!("no lookup")
        })
        .unwrap();
        assert!(
            changed.changes_source(),
            "signed zero is an authored value change"
        );
    }
    let mut authored = graph.clone();
    let Node::ScalarParameter { default, .. } = &mut authored.nodes[0].kind else {
        unreachable!()
    };
    *default = -0.0;
    let Node::VectorParameter { default, .. } = &mut authored.nodes[1].kind else {
        unreachable!()
    };
    default[1] = -0.0;
    let reopened = MaterialGraphAsset::from_toml_str(&authored.to_toml_string().unwrap()).unwrap();
    let scalar = plan_material_graph_expression_details_edit(
        &reopened,
        "scalar",
        ExpressionEdit::ScalarDefault(-0.0),
        |_| panic!("no lookup"),
    )
    .unwrap();
    let vector = plan_material_graph_expression_details_edit(
        &reopened,
        "vector",
        ExpressionEdit::VectorDefault([0.0, -0.0, 0.0, 0.0]),
        |_| panic!("no lookup"),
    )
    .unwrap();
    assert!(
        !scalar.changes_source(),
        "codec must retain the authored scalar sign bit"
    );
    assert!(
        !vector.changes_source(),
        "codec must retain the authored channel sign bit"
    );
}

#[test]
fn material_graph_expression_details_repairs_invalid_default_without_resetting_override() {
    let mut graph = graph();
    graph.nodes = vec![scalar("roughness", "粗糙度", f32::NAN)];
    graph
        .parameters
        .insert("粗糙度".into(), Value::Vector([f32::INFINITY; 4]));
    let before = graph.to_toml_string().unwrap();
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        "roughness",
        ExpressionEdit::ScalarDefault(0.72),
        |_| panic!("no lookup"),
    )
    .unwrap();
    let ExpressionEdit::ScalarDefault(previous) = plan.previous_value() else {
        unreachable!()
    };
    assert!(previous.is_nan());
    assert!(plan.changes_source());
    assert!(
        matches!(plan.source().parameters.get("粗糙度"), Some(Value::Vector(values)) if values[0].is_infinite())
    );
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_rejects_stale_or_ambiguous_keys_and_keeps_exact_id() {
    let mut graph = graph();
    let long_id = "参数_源节点_".repeat(24);
    graph.nodes = vec![scalar(" node ", "Gain", 0.1), scalar(&long_id, "Gain", 0.2)];
    for id in ["", "   ", "node", "Node", "missing"] {
        assert!(plan_material_graph_expression_details_edit(
            &graph,
            id,
            ExpressionEdit::ScalarDefault(0.4),
            |_| panic!("no lookup")
        )
        .is_err());
    }
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        &long_id,
        ExpressionEdit::ScalarDefault(0.4),
        |_| panic!("no lookup"),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.node(), &graph.nodes[1]));
    graph.nodes.push(node("duplicate", Node::Add));
    graph.nodes.push(node("duplicate", Node::Multiply));
    assert!(plan_material_graph_expression_details_edit(
        &graph,
        " node ",
        ExpressionEdit::ScalarDefault(0.4),
        |_| panic!("no lookup")
    )
    .is_err());
}

#[test]
fn material_graph_expression_details_texture_requires_current_full_pair_kind_once() {
    let mut graph = graph();
    let original =
        AssetReference::from_locator(AssetUri::parse("res://textures/albedo.png").unwrap());
    graph.nodes = vec![node(
        "albedo",
        Node::TextureSample {
            texture: original.clone(),
        },
    )];
    let mut moved = original.clone();
    moved.locator = AssetUri::parse("res://textures/moved.not_an_image_extension").unwrap();
    let mut replaced = original.clone();
    replaced.uuid = AssetUuid::from_stable_label("expression details replacement texture test");
    let before = graph.to_toml_string().unwrap();
    for proposed in [&moved, &replaced] {
        let mut calls = 0;
        let plan = plan_material_graph_expression_details_edit(
            &graph,
            "albedo",
            ExpressionEdit::Texture(proposed),
            |reference| {
                calls += 1;
                assert!(std::ptr::eq(reference, proposed));
                Ok(ResourceKind::Texture)
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        assert!(plan.changes_source());
        assert!(std::ptr::eq(plan.node(), &graph.nodes[0]));
        let ExpressionEdit::Texture(old) = plan.previous_value() else {
            unreachable!()
        };
        assert_eq!(old, &original);
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_texture_noop_preserves_unavailable_authored_pair() {
    let mut graph = graph();
    let texture =
        AssetReference::from_locator(AssetUri::parse("res://textures/unavailable.png").unwrap());
    graph.nodes = vec![node(
        "texture",
        Node::TextureSample {
            texture: texture.clone(),
        },
    )];
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        "texture",
        ExpressionEdit::Texture(&texture),
        |_| panic!("an unchanged full pair needs no lookup"),
    )
    .unwrap();
    assert!(!plan.changes_source());
    assert!(std::ptr::eq(plan.source(), &graph));
    let ExpressionEdit::Texture(request) = plan.edit() else {
        unreachable!()
    };
    assert!(std::ptr::eq(request, &texture));
}

#[test]
fn material_graph_expression_details_texture_rejects_wrong_kind_or_resolver_error() {
    let mut graph = graph();
    let original = AssetReference::from_locator(AssetUri::parse("res://textures/old.png").unwrap());
    let proposed =
        AssetReference::from_locator(AssetUri::parse("res://textures/looks_valid.png").unwrap());
    graph.nodes = vec![node("texture", Node::TextureSample { texture: original })];
    let before = graph.to_toml_string().unwrap();
    for result in [
        Ok(ResourceKind::Shader),
        Ok(ResourceKind::Material),
        Err("UUID/locator is not in this catalog".to_owned()),
    ] {
        let mut calls = 0;
        let errors = plan_material_graph_expression_details_edit(
            &graph,
            "texture",
            ExpressionEdit::Texture(&proposed),
            |_| {
                calls += 1;
                result.clone()
            },
        )
        .unwrap_err();
        assert_eq!(calls, 1);
        assert_eq!(
            errors[0].target,
            MaterialGraphDiagnosticTarget::Node {
                node_id: "texture".into()
            }
        );
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_default_and_texture_field_kinds_are_not_interchangeable() {
    let mut graph = graph();
    let texture =
        AssetReference::from_locator(AssetUri::parse("res://textures/albedo.png").unwrap());
    graph.nodes = vec![
        scalar("scalar", "Scalar", 0.5),
        node(
            "vector",
            Node::VectorParameter {
                name: "Vector".into(),
                default: [0.0; 4],
            },
        ),
        node(
            "texture",
            Node::TextureSample {
                texture: texture.clone(),
            },
        ),
        node("output", Node::Output),
        node("add", Node::Add),
        node("multiply", Node::Multiply),
    ];
    for (id, edit) in [
        ("scalar", ExpressionEdit::VectorDefault([0.0; 4])),
        ("scalar", ExpressionEdit::Texture(&texture)),
        ("vector", ExpressionEdit::ScalarDefault(0.0)),
        ("texture", ExpressionEdit::ScalarDefault(0.0)),
        ("output", ExpressionEdit::ScalarDefault(0.0)),
        ("add", ExpressionEdit::ScalarDefault(0.0)),
        ("multiply", ExpressionEdit::ScalarDefault(0.0)),
    ] {
        let errors = plan_material_graph_expression_details_edit(&graph, id, edit, |_| {
            panic!("wrong fields never resolve a reference")
        })
        .unwrap_err();
        assert_eq!(
            errors[0].target,
            MaterialGraphDiagnosticTarget::Node { node_id: id.into() }
        );
    }
}

#[test]
fn material_graph_expression_details_defaults_roundtrip_through_codec_and_real_compile() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        scalar("roughness", "粗糙度", 0.4),
        node(
            "color",
            Node::VectorParameter {
                name: "颜色".into(),
                default: [0.1, 0.2, 0.3, 1.0],
            },
        ),
    ];
    graph.links = vec![link("roughness", "roughness"), link("color", "base_color")];
    graph.parameters.insert("粗糙度".into(), Value::Scalar(0.4));
    graph.parameters.insert("孤立".into(), Value::Scalar(-0.0));
    let before = graph.to_toml_string().unwrap();
    let scalar_plan = plan_material_graph_expression_details_edit(
        &graph,
        "roughness",
        ExpressionEdit::ScalarDefault(0.72),
        |_| panic!("no lookup"),
    )
    .unwrap();
    let vector_plan = plan_material_graph_expression_details_edit(
        &graph,
        "color",
        ExpressionEdit::VectorDefault([0.2, 0.4, 0.6, 1.0]),
        |_| panic!("no lookup"),
    )
    .unwrap();
    let mut authored = graph.clone();
    // Test-only fixture installation; production still requires the real host permit/CAS.
    let ExpressionEdit::ScalarDefault(value) = scalar_plan.edit() else {
        unreachable!()
    };
    let Node::ScalarParameter { default, .. } = &mut authored.nodes[1].kind else {
        unreachable!()
    };
    *default = value;
    let ExpressionEdit::VectorDefault(value) = vector_plan.edit() else {
        unreachable!()
    };
    let Node::VectorParameter { default, .. } = &mut authored.nodes[2].kind else {
        unreachable!()
    };
    *default = value;
    let saved = authored.to_toml_string().unwrap();
    let mut reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    assert_eq!(reopened.parameters, graph.parameters);
    assert_eq!(reopened.links, graph.links);
    assert_eq!(reopened.uri, graph.uri);
    let material = compile_material_graph(&reopened).unwrap();
    assert_eq!(
        material.roughness, 0.4,
        "an equal old override is still present"
    );
    assert_eq!(material.base_color, [0.2, 0.4, 0.6, 1.0]);
    {
        let presentation = MaterialGraphPresentation::new(&reopened).unwrap();
        let rows: Vec<_> = presentation.parameter_declarations().collect();
        assert_eq!(rows[0].default_value(), &Value::Scalar(0.72));
        assert_eq!(rows[0].override_value(), Some(&Value::Scalar(0.4)));
    }
    // An explicit fixture reset exposes the new default; planning never does this reset.
    reopened.parameters.remove("粗糙度");
    let reset = MaterialGraphAsset::from_toml_str(&reopened.to_toml_string().unwrap()).unwrap();
    assert_eq!(compile_material_graph(&reset).unwrap().roughness, 0.72);
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_details_texture_edit_roundtrips_to_real_material_binding() {
    let mut graph = graph();
    let old = AssetReference::from_locator(AssetUri::parse("res://textures/old.png").unwrap());
    let proposed = AssetReference::new(
        AssetUuid::from_stable_label("expression details new binding test"),
        AssetUri::parse("res://textures/new.png").unwrap(),
    );
    graph.nodes = vec![
        node("albedo", Node::TextureSample { texture: old }),
        node("output", Node::Output),
    ];
    graph.links = vec![MaterialGraphLinkAsset {
        from_node: "albedo".into(),
        from_pin: "color".into(),
        to_node: "output".into(),
        to_pin: "base_color".into(),
    }];
    let before = graph.to_toml_string().unwrap();
    let plan = plan_material_graph_expression_details_edit(
        &graph,
        "albedo",
        ExpressionEdit::Texture(&proposed),
        |_| Ok(ResourceKind::Texture),
    )
    .unwrap();
    let ExpressionEdit::Texture(reference) = plan.edit() else {
        unreachable!()
    };
    let mut authored = graph.clone();
    let Node::TextureSample { texture } = &mut authored.nodes[0].kind else {
        unreachable!()
    };
    *texture = reference.clone();
    let reopened = MaterialGraphAsset::from_toml_str(&authored.to_toml_string().unwrap()).unwrap();
    assert_eq!(reopened.nodes[0].id, graph.nodes[0].id);
    assert_eq!(reopened.nodes[0].position, graph.nodes[0].position);
    assert_eq!(reopened.links, graph.links);
    assert_eq!(
        compile_material_graph(&reopened)
            .unwrap()
            .base_color_texture,
        Some(proposed)
    );
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_details_common_scalar_uses_original_defaults_not_override_or_input_order() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("a", "粗糙度A", 0.4),
        scalar("ignored", "Ignored", 9.0),
        scalar("b", "粗糙度B", 0.4),
    ];
    graph
        .parameters
        .insert("粗糙度A".into(), Value::Scalar(0.9));
    graph
        .parameters
        .insert("粗糙度B".into(), Value::Vector([7.0; 4]));
    let before = graph.to_toml_string().unwrap();
    let selection = project_material_graph_details_selection(&graph, &["b", "a"]).unwrap();
    assert!(matches!(
        selection.expression_values(),
        Some(ExpressionValues::ScalarDefault(CommonValue::Common(value)))
            if value.to_bits() == 0.4_f32.to_bits()
    ));
    assert!(std::ptr::eq(selection.selected_nodes()[0], &graph.nodes[0]));
    assert!(std::ptr::eq(selection.selected_nodes()[1], &graph.nodes[2]));
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_details_mixed_scalar_never_recovers_last_value_or_erases_signed_zero() {
    let mut graph = graph();
    for values in [[0.4, 0.8, 0.4], [0.0, -0.0, 0.0]] {
        graph.nodes = vec![
            scalar("a", "A", values[0]),
            scalar("b", "B", values[1]),
            scalar("c", "C", values[2]),
        ];
        let selection = project_material_graph_details_selection(&graph, &["c", "a", "b"]).unwrap();
        assert!(matches!(
            selection.expression_values(),
            Some(ExpressionValues::ScalarDefault(CommonValue::Mixed))
        ));
    }
}

#[test]
fn material_graph_details_vector_common_channels_preserve_mixed_zero_and_repairable_nan() {
    let mut graph = graph();
    let nan = f32::from_bits(0x7fc0_0017);
    graph.nodes = vec![
        node(
            "a",
            Node::VectorParameter {
                name: "A".into(),
                default: [0.1, 0.0, 3.0, nan],
            },
        ),
        node(
            "b",
            Node::VectorParameter {
                name: "B".into(),
                default: [0.1, -0.0, 4.0, nan],
            },
        ),
    ];
    let diagnostics = validate_material_graph_with_diagnostics(&graph);
    assert!(!diagnostics.is_empty());
    let selection = project_material_graph_details_selection(&graph, &["b", "a"]).unwrap();
    let Some(ExpressionValues::VectorDefault(values)) = selection.expression_values() else {
        panic!("same-kind vector fields retain their actual channel schema")
    };
    assert!(
        matches!(values[0], CommonValue::Common(value) if value.to_bits() == 0.1_f32.to_bits())
    );
    assert!(matches!(values[1], CommonValue::Mixed));
    assert!(matches!(values[2], CommonValue::Mixed));
    assert!(matches!(values[3], CommonValue::Common(value) if value.to_bits() == nan.to_bits()));
    assert_eq!(
        validate_material_graph_with_diagnostics(&graph),
        diagnostics
    );
}

#[test]
fn material_graph_details_texture_common_borrows_first_and_compares_uuid_and_locator() {
    let mut graph = graph();
    let first = AssetReference::new(
        AssetUuid::from_stable_label("common texture first"),
        AssetUri::parse("res://textures/first.png").unwrap(),
    );
    for (second, common) in [
        (first.clone(), true),
        (
            AssetReference::new(
                AssetUuid::from_stable_label("common texture second"),
                first.locator.clone(),
            ),
            false,
        ),
        (
            AssetReference::new(
                first.uuid,
                AssetUri::parse("res://textures/second.png").unwrap(),
            ),
            false,
        ),
    ] {
        graph.nodes = vec![
            node(
                "a",
                Node::TextureSample {
                    texture: first.clone(),
                },
            ),
            node("b", Node::TextureSample { texture: second }),
        ];
        // The returned reference remains a source borrow after the projection drops.
        let values = project_material_graph_details_selection(&graph, &["b", "a"])
            .unwrap()
            .expression_values()
            .unwrap();
        let ExpressionValues::Texture(value) = values else {
            panic!("texture schema")
        };
        match value {
            CommonValue::Common(reference) => {
                assert!(common);
                let Node::TextureSample { texture } = &graph.nodes[0].kind else {
                    unreachable!()
                };
                assert!(std::ptr::eq(reference, texture));
            }
            CommonValue::Mixed => assert!(!common),
        }
    }
}

#[test]
fn material_graph_details_without_common_expression_schema_never_invents_a_value() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        node("add", Node::Add),
        node("multiply", Node::Multiply),
        scalar("scalar", "Value", 0.4),
        node(
            "vector",
            Node::VectorParameter {
                name: "Value".into(),
                default: [0.4; 4],
            },
        ),
        node(
            "texture",
            Node::TextureSample {
                texture: AssetReference::from_locator(
                    AssetUri::parse("res://textures/a.png").unwrap(),
                ),
            },
        ),
    ];
    for selected in [
        vec![],
        vec!["output"],
        vec!["add"],
        vec!["multiply"],
        vec!["scalar", "output"],
        vec!["scalar", "add"],
        vec!["scalar", "vector"],
        vec!["vector", "texture"],
    ] {
        let selection = project_material_graph_details_selection(&graph, &selected).unwrap();
        assert!(selection.expression_values().is_none(), "{selected:?}");
    }
}

#[test]
fn material_graph_details_common_nonfinite_source_preserves_bits_instead_of_hiding_repair() {
    let mut graph = graph();
    let nan = f32::from_bits(0x7fc0_0017);
    for value in [nan, f32::INFINITY, f32::NEG_INFINITY, -0.0] {
        graph.nodes = vec![scalar("a", "A", value), scalar("b", "B", value)];
        let selection = project_material_graph_details_selection(&graph, &["a", "b"]).unwrap();
        assert!(matches!(
            selection.expression_values(),
            Some(ExpressionValues::ScalarDefault(CommonValue::Common(actual)))
                if actual.to_bits() == value.to_bits()
        ));
    }
    graph.nodes = vec![
        scalar("a", "A", nan),
        scalar("b", "B", f32::from_bits(0x7fc0_0018)),
    ];
    let selection = project_material_graph_details_selection(&graph, &["a", "b"]).unwrap();
    assert!(matches!(
        selection.expression_values(),
        Some(ExpressionValues::ScalarDefault(CommonValue::Mixed))
    ));
}

#[test]
fn material_graph_expression_batch_scalar_keeps_source_order_noops_and_override_presence() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("first", "共享名", -0.0),
        scalar("unselected", "共享名", 0.9),
        scalar("second", "共享名", 0.4),
    ];
    graph
        .parameters
        .insert("共享名".into(), Value::Vector([3.0; 4]));
    graph
        .parameters
        .insert("orphan".into(), Value::Scalar(-0.0));
    let before = graph.to_toml_string().unwrap();
    let plan = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["second", "first"],
        BatchEdit::ScalarDefault(-0.0),
        |_| panic!("numeric changes do not resolve assets"),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.source(), &graph));
    assert_eq!(plan.edits().len(), 2);
    assert!(std::ptr::eq(plan.edits()[0].node(), &graph.nodes[0]));
    assert!(std::ptr::eq(plan.edits()[1].node(), &graph.nodes[2]));
    assert!(!plan.edits()[0].changes_source());
    assert!(plan.edits()[1].changes_source());
    assert!(plan.changes_source());
    for edit in plan.edits() {
        assert!(std::ptr::eq(edit.source(), &graph));
        let ExpressionEdit::ScalarDefault(value) = edit.edit() else {
            panic!("scalar field expected")
        };
        assert_eq!(value.to_bits(), (-0.0_f32).to_bits());
    }
    assert!(matches!(
        plan.source().parameters.get("共享名"),
        Some(Value::Vector(_))
    ));
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_batch_vector_preserves_each_unedited_channel_bit_pattern() {
    let mut graph = graph();
    let defaults = [
        [1.0, 0.2, f32::from_bits(0x7fc00019), 0.0],
        [-3.0, 0.8, 2.0, -0.0],
    ];
    graph.nodes = defaults
        .iter()
        .enumerate()
        .map(|(index, default)| {
            node(
                &format!("v{index}"),
                Node::VectorParameter {
                    name: format!("Vector{index}"),
                    default: *default,
                },
            )
        })
        .collect();
    let before = graph.to_toml_string().unwrap();
    let plan = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["v1", "v0"],
        BatchEdit::VectorChannels([None, Some(1.5), None, Some(-0.0)]),
        |_| panic!("no lookup"),
    )
    .unwrap();
    for (index, edit) in plan.edits().iter().enumerate() {
        let ExpressionEdit::VectorDefault(after) = edit.edit() else {
            panic!("vector field expected")
        };
        let ExpressionEdit::VectorDefault(previous) = edit.previous_value() else {
            panic!("vector inverse data expected")
        };
        assert_eq!(
            previous.map(f32::to_bits),
            defaults[index].map(f32::to_bits)
        );
        assert_eq!(after[0].to_bits(), defaults[index][0].to_bits());
        assert_eq!(after[1], 1.5, "finite HDR values are not clamped");
        assert_eq!(after[2].to_bits(), defaults[index][2].to_bits());
        assert_eq!(after[3].to_bits(), (-0.0_f32).to_bits());
        assert!(edit.changes_source());
    }
    let noop = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["v0", "v1"],
        BatchEdit::VectorChannels([None; 4]),
        |_| panic!("no lookup"),
    )
    .unwrap();
    assert_eq!(noop.edits().len(), 2);
    assert!(!noop.changes_source());
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_batch_rejects_whole_stale_or_incompatible_selection_before_lookup() {
    let mut graph = graph();
    let texture = AssetReference::from_locator(AssetUri::parse("res://textures/old.png").unwrap());
    graph.nodes = vec![
        node("texture", Node::TextureSample { texture }),
        scalar("scalar", "Scalar", 0.4),
        node("output", Node::Output),
        node("add", Node::Add),
    ];
    let proposed = AssetReference::from_locator(AssetUri::parse("res://textures/new.png").unwrap());
    let before = graph.to_toml_string().unwrap();
    for selected in [
        vec![],
        vec!["texture", "missing"],
        vec!["texture", "texture"],
        vec!["texture", " "],
        vec!["texture", "Texture"],
        vec!["texture", "scalar"],
        vec!["texture", "output"],
        vec!["texture", "add"],
    ] {
        let result = plan_material_graph_expression_details_batch_edit(
            &graph,
            &selected,
            BatchEdit::Texture(&proposed),
            |_| panic!("preflight must finish first"),
        );
        assert!(result.is_err(), "{selected:?}");
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
    graph.nodes.push(node("duplicate", Node::Add));
    graph.nodes.push(node("duplicate", Node::Multiply));
    assert!(plan_material_graph_expression_details_batch_edit(
        &graph,
        &["texture"],
        BatchEdit::Texture(&proposed),
        |_| panic!("ambiguous source"),
    )
    .is_err());
}

#[test]
fn material_graph_expression_batch_rejects_nonfinite_requests_without_partial_candidate() {
    let mut graph = graph();
    graph.nodes = vec![
        scalar("s0", "A", 0.2),
        scalar("s1", "B", 0.8),
        node(
            "v0",
            Node::VectorParameter {
                name: "V0".into(),
                default: [1.0; 4],
            },
        ),
        node(
            "v1",
            Node::VectorParameter {
                name: "V1".into(),
                default: [2.0; 4],
            },
        ),
    ];
    let before = graph.to_toml_string().unwrap();
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        for (selected, edit) in [
            (["s1", "s0"], BatchEdit::ScalarDefault(invalid)),
            (
                ["v1", "v0"],
                BatchEdit::VectorChannels([None, Some(invalid), None, None]),
            ),
        ] {
            let errors =
                plan_material_graph_expression_details_batch_edit(&graph, &selected, edit, |_| {
                    panic!("no lookup")
                })
                .unwrap_err();
            assert_eq!(errors.len(), 1);
            let MaterialGraphDiagnosticTarget::Node { node_id } = &errors[0].target else {
                panic!("exact expression target expected")
            };
            assert_eq!(node_id, selected[1], "diagnostic follows source order");
        }
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_batch_texture_resolves_one_full_pair_after_all_kinds_pass() {
    let mut graph = graph();
    let proposed =
        AssetReference::from_locator(AssetUri::parse("res://textures/desired.png").unwrap());
    let mut moved = proposed.clone();
    moved.locator = AssetUri::parse("res://textures/moved.png").unwrap();
    let mut replaced = proposed.clone();
    replaced.uuid = AssetUuid::from_stable_label("batch texture replacement");
    graph.nodes = vec![
        node(
            "noop",
            Node::TextureSample {
                texture: proposed.clone(),
            },
        ),
        node(
            "moved",
            Node::TextureSample {
                texture: moved.clone(),
            },
        ),
        node(
            "replaced",
            Node::TextureSample {
                texture: replaced.clone(),
            },
        ),
    ];
    let before = graph.to_toml_string().unwrap();
    let mut calls = 0;
    let plan = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["replaced", "noop", "moved"],
        BatchEdit::Texture(&proposed),
        |reference| {
            calls += 1;
            assert!(std::ptr::eq(reference, &proposed));
            Ok(ResourceKind::Texture)
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(plan.edits().len(), 3);
    assert!(!plan.edits()[0].changes_source());
    for (index, previous) in [&proposed, &moved, &replaced].into_iter().enumerate() {
        let ExpressionEdit::Texture(actual) = plan.edits()[index].previous_value() else {
            panic!("original texture expected")
        };
        assert_eq!(actual, previous);
        let ExpressionEdit::Texture(actual) = plan.edits()[index].edit() else {
            panic!("new texture expected")
        };
        assert!(std::ptr::eq(actual, &proposed));
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_batch_texture_noop_needs_no_lookup_and_errors_keep_exact_target() {
    let mut graph = graph();
    let proposed =
        AssetReference::from_locator(AssetUri::parse("res://textures/desired.png").unwrap());
    let old = AssetReference::from_locator(AssetUri::parse("res://textures/old.png").unwrap());
    graph.nodes = vec![
        node(
            "unchanged",
            Node::TextureSample {
                texture: proposed.clone(),
            },
        ),
        node("changed", Node::TextureSample { texture: old }),
    ];
    let noop = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["unchanged"],
        BatchEdit::Texture(&proposed),
        |_| panic!("unavailable no-op"),
    )
    .unwrap();
    assert!(!noop.changes_source());
    let before = graph.to_toml_string().unwrap();
    for error in [
        Ok(ResourceKind::Material),
        Ok(ResourceKind::Shader),
        Err("not in current catalog".into()),
    ] {
        let mut calls = 0;
        let errors = plan_material_graph_expression_details_batch_edit(
            &graph,
            &["changed", "unchanged"],
            BatchEdit::Texture(&proposed),
            |_| {
                calls += 1;
                error.clone()
            },
        )
        .unwrap_err();
        assert_eq!(calls, 1);
        assert_eq!(
            errors[0].target,
            MaterialGraphDiagnosticTarget::Node {
                node_id: "changed".into()
            }
        );
    }
    assert_eq!(graph.to_toml_string().unwrap(), before);
}

#[test]
fn material_graph_expression_batch_codec_and_inverse_keep_real_compile_and_source_identity() {
    let mut graph = graph();
    graph.nodes = vec![
        node("output", Node::Output),
        scalar("a", "A", 0.2),
        scalar("b", "B", 0.5),
        node("product", Node::Multiply),
        node(
            "color",
            Node::VectorParameter {
                name: "Color".into(),
                default: [0.1, 0.2, 0.3, 1.0],
            },
        ),
    ];
    graph.links = vec![
        link("color", "base_color"),
        link("product", "roughness"),
        MaterialGraphLinkAsset {
            from_node: "a".into(),
            from_pin: "value".into(),
            to_node: "product".into(),
            to_pin: "a".into(),
        },
        MaterialGraphLinkAsset {
            from_node: "b".into(),
            from_pin: "value".into(),
            to_node: "product".into(),
            to_pin: "b".into(),
        },
    ];
    graph
        .parameters
        .insert("orphan".into(), Value::Scalar(-0.0));
    let before = graph.to_toml_string().unwrap();
    let baseline = compile_material_graph(&graph).unwrap();
    let plan = plan_material_graph_expression_details_batch_edit(
        &graph,
        &["b", "a"],
        BatchEdit::ScalarDefault(0.8),
        |_| panic!("no lookup"),
    )
    .unwrap();
    // Test-only installation of the finite field candidate, never a production transaction/history.
    let mut authored = graph.clone();
    for edit in plan.edits() {
        let ExpressionEdit::ScalarDefault(value) = edit.edit() else {
            panic!("scalar expected")
        };
        let node = authored
            .nodes
            .iter_mut()
            .find(|node| node.id == edit.node().id)
            .unwrap();
        let Node::ScalarParameter { default, .. } = &mut node.kind else {
            panic!("scalar node expected")
        };
        *default = value;
    }
    let mut reopened =
        MaterialGraphAsset::from_toml_str(&authored.to_toml_string().unwrap()).unwrap();
    assert_eq!(
        compile_material_graph(&reopened).unwrap().roughness,
        0.8 * 0.8
    );
    assert_eq!(reopened.links, graph.links);
    assert_eq!(reopened.parameters, graph.parameters);
    assert_eq!(reopened.shader, graph.shader);
    assert_eq!(reopened.uri, graph.uri);
    for (actual, original) in reopened.nodes.iter().zip(&graph.nodes) {
        assert_eq!(actual.id, original.id);
        assert_eq!(actual.position, original.position);
    }
    for edit in plan.edits() {
        let ExpressionEdit::ScalarDefault(previous) = edit.previous_value() else {
            panic!("scalar inverse expected")
        };
        let node = reopened
            .nodes
            .iter_mut()
            .find(|node| node.id == edit.node().id)
            .unwrap();
        let Node::ScalarParameter { default, .. } = &mut node.kind else {
            panic!("scalar node expected")
        };
        *default = previous;
    }
    let restored = MaterialGraphAsset::from_toml_str(&reopened.to_toml_string().unwrap()).unwrap();
    assert_eq!(compile_material_graph(&restored).unwrap(), baseline);
    assert_eq!(restored.to_toml_string().unwrap(), before);
    assert_eq!(graph.to_toml_string().unwrap(), before);
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
