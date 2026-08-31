use std::collections::BTreeMap;

use zircon_runtime::asset::assets::{
    default_pbr_shader_reference, STANDARD_MATERIAL_NORMAL_SCALE_PROPERTY,
    STANDARD_MATERIAL_OCCLUSION_STRENGTH_PROPERTY,
};
use zircon_runtime::asset::importer::{
    gltf_texture_color_space_usages, gltf_texture_label, gltf_texture_variant,
    project_gltf_material_extensions, project_gltf_texture_transform, GltfTextureColorSpace,
    GltfTextureTransformProjection, GltfTextureUsage,
};
use zircon_runtime::asset::{
    AlphaMode, AssetImportError, AssetImportOutcome, AssetReference, AssetUri, DataAsset,
    DataAssetFormat, ImportedAsset, ImportedAssetEntry, MaterialAsset, MaterialTextureSlotValue,
    MeshAsset, ModelAsset, ModelPrimitiveAsset, SceneAsset, SceneEntityAsset,
    SceneMeshInstanceAsset, SceneMeshPrimitiveBindingAsset, SceneMobilityAsset, TransformAsset,
};
use zircon_runtime::core::framework::render::{RenderMaterialTextureTransform, TextureUsageHint};

#[cfg(test)]
mod texture_variant_tests;

#[derive(Clone)]
pub(crate) struct GltfMeshSubasset {
    pub(crate) mesh_index: usize,
    pub(crate) primitives: Vec<GltfPrimitiveSubasset>,
}

#[derive(Clone)]
pub(crate) struct GltfPrimitiveSubasset {
    pub(crate) primitive_index: usize,
    pub(crate) material_index: Option<usize>,
    pub(crate) mesh: MeshAsset,
}

pub(crate) use zircon_runtime::asset::importer::{
    add_gltf_texture_subassets, validate_gltf_texture_import_support,
};

pub(crate) fn add_gltf_material_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
) -> AssetImportOutcome {
    let texture_usages = gltf_texture_color_space_usages(document);
    let default_uri = gltf_label_uri(root_uri, "DefaultMaterial");
    let default_asset = default_material_asset(default_uri.clone());
    outcome = with_root_dependency_and_entry(
        outcome,
        material_entry_from_asset(default_uri, default_asset),
    );

    for material in document.materials() {
        if let Some(material_index) = material.index() {
            let uri = gltf_label_uri(root_uri, &format!("Material{material_index}"));
            let asset = material_asset_from_gltf_material(
                root_uri,
                uri.clone(),
                &material,
                &texture_usages,
            );
            outcome =
                with_root_dependency_and_entry(outcome, material_entry_from_asset(uri, asset));
        }
    }
    outcome
}

pub(crate) fn material_entry_from_asset(uri: AssetUri, asset: MaterialAsset) -> ImportedAssetEntry {
    let mut dependencies = vec![asset.shader.locator.clone()];
    for reference in asset
        .all_texture_slots()
        .into_iter()
        .map(|(_, reference)| reference)
    {
        if !dependencies.contains(&reference.locator) {
            dependencies.push(reference.locator.clone());
        }
    }
    let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Material(asset));
    entry.dependencies = dependencies;
    entry
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
                    mesh_sdf: None,
                    virtual_geometry: None,
                })
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

        for primitive in mesh.primitives {
            let primitive_uri = gltf_label_uri(
                root_uri,
                &format!(
                    "Mesh{}/Primitive{}",
                    mesh.mesh_index, primitive.primitive_index
                ),
            );
            debug_assert_eq!(primitive.mesh.uri, primitive_uri);
            let entry = ImportedAssetEntry::new(primitive_uri, ImportedAsset::Mesh(primitive.mesh))
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

pub(crate) fn add_gltf_animation_placeholders_and_skin_subassets(
    mut outcome: AssetImportOutcome,
    root_uri: &AssetUri,
    document: &gltf::Document,
    buffers: &[gltf::buffer::Data],
) -> Result<AssetImportOutcome, AssetImportError> {
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
        let inverse_bind_matrices = inverse_bind_matrices_for_skin(&skin, buffers)?;
        let matrices_uri = inverse_bind_matrices
            .as_ref()
            .map(|_| gltf_label_uri(root_uri, &format!("{label}/InverseBindMatrices")));
        let mut skin_entry = ImportedAssetEntry::new(
            uri.clone(),
            ImportedAsset::Data(gltf_skin_data_asset(
                root_uri,
                uri,
                &label,
                &skin,
                matrices_uri.as_ref(),
                inverse_bind_matrices
                    .as_ref()
                    .map_or(0, |matrices| matrices.len()),
            )),
        );
        for joint in skin.joints() {
            push_dependency_once(
                &mut skin_entry,
                gltf_label_uri(root_uri, &format!("Node{}", joint.index())),
            );
        }
        if let Some(skeleton) = skin.skeleton() {
            push_dependency_once(
                &mut skin_entry,
                gltf_label_uri(root_uri, &format!("Node{}", skeleton.index())),
            );
        }
        if let Some(matrices_uri) = &matrices_uri {
            push_dependency_once(&mut skin_entry, matrices_uri.clone());
        }
        outcome = with_root_dependency_and_entry(outcome, skin_entry);

        if skin.inverse_bind_matrices().is_some() {
            let matrices_label = format!("{label}/InverseBindMatrices");
            let matrices_uri = matrices_uri.expect("matrix uri should exist when accessor exists");
            let inverse_bind_matrices =
                inverse_bind_matrices.expect("matrix payload should exist when accessor exists");
            outcome = with_root_dependency_and_entry(
                outcome,
                ImportedAssetEntry::new(
                    matrices_uri.clone(),
                    ImportedAsset::Data(gltf_inverse_bind_matrices_data_asset(
                        matrices_uri,
                        &matrices_label,
                        inverse_bind_matrices,
                    )),
                ),
            );
        }
    }
    Ok(outcome)
}

fn inverse_bind_matrices_for_skin(
    skin: &gltf::Skin<'_>,
    buffers: &[gltf::buffer::Data],
) -> Result<Option<Vec<[[f32; 4]; 4]>>, AssetImportError> {
    let Some(accessor) = skin.inverse_bind_matrices() else {
        return Ok(None);
    };
    let matrices = skin
        .reader(|buffer| Some(&buffers[buffer.index()].0))
        .read_inverse_bind_matrices()
        .ok_or_else(|| {
            AssetImportError::Parse(format!(
                "gltf Skin{} inverseBindMatrices accessor {} could not be read",
                skin.index(),
                accessor.index()
            ))
        })?
        .collect();
    Ok(Some(matrices))
}

fn material_asset_from_gltf_material(
    root_uri: &AssetUri,
    uri: AssetUri,
    material: &gltf::Material<'_>,
    texture_usages: &[GltfTextureUsage],
) -> MaterialAsset {
    let pbr = material.pbr_metallic_roughness();
    let base_color_texture_info = pbr.base_color_texture();
    let normal_texture_info = material.normal_texture();
    let metallic_roughness_texture_info = pbr.metallic_roughness_texture();
    let occlusion_texture_info = material.occlusion_texture();
    let emissive_texture_info = material.emissive_texture();
    let base_color_texture = base_color_texture_info.as_ref().map(|info| {
        texture_reference(
            root_uri,
            info.texture().index(),
            GltfTextureColorSpace::Srgb,
            TextureUsageHint::Albedo,
            texture_usages,
        )
    });
    let normal_texture = normal_texture_info.as_ref().map(|texture| {
        texture_reference(
            root_uri,
            texture.texture().index(),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Normal,
            texture_usages,
        )
    });
    let metallic_roughness_texture = metallic_roughness_texture_info.as_ref().map(|info| {
        texture_reference(
            root_uri,
            info.texture().index(),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Data,
            texture_usages,
        )
    });
    let occlusion_texture = occlusion_texture_info.as_ref().map(|texture| {
        texture_reference(
            root_uri,
            texture.texture().index(),
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Data,
            texture_usages,
        )
    });
    let emissive_texture = emissive_texture_info.as_ref().map(|info| {
        texture_reference(
            root_uri,
            info.texture().index(),
            GltfTextureColorSpace::Srgb,
            TextureUsageHint::Albedo,
            texture_usages,
        )
    });
    let base_color_metadata = texture_info_metadata(base_color_texture_info.as_ref());
    let normal_metadata = normal_texture_metadata(normal_texture_info.as_ref());
    let metallic_roughness_metadata =
        texture_info_metadata(metallic_roughness_texture_info.as_ref());
    let occlusion_metadata = occlusion_texture_metadata(occlusion_texture_info.as_ref());
    let emissive_metadata = texture_info_metadata(emissive_texture_info.as_ref());
    let mut emissive = material.emissive_factor();
    let mut property_values = BTreeMap::new();
    if let Some(normal_texture_info) = normal_texture_info.as_ref() {
        let scale = normal_texture_info.scale();
        if scale != 1.0 {
            property_values.insert(
                STANDARD_MATERIAL_NORMAL_SCALE_PROPERTY.to_string(),
                toml::Value::Float(f64::from(scale)),
            );
        }
    }
    if let Some(occlusion_texture_info) = occlusion_texture_info.as_ref() {
        let strength = occlusion_texture_info.strength();
        if (strength - 1.0).abs() > f32::EPSILON {
            property_values.insert(
                STANDARD_MATERIAL_OCCLUSION_STRENGTH_PROPERTY.to_string(),
                toml::Value::Float(f64::from(strength)),
            );
        }
    }
    let mut validation_diagnostics = vec![format!(
        "{} imported from glTF Material{}",
        uri,
        material.index().unwrap_or_default()
    )];
    let clearcoat_normal_projection = project_gltf_material_extensions(
        material,
        &uri,
        &mut emissive,
        &mut property_values,
        &mut validation_diagnostics,
    );
    let clearcoat_normal_texture = clearcoat_normal_projection.map(|projection| {
        texture_reference(
            root_uri,
            projection.texture_index,
            GltfTextureColorSpace::Linear,
            TextureUsageHint::Normal,
            texture_usages,
        )
    });
    let clearcoat_normal_metadata =
        clearcoat_normal_projection.map_or(GltfTextureSlotMetadata::default(), |projection| {
            GltfTextureSlotMetadata {
                transform: projection.transform,
                uv_channel: projection.uv_channel,
            }
        });

    let mut texture_slots = BTreeMap::new();
    insert_texture_slot(
        &mut texture_slots,
        "base_color",
        &base_color_texture,
        base_color_metadata,
    );
    insert_texture_slot(
        &mut texture_slots,
        "normal",
        &normal_texture,
        normal_metadata,
    );
    insert_texture_slot(
        &mut texture_slots,
        "metallic_roughness",
        &metallic_roughness_texture,
        metallic_roughness_metadata,
    );
    insert_texture_slot(
        &mut texture_slots,
        "occlusion",
        &occlusion_texture,
        occlusion_metadata,
    );
    insert_texture_slot(
        &mut texture_slots,
        "emissive",
        &emissive_texture,
        emissive_metadata,
    );
    insert_texture_slot(
        &mut texture_slots,
        "clearcoat_normal",
        &clearcoat_normal_texture,
        clearcoat_normal_metadata,
    );

    MaterialAsset {
        name: material.name().map(str::to_owned),
        shader: default_pbr_shader_reference(),
        parent: None,
        options: Default::default(),
        queue: None,
        base_color: pbr.base_color_factor(),
        base_color_texture,
        normal_texture,
        metallic: pbr.metallic_factor(),
        roughness: pbr.roughness_factor(),
        metallic_roughness_texture,
        occlusion_texture,
        emissive,
        emissive_texture,
        alpha_mode: gltf_alpha_mode(material),
        double_sided: material.double_sided(),
        property_values,
        texture_slots,
        validation_diagnostics,
    }
}

#[derive(Clone, Copy, Default)]
struct GltfTextureSlotMetadata {
    transform: Option<RenderMaterialTextureTransform>,
    uv_channel: u32,
}

fn texture_info_metadata(info: Option<&gltf::texture::Info<'_>>) -> GltfTextureSlotMetadata {
    let Some(info) = info else {
        return GltfTextureSlotMetadata::default();
    };
    let mut metadata = GltfTextureSlotMetadata {
        transform: None,
        uv_channel: info.tex_coord(),
    };
    if let Some(transform) = info.texture_transform() {
        metadata.uv_channel = transform.tex_coord().unwrap_or(metadata.uv_channel);
        metadata.transform = non_identity_texture_transform(RenderMaterialTextureTransform {
            scale: transform.scale(),
            offset: transform.offset(),
            rotation: transform.rotation(),
        });
    }
    metadata
}

fn normal_texture_metadata(
    info: Option<&gltf::material::NormalTexture<'_>>,
) -> GltfTextureSlotMetadata {
    let Some(info) = info else {
        return GltfTextureSlotMetadata::default();
    };
    texture_transform_projection_metadata(project_gltf_texture_transform(
        info.tex_coord(),
        info.extension_value("KHR_texture_transform"),
    ))
}

fn occlusion_texture_metadata(
    info: Option<&gltf::material::OcclusionTexture<'_>>,
) -> GltfTextureSlotMetadata {
    let Some(info) = info else {
        return GltfTextureSlotMetadata::default();
    };
    texture_transform_projection_metadata(project_gltf_texture_transform(
        info.tex_coord(),
        info.extension_value("KHR_texture_transform"),
    ))
}

fn texture_transform_projection_metadata(
    projection: GltfTextureTransformProjection,
) -> GltfTextureSlotMetadata {
    GltfTextureSlotMetadata {
        transform: projection.transform,
        uv_channel: projection.uv_channel,
    }
}

fn non_identity_texture_transform(
    transform: RenderMaterialTextureTransform,
) -> Option<RenderMaterialTextureTransform> {
    (!transform.is_identity()).then_some(transform)
}

fn default_material_asset(uri: AssetUri) -> MaterialAsset {
    MaterialAsset {
        name: Some("DefaultMaterial".to_string()),
        shader: default_pbr_shader_reference(),
        parent: None,
        options: Default::default(),
        queue: None,
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
    metadata: GltfTextureSlotMetadata,
) {
    if let Some(reference) = reference {
        let mut value = MaterialTextureSlotValue::new(reference.clone());
        value.transform = metadata.transform;
        value.uv_channel = metadata.uv_channel;
        slots.insert(slot.to_string(), value);
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

fn gltf_placeholder_data_asset(uri: AssetUri, text: String) -> DataAsset {
    DataAsset {
        uri,
        format: DataAssetFormat::Text,
        text,
        canonical_json: Default::default(),
    }
}

fn gltf_skin_data_asset(
    root_uri: &AssetUri,
    uri: AssetUri,
    label: &str,
    skin: &gltf::Skin<'_>,
    inverse_bind_matrices_uri: Option<&AssetUri>,
    inverse_bind_matrix_count: usize,
) -> DataAsset {
    let joints = skin
        .joints()
        .map(|joint| {
            serde_json::json!({
                "node_index": joint.index(),
                "node": gltf_label_uri(root_uri, &format!("Node{}", joint.index())).to_string(),
                "name": joint.name(),
            })
        })
        .collect::<Vec<_>>();
    let joint_count = joints.len();
    let skeleton = skin.skeleton().map(|node| {
        serde_json::json!({
            "node_index": node.index(),
            "node": gltf_label_uri(root_uri, &format!("Node{}", node.index())).to_string(),
            "name": node.name(),
        })
    });
    let canonical_json = serde_json::json!({
        "kind": "gltf_skin",
        "label": label,
        "skin_index": skin.index(),
        "name": skin.name(),
        "skeleton": skeleton,
        "joints": joints,
        "joint_count": joint_count,
        "inverse_bind_matrices": inverse_bind_matrices_uri.map(ToString::to_string),
        "inverse_bind_matrix_count": inverse_bind_matrix_count,
    });
    json_data_asset(uri, canonical_json)
}

fn gltf_inverse_bind_matrices_data_asset(
    uri: AssetUri,
    label: &str,
    inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
) -> DataAsset {
    json_data_asset(
        uri,
        serde_json::json!({
            "kind": "gltf_inverse_bind_matrices",
            "label": label,
            "matrix_count": inverse_bind_matrices.len(),
            "matrices": inverse_bind_matrices,
        }),
    )
}

fn json_data_asset(uri: AssetUri, canonical_json: serde_json::Value) -> DataAsset {
    DataAsset {
        uri,
        format: DataAssetFormat::Json,
        text: serde_json::to_string_pretty(&canonical_json)
            .expect("generated gltf data JSON should serialize"),
        canonical_json,
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

fn texture_reference(
    root_uri: &AssetUri,
    texture_index: usize,
    color_space: GltfTextureColorSpace,
    usage_hint: TextureUsageHint,
    texture_usages: &[GltfTextureUsage],
) -> AssetReference {
    gltf_label_reference(
        root_uri,
        &gltf_texture_label(
            texture_index,
            gltf_texture_variant(color_space, usage_hint),
            texture_usages,
        ),
    )
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
