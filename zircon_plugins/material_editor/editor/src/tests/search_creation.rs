use std::collections::BTreeMap;

use super::{
    compile_material_graph, plan_material_graph_node_search_creation,
    MaterialGraphConnectionAction, MaterialGraphDiagnosticTarget, MaterialGraphNodeSearch,
    MaterialGraphValueType,
};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset as Link,
    MaterialGraphNodeAsset, MaterialGraphNodeKindAsset as Node,
};
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn material_graph_search_creation_borrows_the_visible_candidate_without_consuming_menu() {
    let graph = graph();
    let graph_before = graph.clone();
    let menu = MaterialGraphNodeSearch::new("math");
    let menu_before = menu.clone();
    let candidate = node("sum", Node::Add);
    let plan =
        plan_material_graph_node_search_creation(&menu, &graph, "add", &candidate, None, |_| {
            panic!("math actions do not resolve resources")
        })
        .unwrap();
    assert!(std::ptr::eq(plan.node(), &candidate));
    assert!(plan.link().is_none());
    assert!(plan.connection().is_none());
    assert_eq!(menu, menu_before);
    assert_eq!(graph, graph_before);
}

#[test]
fn material_graph_search_creation_clicked_key_does_not_alias_keyboard_selection() {
    let graph = graph();
    let menu = MaterialGraphNodeSearch::new("math");
    assert_eq!(menu.selected().unwrap().id(), "add");
    let candidate = node("product", Node::Multiply);
    let plan = plan_material_graph_node_search_creation(
        &menu,
        &graph,
        "multiply",
        &candidate,
        None,
        |_| panic!("math actions do not resolve resources"),
    )
    .unwrap();
    assert!(std::ptr::eq(plan.node(), &candidate));
    assert_eq!(menu.selected().unwrap().id(), "add");
    assert!(menu.is_open());
}

#[test]
fn material_graph_search_creation_hidden_and_unknown_keys_reject_before_resolver() {
    let graph = graph();
    let mut menu = MaterialGraphNodeSearch::default();
    assert!(menu
        .matches()
        .iter()
        .any(|action| action.id() == "texture_sample"));
    menu.set_query("math");
    let before = menu.clone();
    let candidate = node(
        "texture",
        Node::TextureSample {
            texture: reference("res://textures/rock.png"),
        },
    );
    for key in ["texture_sample", "Multiply display caption", "missing"] {
        let errors =
            plan_material_graph_node_search_creation(&menu, &graph, key, &candidate, None, |_| {
                panic!("a stale or unknown row must not resolve resources")
            })
            .unwrap_err();
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert!(errors[0].message.contains("not visible"));
        assert_eq!(menu, before);
    }
}

#[test]
fn material_graph_search_creation_closed_menu_rejects_even_a_retained_visible_row() {
    let graph = graph();
    let mut menu = MaterialGraphNodeSearch::new("texture");
    menu.cancel();
    assert!(menu
        .matches()
        .iter()
        .any(|action| action.id() == "texture_sample"));
    let before = menu.clone();
    let candidate = node(
        "texture",
        Node::TextureSample {
            texture: reference("res://textures/rock.png"),
        },
    );
    let errors = plan_material_graph_node_search_creation(
        &menu,
        &graph,
        "texture_sample",
        &candidate,
        None,
        |_| panic!("a closed menu must not resolve resources"),
    )
    .unwrap_err();
    assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
    assert!(errors[0].message.contains("closed"));
    assert_eq!(menu, before);
}

#[test]
fn material_graph_search_creation_wrong_kind_and_invalid_wire_leave_selection_alone() {
    let graph = graph();
    let menu = MaterialGraphNodeSearch::new("math");
    let before = menu.clone();
    let candidate = node("product", Node::Multiply);
    let invalid_wire = link("product", "value", "missing", "a");
    for (key, wire) in [("add", None), ("multiply", Some(&invalid_wire))] {
        let errors =
            plan_material_graph_node_search_creation(&menu, &graph, key, &candidate, wire, |_| {
                panic!("invalid math proposals must not resolve resources")
            })
            .unwrap_err();
        assert_eq!(errors[0].target, MaterialGraphDiagnosticTarget::Graph);
        assert_eq!(menu, before);
    }
}

#[test]
fn material_graph_search_creation_texture_uses_actual_resolver_without_closing_on_error() {
    let graph = graph();
    let graph_before = graph.clone();
    let menu = MaterialGraphNodeSearch::new("texture");
    let menu_before = menu.clone();
    let texture = reference("res://textures/岩石法线.png");
    let candidate = node(
        "normal",
        Node::TextureSample {
            texture: texture.clone(),
        },
    );
    let wire = link("normal", "color", "output", "normal");
    let mut calls = 0;
    let plan = plan_material_graph_node_search_creation(
        &menu,
        &graph,
        "texture_sample",
        &candidate,
        Some(&wire),
        |actual| {
            calls += 1;
            assert_eq!(actual, &texture);
            Ok(ResourceKind::Texture)
        },
    )
    .unwrap();
    assert_eq!(calls, 1);
    assert_eq!(
        plan.connection().unwrap().source_type,
        MaterialGraphValueType::Texture
    );
    for result in [
        Err("no current catalog entry".into()),
        Ok(ResourceKind::Material),
    ] {
        assert!(plan_material_graph_node_search_creation(
            &menu,
            &graph,
            "texture_sample",
            &candidate,
            Some(&wire),
            |_| result.clone(),
        )
        .is_err());
        assert_eq!(menu, menu_before);
        assert_eq!(graph, graph_before);
    }
}

#[test]
fn material_graph_search_creation_replacement_keeps_old_edge_and_real_compiler_semantics() {
    let mut graph = graph();
    graph
        .links
        .push(link("gain", "value", "output", "roughness"));
    let graph_before = graph.clone();
    let menu = MaterialGraphNodeSearch::new("scalar parameter");
    let menu_before = menu.clone();
    let candidate = MaterialGraphNodeAsset {
        id: "粗糙度_参数_1".into(),
        position: [-16.25, 128.5],
        kind: Node::ScalarParameter {
            name: "岩石粗糙度".into(),
            default: 0.6,
        },
    };
    let wire = link(&candidate.id, "value", "output", "roughness");
    let plan = plan_material_graph_node_search_creation(
        &menu,
        &graph,
        "scalar_parameter",
        &candidate,
        Some(&wire),
        |_| panic!("scalar actions do not resolve resources"),
    )
    .unwrap();
    let MaterialGraphConnectionAction::Replace(old) = plan.connection().unwrap().action else {
        panic!("the exact occupied edge must be retained");
    };
    assert!(std::ptr::eq(old, &graph.links[1]));
    assert!(std::ptr::eq(plan.node(), &candidate));
    assert!(std::ptr::eq(plan.link().unwrap(), &wire));

    // Unit-only DTO projection exercises the real codec/compiler. Production
    // application and inverse belong to the host's one prepared history delta.
    let mut projected = graph.clone();
    projected.nodes.push(plan.node().clone());
    projected.links[1] = plan.link().unwrap().clone();
    let serialized = projected.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&serialized).unwrap();
    assert_eq!(reopened, projected);
    assert_eq!(compile_material_graph(&reopened).unwrap().roughness, 0.6);
    assert_eq!(compile_material_graph(&graph).unwrap().roughness, 0.25);
    assert_eq!(graph, graph_before);
    assert_eq!(menu, menu_before);
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/search_creation.material_graph.toml").unwrap(),
        name: "Search creation / 搜索创建".into(),
        shader: Some(reference("res://shaders/pbr.wgsl")),
        nodes: vec![
            node(
                "base",
                Node::VectorParameter {
                    name: "Base Color".into(),
                    default: [0.2, 0.4, 0.8, 1.0],
                },
            ),
            node(
                "gain",
                Node::ScalarParameter {
                    name: "Gain".into(),
                    default: 0.25,
                },
            ),
            node("output", Node::Output),
        ],
        links: vec![link("base", "value", "output", "base_color")],
        parameters: BTreeMap::new(),
    }
}

fn node(id: &str, kind: Node) -> MaterialGraphNodeAsset {
    MaterialGraphNodeAsset {
        id: id.into(),
        position: [0.0; 2],
        kind,
    }
}

fn link(from: &str, from_pin: &str, to: &str, to_pin: &str) -> Link {
    Link {
        from_node: from.into(),
        from_pin: from_pin.into(),
        to_node: to.into(),
        to_pin: to_pin.into(),
    }
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
