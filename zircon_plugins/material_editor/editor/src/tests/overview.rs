use std::collections::BTreeMap;

use super::{compile_material_graph, MaterialGraphPresentation};
use zircon_runtime::asset::{
    AssetReference, AssetUri, MaterialGraphAsset, MaterialGraphLinkAsset, MaterialGraphNodeAsset,
    MaterialGraphNodeKindAsset as Node,
};

#[test]
fn material_graph_texture_overview_empty_source_has_no_placeholder_or_resolver_call() {
    let mut graph = graph();
    graph.shader = Some(reference("res://shaders/pbr.wgsl"));
    graph.nodes.push(node("output", Node::Output));
    graph.nodes.push(node(
        "parameter",
        Node::ScalarParameter {
            name: "Roughness".into(),
            default: 0.5,
        },
    ));
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    assert!(presentation
        .texture_overview(
            |_| panic!("non-texture references are not usage rows"),
            |_, _| panic!("empty rows must not invoke name collation"),
        )
        .is_empty());
}

#[test]
fn material_graph_texture_overview_counts_disconnected_expressions_and_sorts_ue_rows() {
    let mut graph = graph();
    let first = reference("res://textures/first.texture.toml");
    let second = reference("res://textures/second.texture.toml");
    let repeated = reference("res://textures/repeated.texture.toml");
    graph.nodes = vec![
        sample("repeat-a", &repeated),
        sample("first", &first),
        sample("repeat-b", &repeated),
        node("output", Node::Output),
        sample("second", &second),
    ];
    let before = graph.clone();
    let mut calls = 0;
    let rows = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .texture_overview(
            |texture| {
                calls += 1;
                if texture == &first { "Z" } else { "A" }.into()
            },
            names,
        );
    assert_eq!(calls, 3);
    assert_eq!(
        rows.iter()
            .map(|row| (row.display_name(), row.reference_count()))
            .collect::<Vec<_>>(),
        vec![("A", 1), ("Z", 1), ("A", 2)]
    );
    assert!(std::ptr::eq(rows[0].details_node(), &graph.nodes[4]));
    assert!(std::ptr::eq(rows[2].nodes()[0], &graph.nodes[0]));
    assert!(std::ptr::eq(rows[2].nodes()[1], &graph.nodes[2]));
    let Node::TextureSample { texture } = &graph.nodes[0].kind else {
        unreachable!()
    };
    assert!(std::ptr::eq(rows[2].texture(), texture));
    assert!(graph.links.is_empty());
    assert_eq!(graph, before);
}

#[test]
fn material_graph_texture_overview_keeps_full_resource_identity_and_label_collisions() {
    let a = reference("res://textures/rock.texture.toml");
    let different_uuid = AssetReference::new(
        reference("res://textures/other.texture.toml").uuid,
        a.locator.clone(),
    );
    let moved = AssetReference::new(
        a.uuid,
        AssetUri::parse("res://moved/rock.texture.toml").unwrap(),
    );
    let subasset = AssetReference::new(
        a.uuid,
        AssetUri::parse("res://textures/rock.texture.toml#preview").unwrap(),
    );
    let mut graph = graph();
    graph.nodes = [&a, &different_uuid, &moved, &subasset]
        .into_iter()
        .enumerate()
        .map(|(index, texture)| sample(&format!("sample-{index}"), texture))
        .collect();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let rows = presentation.texture_overview(|_| "岩石_相同显示名_很长的资源名称".into(), names);
    assert_eq!(rows.len(), 4);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row.reference_count(), 1);
        assert!(std::ptr::eq(row.details_node(), &graph.nodes[index]));
    }
    assert_ne!(rows[0].texture(), rows[1].texture());
    assert_ne!(rows[0].texture(), rows[2].texture());
    assert_ne!(rows[0].texture(), rows[3].texture());
}

#[test]
fn material_graph_texture_overview_details_tracks_actual_last_source_occurrence() {
    let texture = reference("res://textures/normal.texture.toml");
    let mut graph = graph();
    graph.nodes = vec![sample("first", &texture), sample("last", &texture)];
    for expected in ["last", "first"] {
        let rows = MaterialGraphPresentation::new(&graph)
            .unwrap()
            .texture_overview(|_| "Normal".into(), names);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].details_node().id, expected);
        assert!(std::ptr::eq(
            rows[0].details_node(),
            graph.nodes.last().unwrap()
        ));
        drop(rows);
        graph.nodes.reverse();
    }
}

#[test]
fn material_graph_texture_overview_keeps_invalid_graph_visible_without_hiding_diagnostics() {
    let mut graph = graph();
    graph.nodes = vec![
        sample("texture", &reference("res://textures/base.texture.toml")),
        node("output", Node::Output),
    ];
    graph.links.push(MaterialGraphLinkAsset {
        from_node: "texture".into(),
        from_pin: "color".into(),
        to_node: "output".into(),
        to_pin: "roughness".into(),
    });
    let before = graph.clone();
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let diagnostic = presentation.type_diagnostic().unwrap().clone();
    let rows = presentation.texture_overview(|_| "Base".into(), names);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].details_node().id, "texture");
    assert_eq!(presentation.type_diagnostic(), Some(&diagnostic));
    assert_eq!(graph, before);
}

#[test]
fn material_graph_texture_overview_roundtrips_source_and_actual_pbr_texture_compile() {
    let texture = reference("res://textures/岩石_颜色.texture.toml");
    let mut graph = graph();
    graph.shader = Some(reference("res://shaders/pbr.wgsl"));
    graph.nodes = vec![
        sample("base", &texture),
        sample("unconnected", &texture),
        node("output", Node::Output),
    ];
    graph.links.push(MaterialGraphLinkAsset {
        from_node: "base".into(),
        from_pin: "color".into(),
        to_node: "output".into(),
        to_pin: "base_color".into(),
    });
    let saved = graph.to_toml_string().unwrap();
    let reopened = MaterialGraphAsset::from_toml_str(&saved).unwrap();
    assert_eq!(reopened, graph);
    let rows = MaterialGraphPresentation::new(&reopened)
        .unwrap()
        .texture_overview(|_| "岩石颜色".into(), names);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].texture(), &texture);
    assert_eq!(rows[0].reference_count(), 2);
    assert_eq!(rows[0].details_node().id, "unconnected");
    let material = compile_material_graph(&reopened).unwrap();
    assert_eq!(material.base_color_texture, Some(texture));
    assert_eq!(material.base_color, [1.0; 4]);
}

#[test]
fn material_graph_texture_overview_display_rename_only_reorders_distinct_sources() {
    let a = reference("res://textures/a.texture.toml");
    let b = reference("res://textures/b.texture.toml");
    let mut graph = graph();
    graph.nodes = vec![sample("a", &a), sample("b", &b)];
    let presentation = MaterialGraphPresentation::new(&graph).unwrap();
    let original = presentation.texture_overview(
        |texture| if texture == &a { "A" } else { "B" }.into(),
        names,
    );
    let renamed =
        presentation.texture_overview(|texture| if texture == &a { "Z" } else { "" }.into(), names);
    assert_eq!(original[0].details_node().id, "a");
    assert_eq!(renamed[0].details_node().id, "b");
    assert_eq!(renamed[0].display_name(), "");
    assert_eq!(original[0].texture(), renamed[1].texture());
    assert_eq!(original[1].texture(), renamed[0].texture());
}

#[test]
fn material_graph_texture_overview_uses_host_collation_and_stable_equal_name_order() {
    let a = reference("res://textures/a.texture.toml");
    let b = reference("res://textures/b.texture.toml");
    let same = reference("res://textures/another-a.texture.toml");
    let mut graph = graph();
    graph.nodes = vec![sample("b", &b), sample("a", &a), sample("same", &same)];
    let rows = MaterialGraphPresentation::new(&graph)
        .unwrap()
        .texture_overview(
            |texture| {
                if texture == &b {
                    "B"
                } else if texture == &a {
                    "a"
                } else {
                    "A"
                }
                .into()
            },
            names,
        );
    assert_eq!(
        rows.iter()
            .map(|row| row.details_node().id.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "same", "b"]
    );
}

// An injected test host for the ASCII case-folding contract. Production Unicode
// collation remains the real host's responsibility; this is not an engine port.
fn names(left: &str, right: &str) -> std::cmp::Ordering {
    left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase())
}

fn graph() -> MaterialGraphAsset {
    MaterialGraphAsset {
        uri: AssetUri::parse("res://materials/texture_overview.material_graph.toml").unwrap(),
        name: "Texture Overview / 纹理用量".into(),
        shader: None,
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

fn sample(id: &str, texture: &AssetReference) -> MaterialGraphNodeAsset {
    node(
        id,
        Node::TextureSample {
            texture: texture.clone(),
        },
    )
}

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}
