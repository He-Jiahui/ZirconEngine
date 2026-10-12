use super::*;
use zircon_runtime::asset::{AssetImportOutcome, AssetUri, ImportedAsset, ModelAsset};

#[test]
fn scene_traversal_uses_iterative_stack_and_indexed_dependencies() {
    let source = include_str!("../scene_traversal.rs");
    let parent_source = include_str!("../../subassets.rs");
    assert!(parent_source.contains("mod scene_traversal;"));
    assert!(parent_source.contains("pub(crate) use scene_traversal::add_gltf_scene_subassets;"));
    let index_body = source
        .split("struct GltfSceneDependencyIndex")
        .nth(1)
        .expect("scene dependency index remains present");
    assert!(index_body.contains("nodes: Vec<gltf::Node"));
    assert!(index_body.contains("children: Vec<Vec<usize>>"));
    let dependency_body = source
        .split("fn scene_entry_with_scene_dependencies")
        .nth(1)
        .expect("scene dependency builder remains present")
        .split("fn push_scene_node")
        .next()
        .expect("scene dependency builder remains bounded");
    assert!(dependency_body.contains("HashSet"));
    assert!(dependency_body.contains("while let Some(node) = pending.pop()"));
    assert!(dependency_body.contains("visited_nodes"));
    assert!(
        !source.contains("fn push_node_dependencies"),
        "scene dependency traversal must not retain the recursive helper"
    );

    let entity_body = source
        .split("fn push_scene_node")
        .nth(1)
        .expect("scene entity builder remains present")
        .split("fn scene_entity_from_gltf_node")
        .next()
        .expect("scene entity builder remains bounded");
    assert!(entity_body.contains("while let Some"));
    assert!(
        !entity_body.contains("push_scene_node(root_uri, &child"),
        "scene entity traversal must not recurse through child nodes"
    );
}

#[test]
fn scene_traversal_preserves_depth_first_order_and_parent_links() {
    let gltf = gltf::Gltf::from_slice(
        br#"{
            "asset": { "version": "2.0" },
            "nodes": [
                { "name": "Root", "children": [1, 2] },
                { "name": "Left", "children": [3] },
                { "name": "Right" },
                { "name": "Leaf" }
            ],
            "scenes": [{ "name": "Main", "nodes": [0] }],
            "scene": 0
        }"#,
    )
    .expect("scene traversal fixture should parse");
    let root_uri = AssetUri::parse("res://models/scene_traversal.gltf").unwrap();
    let outcome = AssetImportOutcome::new(
        root_uri.clone(),
        ImportedAsset::Model(ModelAsset {
            uri: root_uri.clone(),
            primitives: Vec::new(),
        }),
    );
    let outcome = add_gltf_scene_subassets(outcome, &root_uri, &gltf.document);
    let scene_uri = gltf_label_uri(&root_uri, "Scene0");
    let scene_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == scene_uri)
        .expect("scene entry should be emitted");

    let expected_dependencies = ["Node0", "Node1", "Node3", "Node2"]
        .into_iter()
        .map(|label| gltf_label_uri(&root_uri, label))
        .collect::<Vec<_>>();
    assert_eq!(scene_entry.dependencies, expected_dependencies);

    let node0_uri = gltf_label_uri(&root_uri, "Node0");
    let node0_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == node0_uri)
        .expect("node entry should be emitted");
    assert_eq!(
        node0_entry.dependencies,
        vec![
            gltf_label_uri(&root_uri, "Node0"),
            gltf_label_uri(&root_uri, "Node1"),
            gltf_label_uri(&root_uri, "Node2"),
        ]
    );

    let ImportedAsset::Scene(scene) = &scene_entry.asset else {
        panic!("scene entry should contain a SceneAsset");
    };
    assert_eq!(
        scene
            .entities
            .iter()
            .map(|entity| entity.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Root", "Left", "Leaf", "Right"]
    );
    assert_eq!(
        scene
            .entities
            .iter()
            .map(|entity| entity.parent)
            .collect::<Vec<_>>(),
        vec![None, Some(0), Some(1), Some(0)]
    );
}
