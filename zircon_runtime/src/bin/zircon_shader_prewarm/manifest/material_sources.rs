use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use zircon_runtime::asset::{
    AlphaMode, AssetReference, AssetRegistryIndex, MaterialAsset, ReferenceResolutionError,
    ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    GeometrySourceDescriptor, GeometrySourceId, ShaderFeatureBits, ShaderPassType,
    ShaderQualityTier, ShaderVariantPrewarmRequest, ShadingModelId, SHADING_MODEL_ID_STANDARD_PBR,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::dynamic_api::{
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
};

use super::paths::{has_extension, stable_label_for_path};
use super::{
    prewarm_requests_for_source_with_dimensions, ShaderPrewarmSource,
    BUILTIN_STANDARD_MATERIAL_SHADER_URI,
};
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

const ASSET_SCAN_ALPHA_BLEND_PASSES: [ShaderPassType; 1] = [ShaderPassType::Forward];

pub(super) fn collect_material_sources(
    root: &Path,
    asset_root: &Path,
    sources: &mut Vec<MaterialPrewarmSource>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
) -> ShaderPrewarmAssetScanResult<()> {
    let registry =
        AssetRegistryIndex::inspect_project(&[asset_root.to_path_buf()]).map_err(|source| {
            ShaderPrewarmAssetScanError::InspectAssetRegistry {
                path: asset_root.to_path_buf(),
                source,
            }
        })?;
    collect_material_sources_with_registry(root, asset_root, sources, shading_model_ids, &registry)
}

fn collect_material_sources_with_registry(
    root: &Path,
    asset_root: &Path,
    sources: &mut Vec<MaterialPrewarmSource>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    registry: &AssetRegistryIndex,
) -> ShaderPrewarmAssetScanResult<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in
        fs::read_dir(root).map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRoot {
            path: root.to_path_buf(),
            source,
        })?
    {
        let entry = entry.map_err(|source| ShaderPrewarmAssetScanError::ReadAssetRootEntry {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_material_sources_with_registry(
                &path,
                asset_root,
                sources,
                shading_model_ids,
                registry,
            )?;
            continue;
        }
        if has_extension(&path, "zmaterial") {
            sources.push(material_source_from_zmaterial(
                asset_root,
                &path,
                shading_model_ids,
                registry,
            )?);
        }
    }
    Ok(())
}

pub(super) fn prewarm_requests_for_material_source(
    material: MaterialPrewarmSource,
    shader_sources: &[ShaderPrewarmSource],
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
) -> Vec<ShaderVariantPrewarmRequest> {
    let Some(shader_source) = shader_sources
        .iter()
        .find(|source| {
            source.stable_label == material.shader_label
                || source.resource_id == material.shader_resource_id
        })
        .cloned()
    else {
        if material.uses_builtin_standard_shader {
            return geometry_sources
                .iter()
                .copied()
                .flat_map(|geometry_source| {
                    if let Some(descriptor) = geometry_source_descriptors.get(&geometry_source) {
                        builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
                            material.features,
                            material.shading_model,
                            material.alpha_cutoff,
                            descriptor,
                            quality_tiers,
                        )
                        .variants
                    } else {
                        builtin_standard_material_shader_prewarm_manifest_for_geometry(
                            material.features,
                            material.shading_model,
                            material.alpha_cutoff,
                            geometry_source,
                            quality_tiers,
                        )
                        .variants
                    }
                })
                .collect();
        }
        return Vec::new();
    };
    let material_layout_hash = shader_source.material_layout_hash;
    let material_option_bits = shader_source
        .material_option_table
        .bits_for_values(&material.material_option_values);
    let shader_source = material.apply_to_shader_source(shader_source);
    prewarm_requests_for_source_with_dimensions(
        shader_source,
        material.features,
        material.shading_model,
        material.alpha_cutoff,
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
        material_layout_hash,
        material_option_bits,
    )
}

fn material_source_from_zmaterial(
    asset_root: &Path,
    material_path: &Path,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    registry: &AssetRegistryIndex,
) -> ShaderPrewarmAssetScanResult<MaterialPrewarmSource> {
    let document = fs::read_to_string(material_path).map_err(|source| {
        ShaderPrewarmAssetScanError::ReadZMaterial {
            path: material_path.to_path_buf(),
            source,
        }
    })?;
    let material_document = ZMaterialDocument::from_project_toml_str(&document, |reference| {
        if let Some(locator) = reference.builtin_locator() {
            return Ok::<_, ReferenceResolutionError>(AssetReference::from_locator(
                locator.clone(),
            ));
        }
        let reference = reference
            .project_ref()
            .ok_or(ReferenceResolutionError::MissingPayload)?;
        let entry = registry.entry_by_uuid(reference.guid()).ok_or(
            ReferenceResolutionError::MissingGuid {
                guid: reference.guid(),
            },
        )?;
        let root_name = asset_root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ReferenceResolutionError::Registry {
                message: format!("asset root {} has no UTF-8 root name", asset_root.display()),
            })?;
        let expected_path_hint = format!("{root_name}/{}", entry.path().path());
        if entry.path().label() != reference.sub()
            || reference.path_hint().as_str() != expected_path_hint
        {
            return Err(ReferenceResolutionError::Registry {
                message: format!(
                    "asset ref {} disagrees with registry path {}",
                    reference.guid(),
                    entry.path()
                ),
            });
        }
        Ok::<_, ReferenceResolutionError>(AssetReference::new(entry.uuid(), entry.path().clone()))
    })
    .map_err(|source| ShaderPrewarmAssetScanError::ParseZMaterial {
        path: material_path.to_path_buf(),
        source,
    })?;
    let material = MaterialAsset::from_zmaterial_document(material_document);
    Ok(MaterialPrewarmSource {
        stable_label: stable_label_for_path(asset_root, material_path),
        shader_label: material.shader.locator.to_string(),
        shader_resource_id: ResourceId::from_asset_uuid(material.shader.uuid),
        features: material_feature_bits(&material),
        shading_model: material_shading_model_id(&material, shading_model_ids),
        alpha_cutoff: material_alpha_cutoff(&material),
        pass_filter: material_pass_filter(&material),
        material_option_values: material.material_option_values().clone(),
        uses_builtin_standard_shader: material_uses_builtin_standard_shader(&material),
    })
}

fn material_feature_bits(material: &MaterialAsset) -> ShaderFeatureBits {
    let mut bits = 0;
    if matches!(material.alpha_mode, AlphaMode::Mask { .. }) {
        bits |= ShaderFeatureBits::ALPHA_TEST;
    }
    if material.receive_shadows() {
        bits |= ShaderFeatureBits::RECEIVE_SHADOWS;
    }
    if material.double_sided {
        bits |= ShaderFeatureBits::DOUBLE_SIDED;
    }
    ShaderFeatureBits::new(bits)
}

fn material_shading_model_id(
    material: &MaterialAsset,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
) -> ShadingModelId {
    let lighting_model = material.lighting_model();
    ShadingModelId::from_lighting_model(&lighting_model)
        .or_else(|| {
            shading_model_ids
                .get(&lighting_model.as_token().trim().to_ascii_lowercase())
                .copied()
        })
        .unwrap_or(SHADING_MODEL_ID_STANDARD_PBR)
}

fn material_pass_filter(material: &MaterialAsset) -> Option<Vec<ShaderPassType>> {
    matches!(material.alpha_mode, AlphaMode::Blend).then(|| ASSET_SCAN_ALPHA_BLEND_PASSES.to_vec())
}

fn material_alpha_cutoff(material: &MaterialAsset) -> Option<f32> {
    match &material.alpha_mode {
        AlphaMode::Mask { cutoff } => Some(*cutoff),
        AlphaMode::Opaque | AlphaMode::Blend => None,
    }
}

fn material_uses_builtin_standard_shader(material: &MaterialAsset) -> bool {
    material.shader.locator.to_string() == BUILTIN_STANDARD_MATERIAL_SHADER_URI
}

#[derive(Clone, Debug)]
pub(super) struct MaterialPrewarmSource {
    pub(super) stable_label: String,
    shader_label: String,
    shader_resource_id: ResourceId,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    pass_filter: Option<Vec<ShaderPassType>>,
    material_option_values: BTreeMap<String, toml::Value>,
    uses_builtin_standard_shader: bool,
}

impl MaterialPrewarmSource {
    fn apply_to_shader_source(
        &self,
        mut shader_source: ShaderPrewarmSource,
    ) -> ShaderPrewarmSource {
        if let Some(pass_filter) = &self.pass_filter {
            shader_source
                .pass_types
                .retain(|pass_type| pass_filter.contains(pass_type));
        }
        shader_source
    }
}
