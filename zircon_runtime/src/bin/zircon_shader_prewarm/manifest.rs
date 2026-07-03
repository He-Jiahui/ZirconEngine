use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::assets::generate_material_artifact;
use zircon_runtime::asset::project::{AssetMetaDocument, AssetSourceUnit};
use zircon_runtime::asset::{
    AlphaMode, MaterialAsset, ShaderOptionAsset, ShaderTextureSlotAsset, ZShaderDocumentV2,
};
use zircon_runtime::core::framework::render::{
    GeometrySourceDescriptor, GeometrySourceId, MaterialOptionTable, ShaderAssetKind,
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShadingModelId,
    GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
};
use zircon_runtime::core::resource::{ResourceId, ResourceKind};
use zircon_runtime::dynamic_api::{
    builtin_fallback_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry,
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor,
};

mod module_dependencies;
mod pass_types;
mod paths;
pub(crate) mod permutation_registry;
pub(crate) mod resource_registry;
mod revision;

use self::module_dependencies::shader_sources_with_module_dependency_hashes;
use self::pass_types::{asset_scan_full_material_passes, asset_scan_pass_types_for_zshader};
use self::paths::{
    content_hash, has_extension, has_sidecar_zmeta, is_inside_compound_shader_source, is_zmeta,
    meta_path_for_single_source, primary_zshader_path, stable_label_for_path,
    wgsl_files_for_document,
};
use self::resource_registry::ShaderPrewarmResourceRegistryOverlay;
use self::revision::{
    asset_scan_revision_from_content_hashes, asset_scan_revision_from_source_hash,
};
use super::error::{
    ShaderPrewarmAssetScanError, ShaderPrewarmAssetScanResult, ShaderPrewarmManifestError,
    ShaderPrewarmManifestResult,
};

const BUILTIN_STANDARD_MATERIAL_SHADER_URI: &str = "builtin://shader/pbr.wgsl";
const ASSET_SCAN_TEMPLATE_REVISION: &str = "asset-scan-mesh-template-v1";
const ASSET_SCAN_NAGA_VERSION: &str = "naga-29.0.1";
const ASSET_SCAN_WGPU_VERSION: &str = "wgpu-29.0.1";
const ASSET_SCAN_PLATFORM_TOKEN: &str = "wgpu-runtime";
const ASSET_SCAN_ALPHA_BLEND_PASSES: [ShaderPassType; 1] = [ShaderPassType::Forward];

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
    let variants = geometry_sources
        .into_iter()
        .flat_map(|geometry_source| {
            if let Some(descriptor) = geometry_source_descriptors.get(&geometry_source) {
                builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
                    ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
                    SHADING_MODEL_ID_STANDARD_PBR,
                    None,
                    descriptor,
                    &quality_tiers,
                )
                .variants
            } else {
                builtin_standard_material_shader_prewarm_manifest_for_geometry(
                    ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
                    SHADING_MODEL_ID_STANDARD_PBR,
                    None,
                    geometry_source,
                    &quality_tiers,
                )
                .variants
            }
        })
        .collect::<Vec<_>>();
    ShaderVariantPrewarmManifest::new(dedupe_prewarm_requests(variants))
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
    asset_root_manifest_with_resource_registry_revisions(
        asset_root,
        quality_tiers,
        geometry_sources,
        &geometry_source_descriptors,
        shading_model_ids,
        None,
    )
}

pub(crate) fn asset_root_manifest_with_resource_registry_revisions(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    geometry_source_descriptors: &BTreeMap<GeometrySourceId, GeometrySourceDescriptor>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<ShaderVariantPrewarmManifest> {
    let geometry_sources = manifest_geometry_sources(geometry_sources);
    let mut shader_sources = Vec::new();
    collect_shader_sources(
        asset_root,
        asset_root,
        &mut shader_sources,
        resource_registry,
    )?;
    shader_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));

    let mut seen_sources = HashSet::new();
    let shader_sources = shader_sources
        .into_iter()
        .filter(|source| seen_sources.insert(source.stable_label.clone()))
        .collect::<Vec<_>>();
    let shader_sources = shader_sources_with_module_dependency_hashes(shader_sources);
    let mut variants = shader_sources
        .iter()
        .cloned()
        .flat_map(|source| prewarm_requests_for_source(source, quality_tiers, &geometry_sources))
        .collect::<Vec<_>>();

    let mut material_sources = Vec::new();
    collect_material_sources(
        asset_root,
        asset_root,
        &mut material_sources,
        shading_model_ids,
    )?;
    material_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));
    variants.extend(material_sources.into_iter().flat_map(|material| {
        prewarm_requests_for_material_source(
            material,
            &shader_sources,
            quality_tiers,
            &geometry_sources,
            geometry_source_descriptors,
        )
    }));

    Ok(ShaderVariantPrewarmManifest::new(dedupe_prewarm_requests(
        variants,
    )))
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
    base.variants.extend(extra.variants);
    Ok(base)
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
    root: &Path,
    asset_root: &Path,
    sources: &mut Vec<ShaderPrewarmSource>,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
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
            collect_shader_sources(&path, asset_root, sources, resource_registry)?;
            continue;
        }
        if is_zmeta(&path) {
            if let Some(source) = shader_source_from_zmeta(asset_root, &path, resource_registry)? {
                sources.push(source);
            }
            continue;
        }
        if is_inside_compound_shader_source(&path) {
            continue;
        }
        if has_sidecar_zmeta(&path) {
            continue;
        }
        if has_extension(&path, "zshader") {
            sources.push(shader_source_from_zshader(asset_root, &path, None)?);
        } else if has_extension(&path, "wgsl") {
            sources.push(shader_source_from_wgsl(asset_root, &path, None)?);
        }
    }
    Ok(())
}

fn collect_material_sources(
    root: &Path,
    asset_root: &Path,
    sources: &mut Vec<MaterialPrewarmSource>,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
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
            collect_material_sources(&path, asset_root, sources, shading_model_ids)?;
            continue;
        }
        if has_extension(&path, "zmaterial") {
            sources.push(material_source_from_zmaterial(
                asset_root,
                &path,
                shading_model_ids,
            )?);
        }
    }
    Ok(())
}

fn shader_source_from_zmeta(
    asset_root: &Path,
    meta_path: &Path,
    resource_registry: Option<&ShaderPrewarmResourceRegistryOverlay>,
) -> ShaderPrewarmAssetScanResult<Option<ShaderPrewarmSource>> {
    let meta = AssetMetaDocument::load(meta_path).map_err(|source| {
        ShaderPrewarmAssetScanError::LoadShaderMetadata {
            path: meta_path.to_path_buf(),
            source,
        }
    })?;
    if meta.asset_kind != ResourceKind::Shader {
        return Ok(None);
    }
    let Some(shader_source) = shader_source_path_for_meta(meta_path, &meta) else {
        return Ok(None);
    };
    let resource_id = ResourceId::from_asset_uuid(meta.uuid);
    let stable_label = meta.url.to_string();
    let revision = resource_registry
        .and_then(|registry| registry.revision_for(resource_id, &stable_label))
        .unwrap_or_else(|| asset_scan_revision_from_source_hash(&meta.source_hash));
    let metadata = Some(ShaderSourceMetadata {
        resource_id,
        stable_label,
        revision,
    });
    if has_extension(&shader_source, "zshader") {
        shader_source_from_zshader(asset_root, &shader_source, metadata).map(Some)
    } else if has_extension(&shader_source, "wgsl") {
        shader_source_from_wgsl(asset_root, &shader_source, metadata).map(Some)
    } else {
        Ok(None)
    }
}

fn shader_source_path_for_meta(meta_path: &Path, meta: &AssetMetaDocument) -> Option<PathBuf> {
    match meta.unit {
        AssetSourceUnit::Compound => {
            let file_name = meta_path.file_name()?.to_str()?;
            let dir_name = file_name.strip_suffix(".zmeta")?;
            let package_dir = meta_path.with_file_name(dir_name);
            primary_zshader_path(&package_dir)
        }
        AssetSourceUnit::Single => Some(meta_path_for_single_source(meta_path)),
    }
}

fn shader_source_from_zshader(
    asset_root: &Path,
    zshader_path: &Path,
    metadata: Option<ShaderSourceMetadata>,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSource> {
    let document = fs::read_to_string(zshader_path).map_err(|source| {
        ShaderPrewarmAssetScanError::ReadZShader {
            path: zshader_path.to_path_buf(),
            source,
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
    let wgsl_files = wgsl_files_for_document(package_dir, &document)?;
    let mut source = String::new();
    let mut include_hashes = Vec::new();
    for file in wgsl_files {
        let path = package_dir.join(&file);
        let text =
            fs::read_to_string(&path).map_err(|source| ShaderPrewarmAssetScanError::ReadWgsl {
                path: path.to_path_buf(),
                source,
            })?;
        if !source.is_empty() {
            source.push('\n');
        }
        source.push_str(&text);
        include_hashes.push(content_hash(&text));
    }
    shader_prewarm_source(
        asset_root,
        zshader_path,
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
    metadata: Option<ShaderSourceMetadata>,
) -> ShaderPrewarmAssetScanResult<ShaderPrewarmSource> {
    let source =
        fs::read_to_string(wgsl_path).map_err(|source| ShaderPrewarmAssetScanError::ReadWgsl {
            path: wgsl_path.to_path_buf(),
            source,
        })?;
    let include_hashes = vec![content_hash(&source)];
    shader_prewarm_source(
        asset_root,
        wgsl_path,
        source,
        include_hashes,
        asset_scan_full_material_passes(),
        ShaderAssetKind::Surface,
        None,
        Vec::new(),
        metadata,
        0,
        MaterialOptionTable::default(),
    )
}

fn material_source_from_zmaterial(
    asset_root: &Path,
    material_path: &Path,
    shading_model_ids: &BTreeMap<String, ShadingModelId>,
) -> ShaderPrewarmAssetScanResult<MaterialPrewarmSource> {
    let document = fs::read_to_string(material_path).map_err(|source| {
        ShaderPrewarmAssetScanError::ReadZMaterial {
            path: material_path.to_path_buf(),
            source,
        }
    })?;
    let material = MaterialAsset::from_toml_str(&document).map_err(|source| {
        ShaderPrewarmAssetScanError::ParseZMaterial {
            path: material_path.to_path_buf(),
            source,
        }
    })?;
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

fn prewarm_requests_for_source(
    source: ShaderPrewarmSource,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
) -> Vec<ShaderVariantPrewarmRequest> {
    let material_layout_hash = source.material_layout_hash;
    let material_option_bits = source.material_option_table.default_bits();
    prewarm_requests_for_source_with_dimensions(
        source,
        ShaderFeatureBits::new(0),
        SHADING_MODEL_ID_STANDARD_PBR,
        quality_tiers,
        geometry_sources,
        material_layout_hash,
        material_option_bits,
    )
}

fn prewarm_requests_for_material_source(
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
        quality_tiers,
        geometry_sources,
        material_layout_hash,
        material_option_bits,
    )
}

fn prewarm_requests_for_source_with_dimensions(
    source: ShaderPrewarmSource,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    quality_tiers: &[ShaderQualityTier],
    geometry_sources: &[GeometrySourceId],
    material_layout_hash: u64,
    material_option_bits: u32,
) -> Vec<ShaderVariantPrewarmRequest> {
    let ShaderPrewarmSource {
        stable_label,
        resource_id,
        revision,
        wgsl_source,
        include_content_hashes,
        pass_types,
        kind: _,
        import_path: _,
        imports: _,
        material_layout_hash: _,
        material_option_table: _,
    } = source;
    let quality_tiers = manifest_quality_tiers(quality_tiers);
    let geometry_sources = manifest_geometry_sources(geometry_sources);

    let mut requests = Vec::new();
    for pass_type in pass_types {
        for quality in &quality_tiers {
            for geometry_source in &geometry_sources {
                requests.push(ShaderVariantPrewarmRequest {
                    key: ShaderVariantKey {
                        material_shader: resource_id,
                        material_revision: revision,
                        material_layout_hash,
                        material_option_bits,
                        geometry_source: *geometry_source,
                        shading_model,
                        pass_type,
                        features,
                        quality: *quality,
                        platform_token: ASSET_SCAN_PLATFORM_TOKEN.to_string(),
                    },
                    source_label: stable_label.clone(),
                    wgsl_source: wgsl_source.clone(),
                    include_content_hashes: include_content_hashes.clone(),
                    template_revision: ASSET_SCAN_TEMPLATE_REVISION.to_string(),
                    naga_version: ASSET_SCAN_NAGA_VERSION.to_string(),
                    wgpu_version: ASSET_SCAN_WGPU_VERSION.to_string(),
                });
            }
        }
    }
    requests
}

fn dedupe_prewarm_requests(
    variants: Vec<ShaderVariantPrewarmRequest>,
) -> Vec<ShaderVariantPrewarmRequest> {
    let mut seen = HashSet::new();
    variants
        .into_iter()
        .filter(|request| seen.insert(request.key.canonical_string()))
        .collect()
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
struct ShaderPrewarmSource {
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

#[derive(Clone, Debug)]
struct ShaderSourceMetadata {
    resource_id: ResourceId,
    stable_label: String,
    revision: u64,
}

#[derive(Clone, Debug)]
struct MaterialPrewarmSource {
    stable_label: String,
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

#[cfg(test)]
mod tests;
