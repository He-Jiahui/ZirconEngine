use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use zircon_runtime::asset::{
    AlphaMode, AssetReference, AssetRegistryIndex, MaterialAsset, ReferenceResolutionError,
    ZMaterialDocument,
};
use zircon_runtime::core::framework::render::{
    GeometrySourceDescriptor, GeometrySourceId, ShaderFeatureBits, ShaderPassType,
    ShaderPipelinePrewarmState, ShaderQualityTier, ShaderVariantPrewarmManifest, ShadingModelId,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use zircon_runtime::core::resource::ResourceId;
use zircon_runtime::dynamic_api::{
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
};

use super::paths::{has_extension, stable_label_for_path};
use super::{
    append_manifest, prewarm_manifest_for_source_with_dimensions, ShaderPrewarmAssetInventory,
    ShaderPrewarmSource, BUILTIN_STANDARD_MATERIAL_SHADER_URI,
};
use crate::error::{ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult};

const ASSET_SCAN_ALPHA_BLEND_PASSES: [ShaderPassType; 1] = [ShaderPassType::Forward];

pub(super) fn collect_material_sources(
    paths: &[PathBuf],
    asset_root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    sources: &mut Vec<MaterialPrewarmSource>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    registry: &AssetRegistryIndex,
) -> ShaderPrewarmAssetScanResult<()> {
    for path in paths {
        if has_extension(path, "zmaterial") {
            sources.push(material_source_from_zmaterial(
                asset_root,
                path,
                inventory,
                shading_model_ids,
                registry,
            )?);
        }
    }
    Ok(())
}

pub(super) fn prewarm_manifest_for_material_source(
    material: MaterialPrewarmSource,
    shader_sources: &[ShaderPrewarmSource],
    shader_source_index: &ShaderPrewarmSourceIndex,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let pipeline_state = material.pipeline_state;
    let Some(shader_source) = shader_source_index.source_for_material(
        shader_sources,
        &material.shader_label,
        material.shader_resource_id,
    ) else {
        if material.uses_builtin_standard_shader {
            let mut manifest = ShaderVariantPrewarmManifest::empty();
            for geometry_source in geometry_sources.iter().copied() {
                let mut builtin_manifest =
                    if let Some(descriptor) = geometry_source_descriptors.get(&geometry_source) {
                        builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
                            material.features,
                            material.shading_model,
                            material.alpha_cutoff,
                            descriptor,
                            quality_tiers,
                        )
                    } else {
                        builtin_standard_material_shader_prewarm_manifest_for_geometry(
                            material.features,
                            material.shading_model,
                            material.alpha_cutoff,
                            geometry_source,
                            quality_tiers,
                        )
                    };
                for request in &mut builtin_manifest.variants {
                    request.pipeline_state = Some(pipeline_state);
                }
                append_manifest(&mut manifest, builtin_manifest);
            }
            return Ok(manifest);
        }
        return Ok(ShaderVariantPrewarmManifest::empty());
    };
    if !shader_source.kind.participates_in_material_variants() {
        return Err(ShaderPrewarmAssetScanError::MaterialShaderKindMismatch {
            material_path: material.source_path,
            shader_label: shader_source.stable_label.clone(),
            actual_kind: shader_source.kind.token(),
        });
    }
    let material_layout_hash = shader_source.material_layout_hash;
    let material_option_bits = shader_source
        .material_option_table
        .bits_for_values(&material.material_option_values);
    Ok(prewarm_manifest_for_source_with_dimensions(
        shader_source,
        material.features,
        material.shading_model,
        material.alpha_cutoff,
        Some(pipeline_state),
        material.pass_filter.as_deref(),
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
        material_layout_hash,
        material_option_bits,
    ))
}

fn material_source_from_zmaterial(
    asset_root: &Path,
    material_path: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    registry: &AssetRegistryIndex,
) -> ShaderPrewarmAssetScanResult<MaterialPrewarmSource> {
    let document = inventory
        .text(material_path)
        .expect("asset inventory must retain material text");
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
        source_path: material_path.to_path_buf(),
        stable_label: stable_label_for_path(asset_root, material_path),
        shader_label: material.shader.locator.to_string(),
        shader_resource_id: ResourceId::from_asset_uuid(material.shader.uuid),
        features: material_feature_bits(&material),
        shading_model: material_shading_model_id(&material, shading_model_ids),
        alpha_cutoff: material_alpha_cutoff(&material),
        pipeline_state: material_pipeline_state(&material),
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
    if material.normal_texture.is_some() {
        bits |= ShaderFeatureBits::HAS_NORMAL_TEXTURE;
    }
    let advanced_features = material.advanced_pbr_features();
    if advanced_features.uses_clearcoat() {
        bits |= ShaderFeatureBits::PBR_CLEARCOAT;
    }
    if advanced_features.uses_anisotropy() {
        bits |= ShaderFeatureBits::PBR_ANISOTROPY;
    }
    if advanced_features.uses_transmission() {
        bits |= ShaderFeatureBits::PBR_TRANSMISSION;
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

fn material_pipeline_state(material: &MaterialAsset) -> ShaderPipelinePrewarmState {
    ShaderPipelinePrewarmState {
        alpha_blend: matches!(material.alpha_mode, AlphaMode::Blend),
        alpha_cutoff_bits: material_alpha_cutoff(material).map(f32::to_bits),
        unlit: material.lighting_model().is_unlit(),
    }
}

fn material_uses_builtin_standard_shader(material: &MaterialAsset) -> bool {
    material.shader.locator.to_string() == BUILTIN_STANDARD_MATERIAL_SHADER_URI
}

#[derive(Clone, Debug)]
pub(super) struct MaterialPrewarmSource {
    pub(super) source_path: PathBuf,
    pub(super) stable_label: String,
    shader_label: String,
    shader_resource_id: ResourceId,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    pipeline_state: ShaderPipelinePrewarmState,
    pass_filter: Option<Vec<ShaderPassType>>,
    material_option_values: BTreeMap<String, toml::Value>,
    uses_builtin_standard_shader: bool,
}

pub(super) struct ShaderPrewarmSourceIndex {
    source_index_by_label: HashMap<String, usize>,
    source_index_by_resource_id: HashMap<ResourceId, usize>,
}

impl ShaderPrewarmSourceIndex {
    pub(super) fn from_sources(sources: &[ShaderPrewarmSource]) -> Self {
        let mut source_index_by_label = HashMap::with_capacity(sources.len());
        let mut source_index_by_resource_id = HashMap::with_capacity(sources.len());
        for (index, source) in sources.iter().enumerate() {
            source_index_by_label
                .entry(source.stable_label.clone())
                .or_insert(index);
            source_index_by_resource_id
                .entry(source.resource_id)
                .or_insert(index);
        }
        Self {
            source_index_by_label,
            source_index_by_resource_id,
        }
    }

    fn source_for_material<'a>(
        &self,
        sources: &'a [ShaderPrewarmSource],
        shader_label: &str,
        shader_resource_id: ResourceId,
    ) -> Option<&'a ShaderPrewarmSource> {
        let source_index = self.source_index_for_material(shader_label, shader_resource_id)?;
        sources.get(source_index)
    }

    pub(super) fn source_is_affected_for_material(
        &self,
        material: &MaterialPrewarmSource,
        affected_source_indices: &BTreeSet<usize>,
    ) -> bool {
        self.source_index_for_material(&material.shader_label, material.shader_resource_id)
            .is_some_and(|source_index| affected_source_indices.contains(&source_index))
    }

    fn source_index_for_material(
        &self,
        shader_label: &str,
        shader_resource_id: ResourceId,
    ) -> Option<usize> {
        [
            self.source_index_by_label.get(shader_label).copied(),
            self.source_index_by_resource_id
                .get(&shader_resource_id)
                .copied(),
        ]
        .into_iter()
        .flatten()
        .min()
    }
}
