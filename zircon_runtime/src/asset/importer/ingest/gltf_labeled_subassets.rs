use std::collections::HashSet;

use crate::asset::{
    AssetImportOutcome, AssetReference, AssetUri, ImportedAsset, ImportedAssetEntry, MeshAsset,
    ModelAsset, ModelPrimitiveAsset, SceneAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TransformAsset,
};

mod material;
#[cfg(test)]
mod texture_variant_tests;

pub(crate) use self::material::add_gltf_material_subassets;

pub(crate) struct GltfMeshSubasset {
    pub(crate) mesh_index: usize,
    pub(crate) primitives: Vec<GltfPrimitiveSubasset>,
}

pub(crate) struct GltfPrimitiveSubasset {
    pub(crate) primitive_index: usize,
    pub(crate) material_index: Option<usize>,
    pub(crate) mesh: MeshAsset,
}

pub(crate) fn add_gltf_mesh_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    meshes: Vec<GltfMeshSubasset>,
) -> AssetImportOutcome {
    for mesh in meshes {
        let mesh_uri = gltf_indexed_label_uri(root_uri, "Mesh", mesh.mesh_index);
        let mesh_model = ModelAsset {
            uri: mesh_uri.clone(),
            primitives: mesh
                .primitives
                .iter()
                .map(|primitive| ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    mesh: Some(AssetReference::from_locator(primitive.mesh.uri.clone())),
                    mesh_sdf: None,
                    virtual_geometry: None,
                })
                .collect(),
        };
        let mut mesh_entry =
            ImportedAssetEntry::new(mesh_uri.clone(), ImportedAsset::Model(mesh_model));
        let mut dependency_index = HashSet::new();
        for primitive in &mesh.primitives {
            push_dependency_once(
                &mut mesh_entry,
                &mut dependency_index,
                gltf_mesh_primitive_uri(root_uri, mesh.mesh_index, primitive.primitive_index),
            );
            push_dependency_once(
                &mut mesh_entry,
                &mut dependency_index,
                material_uri_for_index(root_uri, primitive.material_index),
            );
        }
        outcome = with_root_dependency_and_entry(outcome, mesh_entry);

        for primitive in mesh.primitives {
            let entry = ImportedAssetEntry::new(
                primitive.mesh.uri.clone(),
                ImportedAsset::Mesh(primitive.mesh),
            )
            .with_dependency(material_uri_for_index(root_uri, primitive.material_index));
            outcome = with_root_dependency_and_entry(outcome, entry);
        }
    }
    outcome
}

pub(crate) fn add_gltf_scene_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
) -> AssetImportOutcome {
    for node in document.nodes() {
        let uri = gltf_indexed_label_uri(root_uri, "Node", node.index());
        let mut entity = scene_entity_from_gltf_node(root_uri, &node, None);
        entity.parent = None;
        let entry = scene_entry_with_node_dependencies(
            root_uri,
            uri,
            SceneAsset {
                entities: vec![entity],
            },
            std::iter::once(node),
        );
        outcome = with_root_dependency_and_entry(outcome, entry);
    }

    for scene in document.scenes() {
        let uri = gltf_indexed_label_uri(root_uri, "Scene", scene.index());
        let mut entities = Vec::new();
        for node in scene.nodes() {
            push_scene_node(root_uri, &node, None, &mut entities);
        }
        let entry = scene_entry_with_node_dependencies(
            root_uri,
            uri,
            SceneAsset { entities },
            scene.nodes(),
        );
        outcome = with_root_dependency_and_entry(outcome, entry);
    }
    outcome
}

fn scene_entry_with_node_dependencies<'a>(
    root_uri: &AssetUri,
    uri: AssetUri,
    scene: SceneAsset,
    roots: impl IntoIterator<Item = gltf::Node<'a>>,
) -> ImportedAssetEntry {
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Scene(scene));
    let mut dependency_index = HashSet::new();
    for node in roots {
        push_node_dependencies(root_uri, &node, &mut entry, &mut dependency_index);
    }
    entry
}

fn push_node_dependencies(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
    entry: &mut ImportedAssetEntry,
    dependency_index: &mut HashSet<AssetUri>,
) {
    push_dependency_once(
        entry,
        dependency_index,
        gltf_indexed_label_uri(root_uri, "Node", node.index()),
    );
    if let Some(mesh) = node.mesh() {
        push_dependency_once(
            entry,
            dependency_index,
            gltf_indexed_label_uri(root_uri, "Mesh", mesh.index()),
        );
        for primitive in mesh.primitives() {
            push_dependency_once(
                entry,
                dependency_index,
                gltf_mesh_primitive_uri(root_uri, mesh.index(), primitive.index()),
            );
            push_dependency_once(
                entry,
                dependency_index,
                material_uri_for_index(root_uri, primitive.material().index()),
            );
        }
    }
    for child in node.children() {
        push_node_dependencies(root_uri, &child, entry, dependency_index);
    }
}

fn push_scene_node(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
    parent: Option<u64>,
    entities: &mut Vec<SceneEntityAsset>,
) {
    let entity_id = node.index() as u64;
    entities.push(scene_entity_from_gltf_node(root_uri, node, parent));
    for child in node.children() {
        push_scene_node(root_uri, &child, Some(entity_id), entities);
    }
}

fn scene_entity_from_gltf_node(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
    parent: Option<u64>,
) -> SceneEntityAsset {
    SceneEntityAsset {
        entity: node.index() as u64,
        name: node
            .name()
            .map(str::to_owned)
            .unwrap_or_else(|| format!("Node{}", node.index())),
        parent,
        transform: transform_from_gltf_node(node),
        active: true,
        render_layer_mask: 0x0000_0001,
        mobility: SceneMobilityAsset::Dynamic,
        camera: None,
        mesh: mesh_instance_from_gltf_node(root_uri, node),
        ambient_light: None,
        directional_light: None,
        point_light: None,
        rect_light: None,
        spot_light: None,
        post_process_volume: None,
        rigid_body: None,
        collider: None,
        joint: None,
        animation_skeleton: None,
        animation_player: None,
        animation_sequence_player: None,
        animation_graph_player: None,
        animation_state_machine_player: None,
        terrain: None,
        tilemap: None,
        prefab_instance: None,
        script_bindings: Vec::new(),
    }
}

fn mesh_instance_from_gltf_node(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
) -> Option<SceneMeshInstanceAsset> {
    let mesh = node.mesh()?;
    Some(SceneMeshInstanceAsset {
        model: gltf_indexed_label_reference(root_uri, "Mesh", mesh.index()),
        mesh: None,
        material: material_reference_for_index(root_uri, first_mesh_material_index(&mesh)),
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        morph_weights: mesh
            .weights()
            .map(|weights| weights.to_vec())
            .unwrap_or_default(),
        primitives: mesh_primitive_bindings_from_gltf_mesh(root_uri, &mesh),
        lods: Vec::new(),
    })
}

fn first_mesh_material_index(mesh: &gltf::Mesh<'_>) -> Option<usize> {
    mesh.primitives()
        .next()
        .and_then(|primitive| primitive.material().index())
}

fn mesh_primitive_bindings_from_gltf_mesh(
    root_uri: &AssetUri,
    mesh: &gltf::Mesh<'_>,
) -> Vec<SceneMeshPrimitiveBindingAsset> {
    mesh.primitives()
        .map(|primitive| SceneMeshPrimitiveBindingAsset {
            mesh: AssetReference::from_locator(gltf_mesh_primitive_uri(
                root_uri,
                mesh.index(),
                primitive.index(),
            )),
            material: material_reference_for_index(root_uri, primitive.material().index()),
        })
        .collect()
}

fn transform_from_gltf_node(node: &gltf::Node<'_>) -> TransformAsset {
    let (translation, rotation, scale) = node.transform().decomposed();
    TransformAsset {
        translation,
        rotation,
        scale,
    }
}

fn with_root_dependency_and_entry(
    outcome: AssetImportOutcome,
    entry: ImportedAssetEntry,
) -> AssetImportOutcome {
    outcome
        .with_dependency(entry.locator.clone())
        .with_entry(entry)
}

fn push_dependency_once(
    entry: &mut ImportedAssetEntry,
    dependency_index: &mut HashSet<AssetUri>,
    locator: AssetUri,
) {
    if dependency_index.insert(locator.clone()) {
        entry.dependencies.push(locator);
    }
}

fn material_reference_for_index(
    root_uri: &AssetUri,
    material_index: Option<usize>,
) -> AssetReference {
    AssetReference::from_locator(material_uri_for_index(root_uri, material_index))
}

fn material_uri_for_index(root_uri: &AssetUri, material_index: Option<usize>) -> AssetUri {
    match material_index {
        Some(index) => gltf_indexed_label_uri(root_uri, "Material", index),
        None => gltf_label_uri(root_uri, "DefaultMaterial"),
    }
}

pub(crate) fn gltf_label_reference(root_uri: &AssetUri, label: &str) -> AssetReference {
    AssetReference::from_locator(gltf_label_uri(root_uri, label))
}

fn gltf_indexed_label_reference(root_uri: &AssetUri, label: &str, index: usize) -> AssetReference {
    AssetReference::from_locator(gltf_indexed_label_uri(root_uri, label, index))
}

fn gltf_indexed_label_uri(root_uri: &AssetUri, label: &str, index: usize) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}{index}"))
        .expect("generated indexed gltf subasset locator must be valid")
}

fn gltf_mesh_primitive_uri(
    root_uri: &AssetUri,
    mesh_index: usize,
    primitive_index: usize,
) -> AssetUri {
    AssetUri::parse(&format!(
        "{root_uri}#Mesh{mesh_index}/Primitive{primitive_index}"
    ))
    .expect("generated gltf mesh primitive locator must be valid")
}

pub(crate) fn gltf_label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}"))
        .expect("generated gltf subasset locator must be valid")
}

#[cfg(test)]
mod plugins07_label_uri_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 21;
    const LABELS_PER_SAMPLE: usize = 8_192;

    #[test]
    fn registry_label_hotpath_contract_single_buffer_gltf_uris() {
        let root = AssetUri::parse("res://models/plugins07.glb").unwrap();

        assert_eq!(
            gltf_indexed_label_uri(&root, "Node", 42),
            gltf_label_uri(&root, "Node42")
        );
        assert_eq!(
            gltf_mesh_primitive_uri(&root, 7, 11),
            gltf_label_uri(&root, "Mesh7/Primitive11")
        );
        assert_eq!(
            gltf_indexed_label_reference(&root, "Material", 9).locator,
            gltf_label_uri(&root, "Material9")
        );
    }

    #[test]
    #[ignore = "release performance gate"]
    fn registry_label_hotpath_performance_release_single_buffer_gltf_uris() {
        let root = AssetUri::parse("res://models/plugins07-benchmark.glb").unwrap();
        for _ in 0..4 {
            black_box(measure_legacy(&root));
            black_box(measure_single_buffer(&root));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for pair_index in 0..SAMPLE_PAIRS {
            let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
                (measure_legacy(&root), measure_single_buffer(&root))
            } else {
                let optimized_ns = measure_single_buffer(&root);
                (measure_legacy(&root), optimized_ns)
            };
            legacy_samples.push(legacy_ns);
            optimized_samples.push(optimized_ns);
        }

        let legacy_p95 = nearest_rank_p95(&legacy_samples);
        let optimized_p95 = nearest_rank_p95(&optimized_samples);
        let improvement_percent =
            legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
        println!(
            "PERF_RESULT plugins07_single_buffer_gltf_label_uris sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20 legacy_uri_input_allocations_per_sample={} optimized_uri_input_allocations_per_sample={LABELS_PER_SAMPLE} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
            csv(&legacy_samples),
            csv(&optimized_samples),
            LABELS_PER_SAMPLE * 2,
        );
        assert!(
            improvement_percent >= 20,
            "single-buffer glTF label URIs must improve P95 by at least 20%"
        );
    }

    fn measure_legacy(root: &AssetUri) -> u128 {
        let started = Instant::now();
        for index in 0..LABELS_PER_SAMPLE {
            let uri = if index % 2 == 0 {
                gltf_label_uri(black_box(root), &format!("Node{}", black_box(index)))
            } else {
                gltf_label_uri(
                    black_box(root),
                    &format!("Mesh{}/Primitive{}", black_box(index), black_box(index + 1)),
                )
            };
            black_box(uri);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn measure_single_buffer(root: &AssetUri) -> u128 {
        let started = Instant::now();
        for index in 0..LABELS_PER_SAMPLE {
            let uri = if index % 2 == 0 {
                gltf_indexed_label_uri(black_box(root), "Node", black_box(index))
            } else {
                gltf_mesh_primitive_uri(black_box(root), black_box(index), black_box(index + 1))
            };
            black_box(uri);
        }
        started.elapsed().as_nanos().max(1)
    }

    fn nearest_rank_p95(samples: &[u128]) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        let rank = (sorted.len() * 95).div_ceil(100);
        sorted[rank.saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
