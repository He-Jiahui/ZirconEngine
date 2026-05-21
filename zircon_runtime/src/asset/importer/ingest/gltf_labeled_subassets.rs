use std::collections::BTreeMap;

use gltf::image::{Data as GltfImageData, Format as GltfImageFormat};

use crate::asset::{
    AlphaMode, AssetImportError, AssetImportOutcome, AssetReference, AssetUri, DataAsset,
    DataAssetFormat, ImportedAsset, ImportedAssetEntry, MaterialAsset, MaterialTextureSlotValue,
    MeshAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMobilityAsset, TextureAsset, TransformAsset,
};

#[derive(Clone)]
pub(crate) struct GltfMeshSubasset {
    pub(crate) mesh_index: usize,
    pub(crate) primitives: Vec<GltfPrimitiveSubasset>,
}

#[derive(Clone)]
pub(crate) struct GltfPrimitiveSubasset {
    pub(crate) primitive_index: usize,
    pub(crate) material_index: Option<usize>,
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

pub(crate) fn add_gltf_material_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
) -> AssetImportOutcome {
    let default_uri = gltf_label_uri(root_uri, "DefaultMaterial");
    let default_asset = default_material_asset(default_uri.clone());
    outcome = with_root_dependency_and_entry(
        outcome,
        ImportedAssetEntry::new(default_uri, ImportedAsset::Material(default_asset.clone()))
            .with_dependency(default_asset.shader.locator.clone()),
    );

    for material in document.materials() {
        if let Some(material_index) = material.index() {
            let uri = gltf_label_uri(root_uri, &format!("Material{material_index}"));
            let asset = material_asset_from_gltf_material(root_uri, uri.clone(), &material);
            let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Material(asset.clone()))
                .with_dependency(asset.shader.locator.clone());
            for reference in asset
                .all_texture_slots()
                .into_iter()
                .map(|(_, reference)| reference)
            {
                if !entry.dependencies.contains(&reference.locator) {
                    entry = entry.with_dependency(reference.locator.clone());
                }
            }
            outcome = with_root_dependency_and_entry(outcome, entry);
        }
    }
    outcome
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
            let mesh_asset =
                MeshAsset::from_model_primitive(primitive_uri.clone(), &primitive.primitive);
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

pub(crate) fn add_gltf_animation_and_skin_placeholders(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
) -> AssetImportOutcome {
    for animation in document.animations() {
        let label = format!("Animation{}", animation.index());
        let uri = gltf_label_uri(root_uri, &label);
        outcome = with_root_dependency_and_entry(
            outcome,
            ImportedAssetEntry::new(
                uri.clone(),
                ImportedAsset::Data(gltf_placeholder_data_asset(
                    uri,
                    format!("{label}: glTF animation channel import is not implemented yet"),
                )),
            ),
        );
    }

    for skin in document.skins() {
        let label = format!("Skin{}", skin.index());
        let uri = gltf_label_uri(root_uri, &label);
        outcome = with_root_dependency_and_entry(
            outcome,
            ImportedAssetEntry::new(
                uri.clone(),
                ImportedAsset::Data(gltf_placeholder_data_asset(
                    uri,
                    format!("{label}: glTF skin asset import is not implemented yet"),
                )),
            ),
        );
        if skin.inverse_bind_matrices().is_some() {
            let matrices_label = format!("{label}/InverseBindMatrices");
            let matrices_uri = gltf_label_uri(root_uri, &matrices_label);
            outcome = with_root_dependency_and_entry(
                outcome,
                ImportedAssetEntry::new(
                    matrices_uri.clone(),
                    ImportedAsset::Data(gltf_placeholder_data_asset(
                        matrices_uri,
                        format!(
                            "{matrices_label}: inverse bind matrix extraction is not implemented yet"
                        ),
                    )),
                ),
            );
        }
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

fn material_asset_from_gltf_material(
    root_uri: &AssetUri,
    uri: AssetUri,
    material: &gltf::Material<'_>,
) -> MaterialAsset {
    let pbr = material.pbr_metallic_roughness();
    let base_color_texture = pbr
        .base_color_texture()
        .map(|info| texture_reference(root_uri, info.texture().index()));
    let normal_texture = material
        .normal_texture()
        .map(|texture| texture_reference(root_uri, texture.texture().index()));
    let metallic_roughness_texture = pbr
        .metallic_roughness_texture()
        .map(|info| texture_reference(root_uri, info.texture().index()));
    let occlusion_texture = material
        .occlusion_texture()
        .map(|texture| texture_reference(root_uri, texture.texture().index()));
    let emissive_texture = material
        .emissive_texture()
        .map(|info| texture_reference(root_uri, info.texture().index()));

    let mut texture_slots = BTreeMap::new();
    insert_texture_slot(&mut texture_slots, "base_color", &base_color_texture);
    insert_texture_slot(&mut texture_slots, "normal", &normal_texture);
    insert_texture_slot(
        &mut texture_slots,
        "metallic_roughness",
        &metallic_roughness_texture,
    );
    insert_texture_slot(&mut texture_slots, "occlusion", &occlusion_texture);
    insert_texture_slot(&mut texture_slots, "emissive", &emissive_texture);

    MaterialAsset {
        name: material.name().map(str::to_owned),
        shader: default_pbr_shader_reference(),
        base_color: pbr.base_color_factor(),
        base_color_texture,
        normal_texture,
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        metallic_roughness_texture,
        occlusion_texture,
        emissive: material.emissive_factor(),
        emissive_texture,
        alpha_mode: gltf_alpha_mode(material),
        double_sided: material.double_sided(),
        property_values: BTreeMap::new(),
        texture_slots,
        validation_diagnostics: vec![format!(
            "{} imported from glTF Material{}",
            uri,
            material.index().unwrap_or_default()
        )],
    }
}

fn default_material_asset(uri: AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("DefaultMaterial".to_string()),
        shader: default_pbr_shader_reference(),
        base_color: [1.0, 1.0, 1.0, 1.0],
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values: BTreeMap::new(),
        texture_slots: BTreeMap::new(),
        validation_diagnostics: vec![format!(
            "{uri} generated for glTF primitives without material"
        )],
    }
}

fn insert_texture_slot(
    slots: &mut BTreeMap<String, MaterialTextureSlotValue>,
    slot: &str,
    reference: &Option<AssetReference>,
) {
    if let Some(reference) = reference {
        slots.insert(
            slot.to_string(),
            MaterialTextureSlotValue::new(reference.clone()),
        );
    }
}

fn gltf_alpha_mode(material: &gltf::Material<'_>) -> AlphaMode {
    match material.alpha_mode() {
        gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
        gltf::material::AlphaMode::Mask => AlphaMode::Mask {
            cutoff: material.alpha_cutoff().unwrap_or(0.5),
        },
        gltf::material::AlphaMode::Blend => AlphaMode::Blend,
    }
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
    entry
        .dependencies
        .push(gltf_label_uri(root_uri, &format!("Node{}", node.index())));
    if let Some(mesh) = node.mesh() {
        entry
            .dependencies
            .push(gltf_label_uri(root_uri, &format!("Mesh{}", mesh.index())));
        let material_index = first_mesh_material_index(&mesh);
        entry
            .dependencies
            .push(material_uri_for_index(root_uri, material_index));
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
    }
}

fn mesh_instance_from_gltf_node(
    root_uri: &AssetUri,
    node: &gltf::Node<'_>,
) -> Option<SceneMeshInstanceAsset> {
    let mesh = node.mesh()?;
    Some(SceneMeshInstanceAsset {
        model: gltf_label_reference(root_uri, &format!("Mesh{}", mesh.index())),
        material: material_reference_for_index(root_uri, first_mesh_material_index(&mesh)),
    })
}

fn first_mesh_material_index(mesh: &gltf::Mesh<'_>) -> Option<usize> {
    mesh.primitives()
        .next()
        .and_then(|primitive| primitive.material().index())
}

fn transform_from_gltf_node(node: &gltf::Node<'_>) -> TransformAsset {
    let (translation, rotation, scale) = node.transform().decomposed();
    TransformAsset {
        translation,
        rotation,
        scale,
    }
}

fn gltf_placeholder_data_asset(uri: AssetUri, text: String) -> DataAsset {
    DataAsset {
        uri,
        format: DataAssetFormat::Text,
        text,
        canonical_json: Default::default(),
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

fn texture_reference(root_uri: &AssetUri, texture_index: usize) -> AssetReference {
    gltf_label_reference(root_uri, &format!("Texture{texture_index}"))
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

fn default_pbr_shader_reference() -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse("res://shaders/default_pbr.zshader")
            .expect("default pbr shader locator must be valid"),
    )
}

fn gltf_label_reference(root_uri: &AssetUri, label: &str) -> AssetReference {
    AssetReference::from_locator(gltf_label_uri(root_uri, label))
}

fn gltf_label_uri(root_uri: &AssetUri, label: &str) -> AssetUri {
    AssetUri::parse(&format!("{root_uri}#{label}"))
        .expect("generated gltf subasset locator must be valid")
}
