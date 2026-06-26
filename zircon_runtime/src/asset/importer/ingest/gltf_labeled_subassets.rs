use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};

use crate::asset::{
    AssetImportError, AssetImportOutcome, AssetReference, AssetUri, ImportedAsset,
    ImportedAssetEntry, MeshAsset, MeshMorphTargetAsset, MeshSkinAsset, ModelAsset,
    ModelPrimitiveAsset, SceneAsset, SceneEntityAsset, SceneMeshInstanceAsset,
    SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TextureAsset, TransformAsset,
};

mod material;

pub(crate) use self::material::add_gltf_material_subassets;

#[derive(Clone)]
pub(crate) struct GltfMeshSubasset {
    pub(crate) mesh_index: usize,
    pub(crate) skin: Option<MeshSkinAsset>,
    pub(crate) primitives: Vec<GltfPrimitiveSubasset>,
}

#[derive(Clone)]
pub(crate) struct GltfPrimitiveSubasset {
    pub(crate) primitive_index: usize,
    pub(crate) material_index: Option<usize>,
    pub(crate) morph_targets: Vec<MeshMorphTargetAsset>,
    pub(crate) primitive: ModelPrimitiveAsset,
}

pub(crate) fn add_gltf_texture_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
    images: &[GltfImageData],
) -> Result<AssetImportOutcome, AssetImportError> {
    for texture in document.textures() {
        let uri = gltf_label_uri(root_uri, &format!("Texture{}", texture.index()));
        let image_index = texture.source().index();
        let image = images.get(image_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf texture {} references missing image {}",
                texture.index(),
                image_index
            ))
        })?;
        let rgba = rgba8_pixels_from_gltf_image(image, image_index)?;
        let asset = TextureAsset::new_rgba8(uri.clone(), image.width, image.height, rgba);
        outcome = with_root_dependency_and_entry(
            outcome,
            ImportedAssetEntry::new(uri, ImportedAsset::Texture(asset)),
        );
    }
    Ok(outcome)
}

pub(crate) fn add_gltf_mesh_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    meshes: &[GltfMeshSubasset],
) -> AssetImportOutcome {
    for mesh in meshes {
        let mesh_uri = gltf_label_uri(root_uri, &format!("Mesh{}", mesh.mesh_index));
        let mesh_model = ModelAsset {
            uri: mesh_uri.clone(),
            primitives: mesh
                .primitives
                .iter()
                .map(|primitive| primitive.primitive.clone())
                .collect(),
        };
        let mut mesh_entry =
            ImportedAssetEntry::new(mesh_uri.clone(), ImportedAsset::Model(mesh_model));
        for primitive in &mesh.primitives {
            mesh_entry = mesh_entry.with_dependency(gltf_label_uri(
                root_uri,
                &format!(
                    "Mesh{}/Primitive{}",
                    mesh.mesh_index, primitive.primitive_index
                ),
            ));
            mesh_entry = mesh_entry
                .with_dependency(material_uri_for_index(root_uri, primitive.material_index));
        }
        outcome = with_root_dependency_and_entry(outcome, mesh_entry);

        for primitive in &mesh.primitives {
            let primitive_uri = gltf_label_uri(
                root_uri,
                &format!(
                    "Mesh{}/Primitive{}",
                    mesh.mesh_index, primitive.primitive_index
                ),
            );
            let mut mesh_asset =
                MeshAsset::from_model_primitive(primitive_uri.clone(), &primitive.primitive);
            mesh_asset.morph_targets = primitive.morph_targets.clone();
            mesh_asset.skin = mesh.skin.clone();
            let entry = ImportedAssetEntry::new(primitive_uri, ImportedAsset::Mesh(mesh_asset))
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
        let uri = gltf_label_uri(root_uri, &format!("Node{}", node.index()));
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
        let uri = gltf_label_uri(root_uri, &format!("Scene{}", scene.index()));
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

fn rgba8_pixels_from_gltf_image(
    image: &GltfImageData,
    image_index: usize,
) -> Result<Vec<u8>, AssetImportError> {
    let pixel_count = image
        .width
        .checked_mul(image.height)
        .and_then(|pixels| usize::try_from(pixels).ok())
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf image {image_index} extent {}x{} is too large",
                image.width, image.height
            ))
        })?;

    let mut rgba = Vec::with_capacity(pixel_count * 4);
    match image.format {
        GltfImageFormat::R8 => {
            validate_image_len(image, image_index, pixel_count)?;
            for value in &image.pixels {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        GltfImageFormat::R8G8 => {
            validate_image_len(image, image_index, pixel_count * 2)?;
            for chunk in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        GltfImageFormat::R8G8B8 => {
            validate_image_len(image, image_index, pixel_count * 3)?;
            for chunk in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        GltfImageFormat::R8G8B8A8 => {
            validate_image_len(image, image_index, pixel_count * 4)?;
            rgba = image.pixels.clone();
        }
        other => {
            return Err(AssetImportError::Parse(format!(
                "gltf image {image_index} format {other:?} is not supported for TextureAsset rgba8 output"
            )));
        }
    }
    Ok(rgba)
}

fn validate_image_len(
    image: &GltfImageData,
    image_index: usize,
    expected: usize,
) -> Result<(), AssetImportError> {
    if image.pixels.len() != expected {
        return Err(AssetImportError::Parse(format!(
            "gltf image {image_index} expected {expected} decoded bytes but found {}",
            image.pixels.len()
        )));
    }
    Ok(())
}

fn scene_entry_with_node_dependencies<'a>(
    root_uri: &AssetUri,
    uri: AssetUri,
    scene: SceneAsset,
    roots: impl IntoIterator<Item = gltf::Node<'a>>,
) -> ImportedAssetEntry {
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Scene(scene));
    for node in roots {
        push_node_dependencies(root_uri, &node, &mut entry);
    }
    entry
}

fn push_node_dependencies(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
    entry: &mut ImportedAssetEntry,
) {
    push_dependency_once(
        entry,
        gltf_label_uri(root_uri, &format!("Node{}", node.index())),
    );
    if let Some(mesh) = node.mesh() {
        push_dependency_once(
            entry,
            gltf_label_uri(root_uri, &format!("Mesh{}", mesh.index())),
        );
        for primitive in mesh.primitives() {
            push_dependency_once(
                entry,
                gltf_label_uri(
                    root_uri,
                    &format!("Mesh{}/Primitive{}", mesh.index(), primitive.index()),
                ),
            );
            push_dependency_once(
                entry,
                material_uri_for_index(root_uri, primitive.material().index()),
            );
        }
    }
    for child in node.children() {
        push_node_dependencies(root_uri, &child, entry);
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
        model: gltf_label_reference(root_uri, &format!("Mesh{}", mesh.index())),
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
            mesh: gltf_label_reference(
                root_uri,
                &format!("Mesh{}/Primitive{}", mesh.index(), primitive.index()),
            ),
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

fn push_dependency_once(entry: &mut ImportedAssetEntry, locator: AssetUri) {
    if !entry.dependencies.contains(&locator) {
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
        Some(index) => gltf_label_uri(root_uri, &format!("Material{index}")),
        None => gltf_label_uri(root_uri, "DefaultMaterial"),
    }
}

pub(crate) fn gltf_label_reference(root_uri: &AssetUri, label: &str) -> AssetReference {
    AssetReference::from_locator(gltf_label_uri(root_uri, label))
}

pub(crate) fn gltf_label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}"))
        .expect("generated gltf subasset locator must be valid")
}
