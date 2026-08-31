use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::assets::generate_material_artifact;
use zircon_runtime::asset::project::{AssetMetaDocument, AssetSourceUnit};
use zircon_runtime::asset::{
    AssetRegistryIndex, ShaderOptionAsset, ShaderTextureSlotAsset, ZShaderDocumentV2,
};
use zircon_runtime::core::framework::render::{
    GeometrySourceDescriptor, GeometrySourceId, MaterialOptionTable, ShaderAssetKind,
    ShaderFeatureBits, ShaderPassType, ShaderPipelinePrewarmState, ShaderQualityTier,
    ShaderVariantKey, ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest,
    ShaderVariantPrewarmSource, ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADER_VARIANT_CACHE_NAGA_VERSION, SHADER_VARIANT_CACHE_WGPU_VERSION,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use zircon_runtime::core::resource::{ResourceId, ResourceKind};
use zircon_runtime::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
    material_surface_shader_prewarm_template_source, ShaderPrewarmTemplateSource,
};

mod asset_inventory;
mod material_sources;
mod module_dependencies;
mod pass_types;
mod paths;
pub(crate) mod permutation_registry;
pub(crate) mod resource_registry;
mod revision;

pub(crate) use self::asset_inventory::ShaderPrewarmAssetInventory;
use self::material_sources::{
    collect_material_sources, prewarm_manifest_for_material_source, ShaderPrewarmSourceIndex,
};
use self::module_dependencies::shader_sources_with_module_dependency_hashes_and_changed_paths;
use self::pass_types::asset_scan_pass_types_for_zshader;
use self::paths::{
    content_hash, has_extension, has_sidecar_zmeta, is_inside_compound_shader_source, is_zmeta,
    meta_path_for_single_source, primary_zshader_path, stable_label_for_path,
    wgsl_files_for_document,
};
use self::resource_registry::ShaderPrewarmResourceRegistryOverlay;
use self::revision::{
    asset_scan_revision_from_content_hashes, asset_scan_revision_from_source_digest,
};
use super::error::{
    ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult, ShaderPrewarmManifestError,
    ShaderPrewarmManifestResult,
};

const BUILTIN_STANDARD_MATERIAL_SHADER_URI: &str = "builtin://shader/pbr.wgsl";
const ASSET_SCAN_TEMPLATE_REVISION: &str = "asset-scan-mesh-template-v1";
const ASSET_SCAN_PLATFORM_TOKEN: &str = "wgpu-runtime";

pub fn read_manifest(path: &Path) -> ShaderPrewarmManifestResult<ShaderVariantPrewarmManifest> {
    let bytes = fs::read(path).map_err(|source| ShaderPrewarmManifestError::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| ShaderPrewarmManifestError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

pub fn builtin_fallback_manifest() -> ShaderVariantPrewarmManifest {
    builtin_fallback_shader_prewarm_manifest()
}

pub fn builtin_fallback_manifest_for_quality_tiers(
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let geometry_source_descriptors = BTreeMap::new();
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
        quality_tiers,
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &geometry_source_descriptors,
    )
}

pub fn builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
) -> ShaderVariantPrewarmManifest {
    builtin_fallback_manifest_for_quality_tiers_geometry_sources_and_descriptors(
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
    )
}

pub fn builtin_fallback_manifest_for_quality_tiers_geometry_sources_and_descriptors(
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
) -> ShaderVariantPrewarmManifest {
    let quality_tiers = manifest_quality_tiers(quality_tiers);
    let geometry_sources = manifest_geometry_sources(geometry_sources);
    let mut manifest = ShaderVariantPrewarmManifest::empty();
    for geometry_source in geometry_sources {
        let source_manifest =
            if let Some(descriptor) = geometry_source_descriptors.get(&geometry_source) {
                builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
                    ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
                    SHADING_MODEL_ID_STANDARD_PBR,
                    None,
                    descriptor,
                    &quality_tiers,
                )
            } else {
                builtin_standard_material_shader_prewarm_manifest_for_geometry(
                    ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
                    SHADING_MODEL_ID_STANDARD_PBR,
                    None,
                    geometry_source,
                    &quality_tiers,
                )
            };
        append_manifest(&mut manifest, source_manifest);
    }
    dedupe_prewarm_manifest(manifest)
}

#[cfg(test)]
pub fn asset_root_manifest(
    asset_root: &Path,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    asset_root_manifest_for_quality_tiers(asset_root, &[ShaderQualityTier::Medium])
}

pub fn asset_root_manifest_for_quality_tiers(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    asset_root_manifest_for_quality_tiers_and_geometry_sources(
        asset_root,
        quality_tiers,
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
    )
}

pub fn asset_root_manifest_for_quality_tiers_and_geometry_sources(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let shading_model_ids = BTreeMap::new();
    asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids(
        asset_root,
        quality_tiers,
        geometry_sources,
        &shading_model_ids,
    )
}

pub fn asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let geometry_source_descriptors = BTreeMap::new();
    let shader_modules = BTreeMap::new();
    asset_root_manifest_with_resource_registry_revisions(
        asset_root,
        quality_tiers,
        geometry_sources,
        &geometry_source_descriptors,
        shading_model_ids,
        &shader_modules,
        None,
    )
}

pub(crate) fn asset_root_manifest_with_resource_registry_revisions(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    shader_modules: &BTreeMap<String, String>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let inventory = ShaderPrewarmAssetInventory::collect(asset_root)?;
    asset_root_manifest_from_inventory_with_resource_registry_revisions(
        asset_root,
        &inventory,
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
        shading_model_ids,
        shader_modules,
        resource_registry,
    )
}

pub(crate) fn asset_root_manifest_from_inventory_with_resource_registry_revisions(
    asset_root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    shader_modules: &BTreeMap<String, String>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(
        asset_root,
        inventory,
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
        shading_model_ids,
        shader_modules,
        resource_registry,
        false,
    )
}

/// External permutation inputs do not belong to an asset-inventory snapshot.
/// Callers must request a complete projection when those inputs can change
/// independently of the asset-root fingerprint.
pub(crate) fn asset_root_manifest_from_inventory_with_resource_registry_revisions_and_external_inputs(
    asset_root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    shader_modules: &BTreeMap<String, String>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
    has_external_permutation_inputs: bool,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let geometry_sources = manifest_geometry_sources(geometry_sources);
    let registry = AssetRegistryIndex::inspect_loaded_meta_documents(inventory.metadata_by_path())
        .map_err(|source| ShaderPrewarmAssetScanError::InspectAssetRegistry {
            path: asset_root.to_path_buf(),
            source,
        })?;
    let mut shader_sources = Vec::new();
    collect_shader_sources(
        inventory.paths(),
        asset_root,
        inventory,
        &mut shader_sources,
        resource_registry,
    )?;
    shader_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));

    let mut seen_sources = HashSet::new();
    let shader_sources = shader_sources
        .into_iter()
        .filter(|source| seen_sources.insert(source.stable_label.clone()))
        .collect::<Vec<_>>();
    let dependency_batch = shader_sources_with_module_dependency_hashes_and_changed_paths(
        shader_sources,
        shader_modules,
        inventory.changed_paths(),
    )?;
    let shader_sources = dependency_batch.sources;
    let affected_source_indices = if !has_external_permutation_inputs
        && shader_modules.is_empty()
        && resource_registry.is_none()
    {
        dependency_batch.affected_source_indices
    } else {
        (0..shader_sources.len()).collect()
    };
    let shader_source_index = ShaderPrewarmSourceIndex::from_sources(&shader_sources);
    let mut manifest = ShaderVariantPrewarmManifest::empty();
    for (source_index, source) in shader_sources.iter().enumerate() {
        if !affected_source_indices.contains(&source_index) {
            continue;
        }
        append_manifest(
            &mut manifest,
            prewarm_manifest_for_source(
                source,
                quality_tiers,
                &geometry_sources,
                geometry_source_descriptors,
            ),
        );
    }

    let mut material_sources = Vec::new();
    collect_material_sources(
        inventory.paths(),
        asset_root,
        inventory,
        &mut material_sources,
        shading_model_ids,
        &registry,
    )?;
    material_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));
    for material in material_sources {
        if !inventory.changed_paths().contains(&material.source_path)
            && !shader_source_index
                .source_is_affected_for_material(&material, &affected_source_indices)
        {
            continue;
        }
        append_manifest(
            &mut manifest,
            prewarm_manifest_for_material_source(
                material,
                &shader_sources,
                &shader_source_index,
                quality_tiers,
                &geometry_sources,
                geometry_source_descriptors,
            )?,
        );
    }

    Ok(dedupe_prewarm_manifest(manifest))
}

pub fn merge_manifests(
    mut base: ShaderVariantPrewarmManifest,
    extra: ShaderVariantPrewarmManifest,
) -> ShaderPrewarmManifestResult<ShaderVariantPrewarmManifest> {
    if base.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        return Err(ShaderPrewarmManifestError::UnsupportedSchema {
            actual: base.schema_version,
            expected: ShaderVariantPrewarmManifest::SCHEMA_VERSION,
        });
    }
    if extra.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        return Err(ShaderPrewarmManifestError::UnsupportedSchema {
            actual: extra.schema_version,
            expected: ShaderVariantPrewarmManifest::SCHEMA_VERSION,
        });
    }
    base.validate_integrity()
        .map_err(|source| ShaderPrewarmManifestError::InvalidSourceTable { source })?;
    extra
        .validate_integrity()
        .map_err(|source| ShaderPrewarmManifestError::InvalidSourceTable { source })?;
    append_manifest(&mut base, extra);
    Ok(dedupe_prewarm_manifest(base))
}

fn manifest_quality_tiers(quality_tiers: &[ShaderQualityTier]) -> Vec<ShaderQualityTier> {
    if quality_tiers.is_empty() {
        return vec![ShaderQualityTier::Medium];
    }
    let mut seen = HashSet::new();
    quality_tiers
        .iter()
        .copied()
        .filter(|quality| seen.insert(*quality))
        .collect()
}

fn manifest_geometry_sources(geometry_sources: &[GeometrySourceId]) -> Vec<GeometrySourceId> {
    if geometry_sources.is_empty() {
        return vec![GEOMETRY_SOURCE_ID_STATIC_MESH];
    }
    let mut seen = HashSet::new();
    geometry_sources
        .iter()
        .copied()
        .filter(|geometry_source| seen.insert(*geometry_source))
        .collect()
}

fn collect_shader_sources(
    paths: &[PathBuf],
    asset_root: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    sources: &mut Vec<ShaderPrewarmSource>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<()> {
    for path in paths {
        if is_zmeta(path) {
            if let Some(source) =
                shader_source_from_zmeta(asset_root, path, inventory, resource_registry)?
            {
                sources.push(source);
            }
            continue;
        }
        if is_inside_compound_shader_source(path, inventory.paths()) {
            continue;
        }
        if has_sidecar_zmeta(path, inventory.paths()) {
            continue;
        }
        if has_extension(path, "zshader") {
            sources.push(shader_source_from_zshader(
                asset_root, path, inventory, None,
            )?);
        } else if has_extension(path, "wgsl") {
            sources.push(shader_source_from_wgsl(asset_root, path, inventory, None)?);
        }
    }
    Ok(())
}

fn shader_source_from_zmeta(
    asset_root: &Path,
    meta_path: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<Option<ShaderPrewarmSource>> {
    let meta = inventory.metadata(meta_path).ok_or_else(|| {
        ShaderPrewarmAssetScanError::MissingAssetInventoryEntry {
            path: meta_path.to_path_buf(),
            entry_kind: "metadata",
        }
    })?;
    if meta.asset_kind != ResourceKind::Shader {
        return Ok(None);
    }
    let Some(shader_source) = shader_source_path_for_meta(meta_path, &meta, inventory.paths())
    else {
        return Ok(None);
    };
    let resource_id = ResourceId::from_asset_uuid(meta.uuid);
    let stable_label = meta.url.to_string();
    let revision = resource_registry
        .and_then(|registry| registry.revision_for(resource_id, &stable_label))
        .unwrap_or_else(|| asset_scan_revision_from_source_digest(&meta.source_digest));
    let metadata = Some(ShaderSourceMetadata {
        resource_id,
        stable_label,
        revision,
    });
    let source = if has_extension(&shader_source, "zshader") {
        shader_source_from_zshader(asset_root, &shader_source, inventory, metadata)
    } else if has_extension(&shader_source, "wgsl") {
        shader_source_from_wgsl(asset_root, &shader_source, inventory, metadata)
    } else {
        return Ok(None);
    }?;
    Ok(Some(source.with_input_path(meta_path.to_path_buf())))
}

fn shader_source_path_for_meta(
    meta_path: &Path,
    meta: &AssetMetaDocument,
    inventory_paths: &[PathBuf],
) -> Option<PathBuf> {
    match meta.unit {
        AssetSourceUnit::Compound => {
            let file_name = meta_path.file_name()?.to_str()?;
            let dir_name = file_name.strip_suffix(".zmeta")?;
            let package_dir = meta_path.with_file_name(dir_name);
            primary_zshader_path(&package_dir, inventory_paths)
        }
        AssetSourceUnit::Single => Some(meta_path_for_single_source(meta_path)),
    }
}

fn shader_source_from_zshader(
    asset_root: &Path,
    zshader_path: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    metadata: Option<ShaderSourceMetadata>,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSource> {
    let document = inventory.text(zshader_path).ok_or_else(|| {
        ShaderPrewarmAssetScanError::MissingAssetInventoryEntry {
            path: zshader_path.to_path_buf(),
            entry_kind: "zshader text",
        }
    })?;
    let document = ZShaderDocumentV2::from_toml_str(&document).map_err(|source| {
        ShaderPrewarmAssetScanError::ParseZShader {
            path: zshader_path.to_path_buf(),
            source,
        }
    })?;
    let kind = document.kind();
    let import_path = document.import_path().map(str::to_string);
    let imports = document
        .imports()
        .iter()
        .map(|import| import.source.clone())
        .collect::<Vec<_>>();
    let pass_types = asset_scan_pass_types_for_zshader(&document);
    let (material_layout_hash, material_option_table) = material_signature_for_zshader(&document);
    let package_dir = zshader_path.parent().unwrap_or_else(|| Path::new(""));
    let wgsl_files = wgsl_files_for_document(package_dir, &document, inventory.paths())?;
    let mut source = String::new();
    let mut include_hashes = Vec::new();
    let mut source_paths = vec![zshader_path.to_path_buf()];
    for file in wgsl_files {
        let path = package_dir.join(&file);
        let text = inventory.text(&path).ok_or_else(|| {
            ShaderPrewarmAssetScanError::MissingAssetInventoryEntry {
                path: path.clone(),
                entry_kind: "WGSL text",
            }
        })?;
        if !source.is_empty() {
            source.push('\n');
        }
        source.push_str(text);
        include_hashes.push(content_hash(text));
        source_paths.push(path);
    }
    shader_prewarm_source(
        asset_root,
        zshader_path,
        source_paths,
        source,
        include_hashes,
        pass_types,
        kind,
        import_path,
        imports,
        metadata,
        material_layout_hash,
        material_option_table,
    )
}

fn shader_source_from_wgsl(
    asset_root: &Path,
    wgsl_path: &Path,
    inventory: &ShaderPrewarmAssetInventory,
    metadata: Option<ShaderSourceMetadata>,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSource> {
    let source = inventory
        .text(wgsl_path)
        .ok_or_else(|| ShaderPrewarmAssetScanError::MissingAssetInventoryEntry {
            path: wgsl_path.to_path_buf(),
            entry_kind: "WGSL text",
        })?
        .to_string();
    let include_hashes = vec![content_hash(&source)];
    shader_prewarm_source(
        asset_root,
        wgsl_path,
        vec![wgsl_path.to_path_buf()],
        source,
        include_hashes,
        Vec::new(),
        ShaderAssetKind::Module,
        None,
        Vec::new(),
        metadata,
        0,
        MaterialOptionTable::default(),
    )
}

fn material_signature_for_zshader(document: &ZShaderDocumentV2) -> (u64, MaterialOptionTable) {
    let options = document
        .options()
        .iter()
        .map(ShaderOptionAsset::from)
        .collect::<Vec<_>>();
    let texture_slots = document
        .texture_slots()
        .iter()
        .map(ShaderTextureSlotAsset::from)
        .collect::<Vec<_>>();
    let generated = generate_material_artifact(document.properties(), &options, &texture_slots);
    (
        generated.property_layout.layout_hash,
        generated.option_table,
    )
}

fn shader_prewarm_source(
    asset_root: &Path,
    source_path: &Path,
    source_paths: Vec<PathBuf>,
    source: String,
    include_hashes: Vec<String>,
    pass_types: Vec<ShaderPassType>,
    kind: ShaderAssetKind,
    import_path: Option<String>,
    imports: Vec<String>,
    metadata: Option<ShaderSourceMetadata>,
    material_layout_hash: u64,
    material_option_table: MaterialOptionTable,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSource> {
    if source.trim().is_empty() {
        return Err(ShaderPrewarmAssetScanError::EmptyShaderSource {
            path: source_path.to_path_buf(),
        });
    }
    let fallback_label = stable_label_for_path(asset_root, source_path);
    let metadata = metadata.unwrap_or_else(|| ShaderSourceMetadata {
        resource_id: ResourceId::from_stable_label(&fallback_label),
        stable_label: fallback_label,
        revision: asset_scan_revision_from_content_hashes(&include_hashes),
    });
    Ok(ShaderPrewarmSource {
        source_paths,
        stable_label: metadata.stable_label,
        resource_id: metadata.resource_id,
        revision: metadata.revision,
        wgsl_source: source,
        include_content_hashes: include_hashes,
        pass_types,
        kind,
        import_path,
        imports,
        material_layout_hash,
        material_option_table,
    })
}

fn prewarm_manifest_for_source(
    source: &ShaderPrewarmSource,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
) -> ShaderVariantPrewarmManifest {
    if !source.kind.participates_in_material_variants() {
        return ShaderVariantPrewarmManifest::empty();
    }
    let material_layout_hash = source.material_layout_hash;
    let material_option_bits = source.material_option_table.default_bits();
    prewarm_manifest_for_source_with_dimensions(
        source,
        ShaderFeatureBits::new(0),
        SHADING_MODEL_ID_STANDARD_PBR,
        None,
        Some(ShaderPipelinePrewarmState::default()),
        None,
        quality_tiers,
        geometry_sources,
        geometry_source_descriptors,
        material_layout_hash,
        material_option_bits,
    )
}

fn prewarm_manifest_for_source_with_dimensions(
    source: &ShaderPrewarmSource,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    pipeline_state: Option<ShaderPipelinePrewarmState>,
    allowed_pass_types: Option<&[ShaderPassType]>,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    material_layout_hash: u64,
    material_option_bits: u32,
) -> ShaderVariantPrewarmManifest {
    let quality_tiers = manifest_quality_tiers(quality_tiers);
    let geometry_sources = manifest_geometry_sources(geometry_sources);

    let mut sources = Vec::new();
    let mut requests = Vec::new();
    for pass_type in
        source.pass_types.iter().copied().filter(|pass_type| {
            allowed_pass_types.is_none_or(|allowed| allowed.contains(pass_type))
        })
    {
        for geometry_source in &geometry_sources {
            let template_source = asset_scan_template_source_for_request(
                &source.wgsl_source,
                pass_type,
                *geometry_source,
                geometry_source_descriptors,
                features,
                alpha_cutoff,
                &source.include_content_hashes,
            );
            let (wgsl_source, include_content_hashes, template_revision) = match template_source {
                Some(source) => (
                    source.wgsl_source,
                    source.include_content_hashes,
                    source.template_revision,
                ),
                None => (
                    source.wgsl_source.clone(),
                    source.include_content_hashes.clone(),
                    ASSET_SCAN_TEMPLATE_REVISION.to_string(),
                ),
            };
            let prewarm_source = ShaderVariantPrewarmSource::new(
                source.stable_label.clone(),
                wgsl_source,
                include_content_hashes,
                template_revision,
                SHADER_VARIANT_CACHE_NAGA_VERSION,
                SHADER_VARIANT_CACHE_WGPU_VERSION,
            );
            for quality in &quality_tiers {
                requests.push(ShaderVariantPrewarmRequest {
                    key: ShaderVariantKey {
                        material_shader: source.resource_id,
                        material_revision: source.revision,
                        material_layout_hash,
                        material_option_bits,
                        geometry_source: *geometry_source,
                        shading_model,
                        pass_type,
                        features,
                        quality: *quality,
                        platform_token: ASSET_SCAN_PLATFORM_TOKEN.to_string(),
                    },
                    pipeline_state,
                    source_id: prewarm_source.id.clone(),
                });
            }
            sources.push(prewarm_source);
        }
    }
    ShaderVariantPrewarmManifest::new(sources, requests)
}

fn asset_scan_template_source_for_request(
    wgsl_source: &str,
    pass_type: ShaderPassType,
    geometry_source: GeometrySourceId,
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    features: ShaderFeatureBits,
    alpha_cutoff: Option<f32>,
    include_content_hashes: &[String],
) -> Option<ShaderPrewarmTemplateSource> {
    material_surface_shader_prewarm_template_source(
        wgsl_source,
        pass_type,
        geometry_source,
        geometry_source_descriptors.get(&geometry_source),
        features,
        alpha_cutoff,
        include_content_hashes,
    )
    .ok()
}

fn append_manifest(base: &mut ShaderVariantPrewarmManifest, extra: ShaderVariantPrewarmManifest) {
    base.sources.extend(extra.sources);
    base.variants.extend(extra.variants);
}

fn dedupe_prewarm_manifest(
    mut manifest: ShaderVariantPrewarmManifest,
) -> ShaderVariantPrewarmManifest {
    let mut source_ids = HashSet::new();
    manifest
        .sources
        .retain(|source| source_ids.insert(source.id.clone()));
    let mut variants = HashSet::new();
    manifest.variants.retain(|request| {
        variants.insert((
            request.key.canonical_string(),
            request.pipeline_state,
            request.source_id.clone(),
        ))
    });
    manifest
}

#[derive(Clone, Debug)]
struct ShaderPrewarmSource {
    source_paths: Vec<PathBuf>,
    stable_label: String,
    resource_id: ResourceId,
    revision: u64,
    wgsl_source: String,
    include_content_hashes: Vec<String>,
    pass_types: Vec<ShaderPassType>,
    kind: ShaderAssetKind,
    import_path: Option<String>,
    imports: Vec<String>,
    material_layout_hash: u64,
    material_option_table: MaterialOptionTable,
}

impl ShaderPrewarmSource {
    fn with_input_path(mut self, path: PathBuf) -> Self {
        if !self.source_paths.contains(&path) {
            self.source_paths.push(path);
        }
        self
    }
}

#[derive(Clone, Debug)]
struct ShaderSourceMetadata {
    resource_id: ResourceId,
    stable_label: String,
    revision: u64,
}

#[cfg(test)]
mod tests;
