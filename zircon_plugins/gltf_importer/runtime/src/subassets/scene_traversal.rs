use std::collections::HashSet;

use zircon_runtime::asset::{
    AssetImportOutcome, AssetUri, ImportedAsset, ImportedAssetEntry, SceneAsset, SceneEntityAsset,
};

use super::{
    gltf_label_uri, material_uri_for_index, scene_entity_from_gltf_node,
    with_root_dependency_and_entry,
};

pub(crate) fn add_gltf_scene_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
) -> AssetImportOutcome {
    let dependency_index = GltfSceneDependencyIndex::build(root_uri, document);
    for node in &dependency_index.nodes {
        let uri = gltf_label_uri(root_uri, &format!("Node{}", node.index()));
        let mut entity = scene_entity_from_gltf_node(root_uri, node, None);
        entity.parent = None;
        let entry = scene_entry_with_direct_node_dependencies(
            root_uri,
            uri,
            SceneAsset {
                entities: vec![entity],
            },
            node.index(),
            &dependency_index,
        );
        outcome = with_root_dependency_and_entry(outcome, entry);
    }

    for scene in document.scenes() {
        let uri = gltf_label_uri(root_uri, &format!("Scene{}", scene.index()));
        let roots = scene.nodes().map(|node| node.index()).collect::<Vec<_>>();
        let mut entities = Vec::new();
        for &node_index in &roots {
            push_scene_node(root_uri, node_index, &dependency_index, None, &mut entities);
        }
        let entry = scene_entry_with_scene_dependencies(
            uri,
            SceneAsset { entities },
            roots,
            &dependency_index,
        );
        outcome = with_root_dependency_and_entry(outcome, entry);
    }
    outcome
}

struct GltfSceneDependencyIndex<'a> {
    // Keep one direct-edge record per node; scene closures are derived by walking
    // this adjacency once instead of rescanning every node's descendant subtree.
    nodes: Vec<gltf::Node<'a>>,
    children: Vec<Vec<usize>>,
    node_dependencies: Vec<Vec<AssetUri>>,
}

impl<'a> GltfSceneDependencyIndex<'a> {
    fn build(root_uri: &AssetUri, document: &'a gltf::Document) -> Self {
        let nodes = document.nodes().collect::<Vec<_>>();
        let mut children = Vec::with_capacity(nodes.len());
        let mut node_dependencies = Vec::with_capacity(nodes.len());
        for node in &nodes {
            let primitive_count = node
                .mesh()
                .map(|mesh| mesh.primitives().count())
                .unwrap_or_default();
            let dependency_capacity = 1usize.saturating_add(primitive_count.saturating_mul(3));
            let mut dependencies = Vec::with_capacity(dependency_capacity);
            let mut dependency_set = HashSet::with_capacity(dependency_capacity);
            append_node_direct_dependencies(root_uri, node, &mut dependencies, &mut dependency_set);
            children.push(node.children().map(|child| child.index()).collect());
            node_dependencies.push(dependencies);
        }
        Self {
            nodes,
            children,
            node_dependencies,
        }
    }

    fn node(&self, node_index: usize) -> &gltf::Node<'a> {
        self.nodes
            .get(node_index)
            .expect("validated glTF node index should be addressable")
    }

    fn children_for_node(&self, node_index: usize) -> &[usize] {
        self.children
            .get(node_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    fn for_node(&self, node_index: usize) -> &[AssetUri] {
        self.node_dependencies
            .get(node_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

fn scene_entry_with_direct_node_dependencies(
    root_uri: &AssetUri,
    uri: AssetUri,
    scene: SceneAsset,
    node_index: usize,
    dependency_index: &GltfSceneDependencyIndex<'_>,
) -> ImportedAssetEntry {
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Scene(scene));
    let child_indices = dependency_index.children_for_node(node_index);
    let mut known_dependencies = HashSet::with_capacity(
        dependency_index
            .for_node(node_index)
            .len()
            .saturating_add(child_indices.len()),
    );
    extend_scene_dependencies(
        &mut entry,
        &mut known_dependencies,
        dependency_index.for_node(node_index),
    );
    for &child_index in child_indices {
        push_scene_dependency_once(
            &mut entry.dependencies,
            &mut known_dependencies,
            gltf_label_uri(root_uri, &format!("Node{child_index}")),
        );
    }
    entry
}

fn scene_entry_with_scene_dependencies(
    uri: AssetUri,
    scene: SceneAsset,
    roots: impl IntoIterator<Item = usize>,
    dependency_index: &GltfSceneDependencyIndex<'_>,
) -> ImportedAssetEntry {
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Scene(scene));
    let mut pending = roots.into_iter().collect::<Vec<_>>();
    let mut known_dependencies = HashSet::with_capacity(pending.len().saturating_mul(4));
    let mut visited_nodes = HashSet::with_capacity(pending.len());
    pending.reverse();
    while let Some(node) = pending.pop() {
        if !visited_nodes.insert(node) {
            continue;
        }
        extend_scene_dependencies(
            &mut entry,
            &mut known_dependencies,
            dependency_index.for_node(node),
        );
        pending.extend(
            dependency_index
                .children_for_node(node)
                .iter()
                .rev()
                .copied(),
        );
    }
    entry
}

fn append_node_direct_dependencies(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
    dependencies: &mut Vec<AssetUri>,
    dependency_index: &mut HashSet<AssetUri>,
) {
    push_scene_dependency_once(
        dependencies,
        dependency_index,
        gltf_label_uri(root_uri, &format!("Node{}", node.index())),
    );
    if let Some(mesh) = node.mesh() {
        push_scene_dependency_once(
            dependencies,
            dependency_index,
            gltf_label_uri(root_uri, &format!("Mesh{}", mesh.index())),
        );
        for primitive in mesh.primitives() {
            push_scene_dependency_once(
                dependencies,
                dependency_index,
                gltf_label_uri(
                    root_uri,
                    &format!("Mesh{}/Primitive{}", mesh.index(), primitive.index()),
                ),
            );
            push_scene_dependency_once(
                dependencies,
                dependency_index,
                material_uri_for_index(root_uri, primitive.material().index()),
            );
        }
    }
}

fn push_scene_node(
    root_uri: &AssetUri,
    node_index: usize,
    dependency_index: &GltfSceneDependencyIndex<'_>,
    parent: Option<u64>,
    entities: &mut Vec<SceneEntityAsset>,
) {
    let mut pending = vec![(node_index, parent)];
    while let Some((node_index, parent)) = pending.pop() {
        let node = dependency_index.node(node_index);
        let entity_id = node_index as u64;
        entities.push(scene_entity_from_gltf_node(root_uri, node, parent));
        pending.extend(
            dependency_index
                .children_for_node(node_index)
                .iter()
                .rev()
                .copied()
                .map(|child_index| (child_index, Some(entity_id))),
        );
    }
}

fn extend_scene_dependencies(
    entry: &mut ImportedAssetEntry,
    dependency_index: &mut HashSet<AssetUri>,
    dependencies: &[AssetUri],
) {
    for dependency in dependencies {
        push_scene_dependency_once(
            &mut entry.dependencies,
            dependency_index,
            dependency.clone(),
        );
    }
}

fn push_scene_dependency_once(
    dependencies: &mut Vec<AssetUri>,
    dependency_index: &mut HashSet<AssetUri>,
    locator: AssetUri,
) {
    if dependency_index.insert(locator.clone()) {
        dependencies.push(locator);
    }
}
