use std::collections::{BTreeMap, HashSet};

use crate::asset::{
    AlphaMode, AssetImportOutcome, AssetReference, AssetUri, ImportedAsset, ImportedAssetEntry,
    MaterialAsset, MaterialTextureSlotValue, STANDARD_MATERIAL_OCCLUSION_STRENGTH_PROPERTY,
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
    let default_shader = default_asset.shader.locator.clone();
    outcome = with_root_dependency_and_entry(
        outcome,
        ImportedAssetEntry::new(default_uri, ImportedAsset::Material(default_asset))
            .with_dependency(default_shader),
    );

    for material in document.materials() {
        if let Some(material_index) = material.index() {
            let uri = gltf_label_uri(root_uri, &format!("Material{material_index}"));
            let asset = material_asset_from_gltf_material(root_uri, uri.clone(), &material);
            let mut dependencies = vec![asset.shader.locator.clone()];
            let mut dependency_index = HashSet::from([asset.shader.locator.clone()]);
            for reference in asset
                .all_texture_slots()
                .into_iter()
                .map(|(_, reference)| reference)
            {
                if dependency_index.insert(reference.locator.clone()) {
                    dependencies.push(reference.locator.clone());
                }
            }
            let mut entry = ImportedAssetEntry::new(uri, ImportedAsset::Material(asset));
            entry.dependencies = dependencies;
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
    let mut emissive = material.emissive_factor();
    let mut property_values = BTreeMap::new();
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
    project_gltf_material_extensions(
        material,
        &uri,
        &mut emissive,
        &mut property_values,
        &mut validation_diagnostics,
    );

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
        emissive,
        emissive_texture,
        alpha_mode: gltf_alpha_mode(material),
        double_sided: material.double_sided(),
        property_values,
        texture_slots,
        validation_diagnostics,
    }
}

fn project_gltf_material_extensions(
    material: &gltf::Material<'_>,
    uri: &AssetUri,
    emissive: &mut [f32; 3],
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) {
    if material.extension_value("KHR_materials_unlit").is_some() {
        properties.insert(
            "lighting_model".to_string(),
            toml::Value::String("unlit".to_string()),
        );
    }

    if let Some(extension) = material.extension_value("KHR_materials_ior") {
        project_f32_extension_property(
            extension,
            "KHR_materials_ior",
            "ior",
            "ior",
            |value| value >= 1.0,
            properties,
            diagnostics,
        );
    }
    if let Some(extension) = material.extension_value("KHR_materials_transmission") {
        project_f32_extension_property(
            extension,
            "KHR_materials_transmission",
            "transmissionFactor",
            "specular_transmission",
            |value| (0.0..=1.0).contains(&value),
            properties,
            diagnostics,
        );
        diagnose_unsupported_extension_field(
            extension,
            uri,
            "KHR_materials_transmission",
            "transmissionTexture",
            diagnostics,
        );
    }
    if let Some(extension) = material.extension_value("KHR_materials_volume") {
        project_f32_extension_property(
            extension,
            "KHR_materials_volume",
            "thicknessFactor",
            "thickness",
            |value| value >= 0.0,
            properties,
            diagnostics,
        );
        project_f32_extension_property(
            extension,
            "KHR_materials_volume",
            "attenuationDistance",
            "attenuation_distance",
            |value| value > 0.0,
            properties,
            diagnostics,
        );
        if let Some(value) = extension.get("attenuationColor") {
            match json_vec3(value).filter(|channels| {
                channels
                    .iter()
                    .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
            }) {
                Some(channels) => {
                    properties.insert(
                        "attenuation_color".to_string(),
                        toml::Value::Array(
                            channels
                                .into_iter()
                                .map(|channel| toml::Value::Float(f64::from(channel)))
                                .collect(),
                        ),
                    );
                }
                None => diagnostics.push(
                    "KHR_materials_volume.attenuationColor must contain three finite 0..=1 values"
                        .to_string(),
                ),
            }
        }
        diagnose_unsupported_extension_field(
            extension,
            uri,
            "KHR_materials_volume",
            "thicknessTexture",
            diagnostics,
        );
    }
    if let Some(extension) = material.extension_value("KHR_materials_emissive_strength") {
        if let Some(value) = extension.get("emissiveStrength") {
            match json_f32(value).filter(|value| *value >= 0.0) {
                Some(strength) => {
                    for channel in emissive {
                        *channel *= strength;
                    }
                }
                None => diagnostics.push(
                    "KHR_materials_emissive_strength.emissiveStrength must be non-negative and finite"
                        .to_string(),
                ),
            }
        }
    }
    if material.extension_value("KHR_materials_specular").is_some() {
        diagnostics.push(format!(
            "{uri} uses KHR_materials_specular, but the current StandardMaterialDescriptor has no specular factor/color fields; core PBR fallback values were retained"
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn project_f32_extension_property(
    extension: &serde_json::Value,
    extension_name: &str,
    field_name: &str,
    property_name: &str,
    validate: impl FnOnce(f32) -> bool,
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) {
    let Some(value) = extension.get(field_name) else {
        return;
    };
    match json_f32(value).filter(|value| validate(*value)) {
        Some(value) => {
            properties.insert(
                property_name.to_string(),
                toml::Value::Float(f64::from(value)),
            );
        }
        None => diagnostics.push(format!(
            "{extension_name}.{field_name} contains an invalid numeric value"
        )),
    }
}

fn diagnose_unsupported_extension_field(
    extension: &serde_json::Value,
    uri: &AssetUri,
    extension_name: &str,
    field_name: &str,
    diagnostics: &mut Vec<String>,
) {
    if extension.get(field_name).is_some() {
        diagnostics.push(format!(
            "{uri} uses unsupported {extension_name}.{field_name}; the texture contribution was not projected"
        ));
    }
}

fn json_f32(value: &serde_json::Value) -> Option<f32> {
    let value = value.as_f64()? as f32;
    value.is_finite().then_some(value)
}

fn json_vec3(value: &serde_json::Value) -> Option<[f32; 3]> {
    let items = value.as_array()?;
    if items.len() != 3 {
        return None;
    }
    Some([
        json_f32(items.first()?)?,
        json_f32(items.get(1)?)?,
        json_f32(items.get(2)?)?,
    ])
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
