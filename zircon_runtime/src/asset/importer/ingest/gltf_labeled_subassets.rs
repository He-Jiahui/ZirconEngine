use std::collections::HashSet;

use crate::asset::{
    AssetImportOutcome, AssetReference, AssetUri, ImportedAsset, ImportedAssetEntry, MeshAsset,
    ModelAsset, ModelPrimitiveAsset, SceneAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TransformAsset,
};

mod material;
#[cfg(test)]
#[path = "gltf_labeled_subassets/tests/optimization_batch_is_runtime629_tests.rs"]
mod optimization_batch_is_runtime629_tests;
#[cfg(test)]
#[path = "gltf_labeled_subassets/tests/texture_variant_tests.rs"]
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
        let mut dependency_index = HashSet::with_capacity(mesh.primitives.len().saturating_mul(2));
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
        components: Vec::new(),
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
#[path = "tests/gltf_labeled_subassets_plugins07_label_uri_tests.rs"]
mod plugins07_label_uri_tests;
