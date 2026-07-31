use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};
use std::collections::HashSet;

use crate::asset::{
    AssetImportError, AssetImportOutcome, AssetReference, AssetUri, ImportedAsset,
    ImportedAssetEntry, MeshAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TextureAsset,
    TransformAsset,
};

mod material;

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

pub(crate) fn add_gltf_texture_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
    images: Vec<GltfImageData>,
) -> Result<AssetImportOutcome, AssetImportError> {
    let texture_sources = document
        .textures()
        .map(|texture| gltf_texture_source_index(&texture))
        .collect::<Result<Vec<_>, _>>()?;
    let mut remaining_uses = vec![0usize; images.len()];
    for &image_index in &texture_sources {
        let uses = remaining_uses.get_mut(image_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf texture references missing image {image_index}"
            ))
        })?;
        *uses += 1;
    }
    let mut images = images.into_iter().map(Some).collect::<Vec<_>>();

    for (texture, image_index) in document.textures().zip(texture_sources) {
        let uri = gltf_label_uri(root_uri, &format!("Texture{}", texture.index()));
        let uses = remaining_uses.get_mut(image_index).ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf texture {} references missing image {}",
                texture.index(),
                image_index
            ))
        })?;
        *uses -= 1;
        let image = if *uses == 0 {
            images[image_index].take().expect("validated image source")
        } else {
            images[image_index]
                .as_ref()
                .expect("validated image source")
                .clone()
        };
        let (width, height) = (image.width, image.height);
        let rgba = rgba8_pixels_from_gltf_image(image, image_index)?;
        let asset = TextureAsset::new_rgba8(uri.clone(), width, height, rgba);
        outcome = with_root_dependency_and_entry(
            outcome,
            ImportedAssetEntry::new(uri, ImportedAsset::Texture(asset)),
        );
    }
    Ok(outcome)
}

fn gltf_texture_source_index(texture: &gltf::Texture<'_>) -> Result<usize, AssetImportError> {
    if let Some(extension) = texture.extension_value("EXT_texture_webp") {
        let source = extension
            .get("source")
            .and_then(serde_json::Value::as_u64)
            .and_then(|source| usize::try_from(source).ok())
            .ok_or_else(|| {
                AssetImportError::Parse(format!(
                    "gltf texture {} has malformed EXT_texture_webp source metadata",
                    texture.index()
                ))
            })?;
        return Ok(source);
    }
    texture.source().map(|image| image.index()).ok_or_else(|| {
        AssetImportError::Parse(format!(
            "gltf texture {} has neither a core source nor EXT_texture_webp source",
            texture.index()
        ))
    })
}

pub(crate) fn add_gltf_mesh_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    meshes: Vec<GltfMeshSubasset>,
) -> AssetImportOutcome {
    for mesh in meshes {
        let mesh_uri = gltf_label_uri(root_uri, &format!("Mesh{}", mesh.mesh_index));
        let mesh_model = ModelAsset {
            uri: mesh_uri.clone(),
            primitives: mesh
                .primitives
                .iter()
                .map(|primitive| ModelPrimitiveAsset {
                    vertices: Vec::new(),
                    indices: Vec::new(),
                    mesh: Some(AssetReference::from_locator(primitive.mesh.uri.clone())),
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
                gltf_label_uri(
                    root_uri,
                    &format!(
                        "Mesh{}/Primitive{}",
                        mesh.mesh_index, primitive.primitive_index
                    ),
                ),
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
    image: GltfImageData,
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

    if image.format == GltfImageFormat::R8G8B8A8 {
        validate_image_len(&image, image_index, pixel_count * 4)?;
        return Ok(image.pixels);
    }

    let mut rgba = Vec::with_capacity(pixel_count * 4);
    match image.format {
        GltfImageFormat::R8 => {
            validate_image_len(&image, image_index, pixel_count)?;
            for value in &image.pixels {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        GltfImageFormat::R8G8 => {
            validate_image_len(&image, image_index, pixel_count * 2)?;
            for chunk in image.pixels.chunks_exact(2) {
                rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], chunk[1]]);
            }
        }
        GltfImageFormat::R8G8B8 => {
            validate_image_len(&image, image_index, pixel_count * 3)?;
            for chunk in image.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
        }
        GltfImageFormat::R8G8B8A8 => unreachable!("RGBA8 returned above"),
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
        gltf_label_uri(root_uri, &format!("Node{}", node.index())),
    );
    if let Some(mesh) = node.mesh() {
        push_dependency_once(
            entry,
            dependency_index,
            gltf_label_uri(root_uri, &format!("Mesh{}", mesh.index())),
        );
        for primitive in mesh.primitives() {
            push_dependency_once(
                entry,
                dependency_index,
                gltf_label_uri(
                    root_uri,
                    &format!("Mesh{}/Primitive{}", mesh.index(), primitive.index()),
                ),
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
