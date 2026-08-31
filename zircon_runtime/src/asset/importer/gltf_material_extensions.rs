use std::collections::BTreeMap;

use crate::asset::AssetUri;
use crate::core::framework::render::RenderMaterialTextureTransform;

use super::error::AssetImportError;
use super::project_gltf_texture_transform;

const GLTF_ANISOTROPY_EXTENSION: &str = "KHR_materials_anisotropy";
const GLTF_CLEARCOAT_EXTENSION: &str = "KHR_materials_clearcoat";
const GLTF_IOR_EXTENSION: &str = "KHR_materials_ior";
const GLTF_TRANSMISSION_EXTENSION: &str = "KHR_materials_transmission";
const GLTF_VOLUME_EXTENSION: &str = "KHR_materials_volume";
const UNSUPPORTED_OPTIONAL_GLTF_MATERIAL_EXTENSIONS: &[&str] = &[
    "KHR_materials_diffuse_transmission",
    "KHR_materials_dispersion",
    "KHR_materials_iridescence",
    "KHR_materials_pbrSpecularGlossiness",
    "KHR_materials_sheen",
    "KHR_materials_subsurface",
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GltfClearcoatNormalTextureProjection {
    pub texture_index: usize,
    pub transform: Option<RenderMaterialTextureTransform>,
    pub uv_channel: u32,
    pub scale: f32,
}

/// Rejects required glTF material semantics that the current material/shader
/// contract cannot represent. Optional uses retain their diagnostic fallback.
pub fn validate_required_gltf_material_extension_support(
    document: &gltf::Document,
    required_extensions: &[String],
) -> Result<(), AssetImportError> {
    let requires_anisotropy = required_extensions
        .iter()
        .any(|extension| extension == GLTF_ANISOTROPY_EXTENSION);
    let requires_clearcoat = required_extensions
        .iter()
        .any(|extension| extension == GLTF_CLEARCOAT_EXTENSION);
    let requires_ior = required_extensions
        .iter()
        .any(|extension| extension == GLTF_IOR_EXTENSION);
    let requires_transmission = required_extensions
        .iter()
        .any(|extension| extension == GLTF_TRANSMISSION_EXTENSION);
    let requires_volume = required_extensions
        .iter()
        .any(|extension| extension == GLTF_VOLUME_EXTENSION);
    if !requires_anisotropy
        && !requires_clearcoat
        && !requires_ior
        && !requires_transmission
        && !requires_volume
    {
        return Ok(());
    }

    for material in document.materials() {
        let material_index = material.index().unwrap_or_default();
        for (required, extension_name, field_name) in [
            (
                requires_anisotropy,
                GLTF_ANISOTROPY_EXTENSION,
                "anisotropyTexture",
            ),
            (
                requires_clearcoat,
                GLTF_CLEARCOAT_EXTENSION,
                "clearcoatTexture",
            ),
            (
                requires_clearcoat,
                GLTF_CLEARCOAT_EXTENSION,
                "clearcoatRoughnessTexture",
            ),
            (
                requires_transmission,
                GLTF_TRANSMISSION_EXTENSION,
                "transmissionTexture",
            ),
            (requires_volume, GLTF_VOLUME_EXTENSION, "thicknessTexture"),
        ] {
            if required
                && material
                    .extension_value(extension_name)
                    .and_then(|extension| extension.get(field_name))
                    .is_some()
            {
                return Err(unsupported_required_material_semantic(
                    material_index,
                    &format!("{extension_name}.{field_name}"),
                ));
            }
        }
        if requires_ior
            && material
                .extension_value(GLTF_IOR_EXTENSION)
                .and_then(|extension| extension.get("ior"))
                .and_then(serde_json::Value::as_f64)
                .is_some_and(|ior| ior == 0.0)
        {
            return Err(unsupported_required_material_semantic(
                material_index,
                "KHR_materials_ior.ior=0 compatibility mode",
            ));
        }
    }
    Ok(())
}

fn unsupported_required_material_semantic(
    material_index: usize,
    semantic: &str,
) -> AssetImportError {
    AssetImportError::Parse(format!(
        "gltf requires unsupported {semantic} at material {material_index}"
    ))
}

pub fn gltf_clearcoat_normal_texture_projection(
    material: &gltf::Material<'_>,
) -> Option<GltfClearcoatNormalTextureProjection> {
    let extension = material.extension_value(GLTF_CLEARCOAT_EXTENSION)?;
    project_gltf_clearcoat_normal_texture(extension, None)
}

/// Projects the glTF material-extension subset owned by Standard PBR into the
/// existing material property and diagnostic contracts.
pub fn project_gltf_material_extensions(
    material: &gltf::Material<'_>,
    uri: &AssetUri,
    emissive: &mut [f32; 3],
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) -> Option<GltfClearcoatNormalTextureProjection> {
    if material.extension_value("KHR_materials_unlit").is_some() {
        properties.insert(
            "lighting_model".to_string(),
            toml::Value::String("unlit".to_string()),
        );
    }

    if let Some(extension) = material.extension_value(GLTF_ANISOTROPY_EXTENSION) {
        project_f32_extension_property_with_default(
            extension,
            GLTF_ANISOTROPY_EXTENSION,
            "anisotropyStrength",
            "anisotropy_strength",
            0.0,
            |value| (0.0..=1.0).contains(&value),
            properties,
            diagnostics,
        );
        project_f32_extension_property_with_default(
            extension,
            GLTF_ANISOTROPY_EXTENSION,
            "anisotropyRotation",
            "anisotropy_rotation",
            0.0,
            f32::is_finite,
            properties,
            diagnostics,
        );
        diagnose_unsupported_extension_field(
            extension,
            uri,
            GLTF_ANISOTROPY_EXTENSION,
            "anisotropyTexture",
            diagnostics,
        );
    }
    if let Some(extension) = material.extension_value("KHR_materials_ior") {
        project_gltf_ior(extension, uri, properties, diagnostics);
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
    for extension_name in UNSUPPORTED_OPTIONAL_GLTF_MATERIAL_EXTENSIONS {
        if material.extension_value(extension_name).is_some() {
            diagnostics.push(format!(
                "{uri} uses unsupported optional {extension_name}; core PBR fallback material semantics were retained"
            ));
        }
    }

    material
        .extension_value(GLTF_CLEARCOAT_EXTENSION)
        .and_then(|extension| project_gltf_clearcoat(extension, uri, properties, diagnostics))
}

fn project_gltf_clearcoat(
    extension: &serde_json::Value,
    uri: &AssetUri,
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) -> Option<GltfClearcoatNormalTextureProjection> {
    project_f32_extension_property_with_default(
        extension,
        GLTF_CLEARCOAT_EXTENSION,
        "clearcoatFactor",
        "clearcoat",
        0.0,
        |value| (0.0..=1.0).contains(&value),
        properties,
        diagnostics,
    );
    project_f32_extension_property_with_default(
        extension,
        GLTF_CLEARCOAT_EXTENSION,
        "clearcoatRoughnessFactor",
        "clearcoat_perceptual_roughness",
        0.0,
        |value| (0.0..=1.0).contains(&value),
        properties,
        diagnostics,
    );
    for field in ["clearcoatTexture", "clearcoatRoughnessTexture"] {
        diagnose_unsupported_extension_field(
            extension,
            uri,
            GLTF_CLEARCOAT_EXTENSION,
            field,
            diagnostics,
        );
    }

    let projection = project_gltf_clearcoat_normal_texture(extension, Some(diagnostics))?;
    if projection.scale.to_bits() != 1.0_f32.to_bits() {
        properties.insert(
            "clearcoat_normal_scale".to_string(),
            toml::Value::Float(f64::from(projection.scale)),
        );
    }
    Some(projection)
}

fn project_gltf_clearcoat_normal_texture(
    extension: &serde_json::Value,
    mut diagnostics: Option<&mut Vec<String>>,
) -> Option<GltfClearcoatNormalTextureProjection> {
    let info = extension.get("clearcoatNormalTexture")?;
    let Some(texture_index) = json_texture_index(info) else {
        if let Some(diagnostics) = diagnostics.as_deref_mut() {
            diagnostics.push(
                "KHR_materials_clearcoat.clearcoatNormalTexture.index must be a non-negative integer"
                    .to_string(),
            );
        }
        return None;
    };
    let fallback_uv_channel = info
        .get("texCoord")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or_default();
    let projection = project_gltf_texture_transform(
        fallback_uv_channel,
        info.get("extensions")
            .and_then(|extensions| extensions.get("KHR_texture_transform")),
    );
    let scale = match info.get("scale") {
        Some(value) => match json_f32(value) {
            Some(value) => value,
            None => {
                if let Some(diagnostics) = diagnostics.as_deref_mut() {
                    diagnostics.push(
                        "KHR_materials_clearcoat.clearcoatNormalTexture.scale must be finite"
                            .to_string(),
                    );
                }
                1.0
            }
        },
        None => 1.0,
    };
    Some(GltfClearcoatNormalTextureProjection {
        texture_index,
        transform: projection.transform,
        uv_channel: projection.uv_channel,
        scale,
    })
}

fn project_gltf_ior(
    extension: &serde_json::Value,
    uri: &AssetUri,
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) {
    let Some(value) = extension.get("ior") else {
        return;
    };
    match json_f32(value) {
        Some(value) if value == 0.0 => diagnostics.push(format!(
            "{uri} KHR_materials_ior.ior = 0 requests the unsupported specular-glossiness compatibility mode"
        )),
        Some(value) if value >= 1.0 => {
            properties.insert("ior".to_string(), toml::Value::Float(f64::from(value)));
        }
        _ => diagnostics.push("KHR_materials_ior.ior contains an invalid numeric value".to_string()),
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

#[allow(clippy::too_many_arguments)]
fn project_f32_extension_property_with_default(
    extension: &serde_json::Value,
    extension_name: &str,
    field_name: &str,
    property_name: &str,
    default: f32,
    validate: impl FnOnce(f32) -> bool,
    properties: &mut BTreeMap<String, toml::Value>,
    diagnostics: &mut Vec<String>,
) {
    let value = extension
        .get(field_name)
        .map(json_f32)
        .unwrap_or(Some(default));
    match value.filter(|value| validate(*value)) {
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

fn json_texture_index(value: &serde_json::Value) -> Option<usize> {
    value
        .get("index")?
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
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
