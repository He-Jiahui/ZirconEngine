use std::collections::BTreeMap;

use crate::asset::{
    AlphaMode, AssetImportOutcome, AssetReference, AssetUri, ImportedAsset, ImportedAssetEntry,
    MaterialAsset, MaterialTextureSlotValue,
};
use crate::core::framework::render::RenderMaterialTextureTransform;

use super::{gltf_label_reference, gltf_label_uri, with_root_dependency_and_entry};

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

fn material_asset_from_gltf_material(
    root_uri: &AssetUri,
    uri: AssetUri,
    material: &gltf::Material<'_>,
) -> MaterialAsset {
    let pbr = material.pbr_metallic_roughness();
    let base_color_texture_info = pbr.base_color_texture();
    let normal_texture_info = material.normal_texture();
    let metallic_roughness_texture_info = pbr.metallic_roughness_texture();
    let occlusion_texture_info = material.occlusion_texture();
    let emissive_texture_info = material.emissive_texture();
    let base_color_texture = base_color_texture_info
        .as_ref()
        .map(|info| texture_reference(root_uri, info.texture().index()));
    let normal_texture = normal_texture_info
        .as_ref()
        .map(|texture| texture_reference(root_uri, texture.texture().index()));
    let metallic_roughness_texture = metallic_roughness_texture_info
        .as_ref()
        .map(|info| texture_reference(root_uri, info.texture().index()));
    let occlusion_texture = occlusion_texture_info
        .as_ref()
        .map(|texture| texture_reference(root_uri, texture.texture().index()));
    let emissive_texture = emissive_texture_info
        .as_ref()
        .map(|info| texture_reference(root_uri, info.texture().index()));
    let base_color_metadata = texture_info_metadata(base_color_texture_info.as_ref());
    let normal_metadata = normal_texture_metadata(normal_texture_info.as_ref());
    let metallic_roughness_metadata =
        texture_info_metadata(metallic_roughness_texture_info.as_ref());
    let occlusion_metadata = occlusion_texture_metadata(occlusion_texture_info.as_ref());
    let emissive_metadata = texture_info_metadata(emissive_texture_info.as_ref());

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
    texture_transform_extension_metadata(
        info.tex_coord(),
        info.extension_value("KHR_texture_transform"),
    )
}

fn occlusion_texture_metadata(
    info: Option<&gltf::material::OcclusionTexture<'_>>,
) -> GltfTextureSlotMetadata {
    let Some(info) = info else {
        return GltfTextureSlotMetadata::default();
    };
    texture_transform_extension_metadata(
        info.tex_coord(),
        info.extension_value("KHR_texture_transform"),
    )
}

fn texture_transform_extension_metadata(
    fallback_uv_channel: u32,
    value: Option<&serde_json::Value>,
) -> GltfTextureSlotMetadata {
    let Some(value) = value else {
        return GltfTextureSlotMetadata {
            transform: None,
            uv_channel: fallback_uv_channel,
        };
    };
    let uv_channel = value
        .get("texCoord")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(fallback_uv_channel);
    let transform = RenderMaterialTextureTransform {
        scale: value
            .get("scale")
            .and_then(json_vec2)
            .unwrap_or(RenderMaterialTextureTransform::IDENTITY.scale),
        offset: value
            .get("offset")
            .and_then(json_vec2)
            .unwrap_or(RenderMaterialTextureTransform::IDENTITY.offset),
    };
    GltfTextureSlotMetadata {
        transform: non_identity_texture_transform(transform),
        uv_channel,
    }
}

fn json_vec2(value: &serde_json::Value) -> Option<[f32; 2]> {
    let items = value.as_array()?;
    Some([
        items.first()?.as_f64()? as f32,
        items.get(1)?.as_f64()? as f32,
    ])
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

fn texture_reference(root_uri: &AssetUri, texture_index: usize) -> AssetReference {
    gltf_label_reference(root_uri, &format!("Texture{texture_index}"))
}

fn default_pbr_shader_reference() -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse("res://shaders/default_pbr")
            .expect("default pbr shader locator must be valid"),
    )
}
