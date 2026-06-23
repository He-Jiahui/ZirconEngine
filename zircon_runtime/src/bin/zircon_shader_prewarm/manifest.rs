use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::{AssetMetaDocument, AssetSourceUnit};
use zircon_runtime::asset::{AlphaMode, MaterialAsset, ZShaderDocument};
use zircon_runtime::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShadingModelId,
    GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
};
use zircon_runtime::core::resource::{ResourceId, ResourceKind};
use zircon_runtime::dynamic_api::builtin_fallback_shader_prewarm_manifest;

const ASSET_SCAN_TEMPLATE_REVISION: &str = "asset-scan-mesh-template-v1";
const ASSET_SCAN_NAGA_VERSION: &str = "naga-29.0.1";
const ASSET_SCAN_WGPU_VERSION: &str = "wgpu-29.0.1";
const ASSET_SCAN_PLATFORM_TOKEN: &str = "wgpu-runtime";
const ASSET_SCAN_INITIAL_RESOURCE_REVISION: u64 = 1;
const ASSET_SCAN_FULL_MATERIAL_PASSES: [ShaderPassType; 5] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
];
const ASSET_SCAN_VERTEX_ONLY_PASSES: [ShaderPassType; 3] = [
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
];
const ASSET_SCAN_FRAGMENT_ONLY_PASSES: [ShaderPassType; 2] =
    [ShaderPassType::Forward, ShaderPassType::GBuffer];
const ASSET_SCAN_ALPHA_BLEND_PASSES: [ShaderPassType; 1] = [ShaderPassType::Forward];

pub fn read_manifest(path: &Path) -> Result<ShaderVariantPrewarmManifest, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read shader prewarm manifest {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse shader prewarm manifest {}: {error}",
            path.display()
        )
    })
}

pub fn builtin_fallback_manifest() -> ShaderVariantPrewarmManifest {
    builtin_fallback_shader_prewarm_manifest()
}

pub fn builtin_fallback_manifest_for_quality_tiers(
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    expand_quality_tiers(builtin_fallback_manifest(), quality_tiers)
}

#[cfg(test)]
pub fn asset_root_manifest(asset_root: &Path) -> Result<ShaderVariantPrewarmManifest, String> {
    asset_root_manifest_for_quality_tiers(asset_root, &[ShaderQualityTier::Medium])
}

pub fn asset_root_manifest_for_quality_tiers(
    asset_root: &Path,
    quality_tiers: &[ShaderQualityTier],
) -> Result<ShaderVariantPrewarmManifest, String> {
    let mut shader_sources = Vec::new();
    collect_shader_sources(asset_root, asset_root, &mut shader_sources)?;
    shader_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));

    let mut seen_sources = HashSet::new();
    let shader_sources = shader_sources
        .into_iter()
        .filter(|source| seen_sources.insert(source.stable_label.clone()))
        .collect::<Vec<_>>();
    let mut variants = shader_sources
        .iter()
        .cloned()
        .flat_map(|source| prewarm_requests_for_source(source, quality_tiers))
        .collect::<Vec<_>>();

    let mut material_sources = Vec::new();
    collect_material_sources(asset_root, asset_root, &mut material_sources)?;
    material_sources.sort_by(|left, right| left.stable_label.cmp(&right.stable_label));
    variants.extend(material_sources.into_iter().flat_map(|material| {
        prewarm_requests_for_material_source(material, &shader_sources, quality_tiers)
    }));

    Ok(ShaderVariantPrewarmManifest::new(dedupe_prewarm_requests(
        variants,
    )))
}

pub fn merge_manifests(
    mut base: ShaderVariantPrewarmManifest,
    extra: ShaderVariantPrewarmManifest,
) -> Result<ShaderVariantPrewarmManifest, String> {
    if base.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        return Err(format!(
            "shader prewarm manifest schema {} is not supported; expected {}",
            base.schema_version,
            ShaderVariantPrewarmManifest::SCHEMA_VERSION
        ));
    }
    if extra.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        return Err(format!(
            "shader prewarm manifest schema {} is not supported; expected {}",
            extra.schema_version,
            ShaderVariantPrewarmManifest::SCHEMA_VERSION
        ));
    }
    base.variants.extend(extra.variants);
    Ok(base)
}

fn expand_quality_tiers(
    manifest: ShaderVariantPrewarmManifest,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let quality_tiers = manifest_quality_tiers(quality_tiers);
    let variants = manifest
        .variants
        .into_iter()
        .flat_map(|request| {
            quality_tiers.iter().copied().map(move |quality| {
                let mut request = request.clone();
                request.key.quality = quality;
                request
            })
        })
        .collect::<Vec<_>>();
    ShaderVariantPrewarmManifest::new(dedupe_prewarm_requests(variants))
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

fn collect_shader_sources(
    root: &Path,
    asset_root: &Path,
    sources: &mut Vec<ShaderPrewarmSource>,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read asset root {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read asset root {} entry: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_shader_sources(&path, asset_root, sources)?;
            continue;
        }
        if is_zmeta(&path) {
            if let Some(source) = shader_source_from_zmeta(asset_root, &path)? {
                sources.push(source);
            }
            continue;
        }
        if is_inside_compound_shader_source(&path) {
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
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read asset root {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read asset root {} entry: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_material_sources(&path, asset_root, sources)?;
            continue;
        }
        if has_extension(&path, "zmaterial") {
            sources.push(material_source_from_zmaterial(asset_root, &path)?);
        }
    }
    Ok(())
}

fn shader_source_from_zmeta(
    asset_root: &Path,
    meta_path: &Path,
) -> Result<Option<ShaderPrewarmSource>, String> {
    let meta = match AssetMetaDocument::load(meta_path) {
        Ok(meta) => meta,
        Err(error) => {
            return Err(format!(
                "failed to load shader asset metadata {}: {error}",
                meta_path.display()
            ))
        }
    };
    if meta.asset_kind != ResourceKind::Shader {
        return Ok(None);
    }
    let Some(shader_source) = shader_source_path_for_meta(meta_path, &meta) else {
        return Ok(None);
    };
    let metadata = Some(ShaderSourceMetadata {
        resource_id: ResourceId::from_asset_uuid(meta.uuid),
        stable_label: meta.url.to_string(),
        revision: ASSET_SCAN_INITIAL_RESOURCE_REVISION,
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
) -> Result<ShaderPrewarmSource, String> {
    let document = fs::read_to_string(zshader_path)
        .map_err(|error| format!("failed to read zshader {}: {error}", zshader_path.display()))?;
    let document = ZShaderDocument::from_toml_str(&document).map_err(|error| {
        format!(
            "failed to parse zshader {}: {error}",
            zshader_path.display()
        )
    })?;
    let pass_types = asset_scan_pass_types_for_zshader(&document);
    let package_dir = zshader_path.parent().unwrap_or_else(|| Path::new(""));
    let wgsl_files = wgsl_files_for_document(package_dir, &document)?;
    let mut source = String::new();
    let mut include_hashes = Vec::new();
    for file in wgsl_files {
        let path = package_dir.join(&file);
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read WGSL {}: {error}", path.display()))?;
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
        metadata,
    )
}

fn shader_source_from_wgsl(
    asset_root: &Path,
    wgsl_path: &Path,
    metadata: Option<ShaderSourceMetadata>,
) -> Result<ShaderPrewarmSource, String> {
    let source = fs::read_to_string(wgsl_path)
        .map_err(|error| format!("failed to read WGSL {}: {error}", wgsl_path.display()))?;
    let include_hashes = vec![content_hash(&source)];
    shader_prewarm_source(
        asset_root,
        wgsl_path,
        source,
        include_hashes,
        ASSET_SCAN_FULL_MATERIAL_PASSES.to_vec(),
        metadata,
    )
}

fn material_source_from_zmaterial(
    asset_root: &Path,
    material_path: &Path,
) -> Result<MaterialPrewarmSource, String> {
    let document = fs::read_to_string(material_path).map_err(|error| {
        format!(
            "failed to read zmaterial {}: {error}",
            material_path.display()
        )
    })?;
    let material = MaterialAsset::from_toml_str(&document).map_err(|error| {
        format!(
            "failed to parse zmaterial {}: {error}",
            material_path.display()
        )
    })?;
    Ok(MaterialPrewarmSource {
        stable_label: stable_label_for_path(asset_root, material_path),
        shader_label: material.shader.locator.to_string(),
        shader_resource_id: ResourceId::from_asset_uuid(material.shader.uuid),
        features: material_feature_bits(&material),
        shading_model: material_shading_model_id(&material),
        pass_filter: material_pass_filter(&material),
    })
}

fn shader_prewarm_source(
    asset_root: &Path,
    source_path: &Path,
    source: String,
    include_hashes: Vec<String>,
    pass_types: Vec<ShaderPassType>,
    metadata: Option<ShaderSourceMetadata>,
) -> Result<ShaderPrewarmSource, String> {
    if source.trim().is_empty() {
        return Err(format!(
            "shader source {} has no runtime WGSL payload",
            source_path.display()
        ));
    }
    let fallback_label = stable_label_for_path(asset_root, source_path);
    let metadata = metadata.unwrap_or_else(|| ShaderSourceMetadata {
        resource_id: ResourceId::from_stable_label(&fallback_label),
        stable_label: fallback_label,
        revision: ASSET_SCAN_INITIAL_RESOURCE_REVISION,
    });
    Ok(ShaderPrewarmSource {
        stable_label: metadata.stable_label,
        resource_id: metadata.resource_id,
        revision: metadata.revision,
        wgsl_source: source,
        include_content_hashes: include_hashes,
        pass_types,
    })
}

fn prewarm_requests_for_source(
    source: ShaderPrewarmSource,
    quality_tiers: &[ShaderQualityTier],
) -> Vec<ShaderVariantPrewarmRequest> {
    prewarm_requests_for_source_with_dimensions(
        source,
        ShaderFeatureBits::new(0),
        SHADING_MODEL_ID_STANDARD_PBR,
        quality_tiers,
    )
}

fn prewarm_requests_for_material_source(
    material: MaterialPrewarmSource,
    shader_sources: &[ShaderPrewarmSource],
    quality_tiers: &[ShaderQualityTier],
) -> Vec<ShaderVariantPrewarmRequest> {
    let Some(shader_source) = shader_sources
        .iter()
        .find(|source| {
            source.stable_label == material.shader_label
                || source.resource_id == material.shader_resource_id
        })
        .cloned()
    else {
        return Vec::new();
    };
    let shader_source = material.apply_to_shader_source(shader_source);
    prewarm_requests_for_source_with_dimensions(
        shader_source,
        material.features,
        material.shading_model,
        quality_tiers,
    )
}

fn prewarm_requests_for_source_with_dimensions(
    source: ShaderPrewarmSource,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    quality_tiers: &[ShaderQualityTier],
) -> Vec<ShaderVariantPrewarmRequest> {
    let ShaderPrewarmSource {
        stable_label: _,
        resource_id,
        revision,
        wgsl_source,
        include_content_hashes,
        pass_types,
    } = source;
    let quality_tiers = manifest_quality_tiers(quality_tiers);

    pass_types
        .into_iter()
        .flat_map(|pass_type| {
            quality_tiers.iter().copied().map({
                let wgsl_source = wgsl_source.clone();
                let include_content_hashes = include_content_hashes.clone();
                move |quality| ShaderVariantPrewarmRequest {
                    key: ShaderVariantKey {
                        material_shader: resource_id,
                        material_revision: revision,
                        geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
                        shading_model,
                        pass_type,
                        features,
                        quality,
                        platform_token: ASSET_SCAN_PLATFORM_TOKEN.to_string(),
                    },
                    wgsl_source: wgsl_source.clone(),
                    include_content_hashes: include_content_hashes.clone(),
                    template_revision: ASSET_SCAN_TEMPLATE_REVISION.to_string(),
                    naga_version: ASSET_SCAN_NAGA_VERSION.to_string(),
                    wgpu_version: ASSET_SCAN_WGPU_VERSION.to_string(),
                }
            })
        })
        .collect()
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
    if material.double_sided {
        bits |= ShaderFeatureBits::DOUBLE_SIDED;
    }
    ShaderFeatureBits::new(bits)
}

fn material_shading_model_id(material: &MaterialAsset) -> ShadingModelId {
    ShadingModelId::from_lighting_model(&material.lighting_model())
        .unwrap_or(SHADING_MODEL_ID_STANDARD_PBR)
}

fn material_pass_filter(material: &MaterialAsset) -> Option<Vec<ShaderPassType>> {
    matches!(material.alpha_mode, AlphaMode::Blend).then(|| ASSET_SCAN_ALPHA_BLEND_PASSES.to_vec())
}

#[derive(Clone, Debug)]
struct ShaderPrewarmSource {
    stable_label: String,
    resource_id: ResourceId,
    revision: u64,
    wgsl_source: String,
    include_content_hashes: Vec<String>,
    pass_types: Vec<ShaderPassType>,
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
    pass_filter: Option<Vec<ShaderPassType>>,
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

fn wgsl_files_for_document(
    package_dir: &Path,
    document: &ZShaderDocument,
) -> Result<Vec<PathBuf>, String> {
    if !document.wgsl_files.is_empty() {
        return Ok(document.wgsl_files.iter().map(PathBuf::from).collect());
    }
    let mut files = Vec::new();
    collect_files_with_extension(package_dir, "wgsl", &mut files)?;
    files.sort();
    files
        .into_iter()
        .map(|path| {
            path.strip_prefix(package_dir)
                .map(PathBuf::from)
                .map_err(|error| {
                    format!(
                        "shader source {} is outside package dir {}: {error}",
                        path.display(),
                        package_dir.display()
                    )
                })
        })
        .collect()
}

fn asset_scan_pass_types_for_zshader(document: &ZShaderDocument) -> Vec<ShaderPassType> {
    if document.entry_points.is_empty() {
        return ASSET_SCAN_FULL_MATERIAL_PASSES.to_vec();
    }

    let has_vertex = document
        .entry_points
        .iter()
        .any(|entry| is_vertex_stage(&entry.stage));
    let has_fragment = document
        .entry_points
        .iter()
        .any(|entry| is_fragment_stage(&entry.stage));

    match (has_vertex, has_fragment) {
        (true, true) => ASSET_SCAN_FULL_MATERIAL_PASSES.to_vec(),
        (true, false) => ASSET_SCAN_VERTEX_ONLY_PASSES.to_vec(),
        (false, true) => ASSET_SCAN_FRAGMENT_ONLY_PASSES.to_vec(),
        (false, false) => Vec::new(),
    }
}

fn is_vertex_stage(stage: &str) -> bool {
    matches!(
        stage.trim().to_ascii_lowercase().as_str(),
        "vertex" | "vert" | "vs"
    )
}

fn is_fragment_stage(stage: &str) -> bool {
    matches!(
        stage.trim().to_ascii_lowercase().as_str(),
        "fragment" | "frag" | "fs"
    )
}

fn primary_zshader_path(package_dir: &Path) -> Option<PathBuf> {
    let mut files = Vec::new();
    collect_files_with_extension(package_dir, "zshader", &mut files).ok()?;
    files.sort();
    files.into_iter().next()
}

fn collect_files_with_extension(
    root: &Path,
    extension: &str,
    files: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)
        .map_err(|error| format!("failed to read shader package {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read shader package {} entry: {error}",
                root.display()
            )
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_with_extension(&path, extension, files)?;
        } else if has_extension(&path, extension) {
            files.push(path);
        }
    }
    Ok(())
}

fn meta_path_for_single_source(meta_path: &Path) -> PathBuf {
    let file_name = meta_path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".zmeta"))
        .unwrap_or_default();
    meta_path.with_file_name(file_name)
}

fn is_inside_compound_shader_source(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    let Some(parent_name) = parent.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    parent
        .parent()
        .map(|grandparent| grandparent.join(format!("{parent_name}.zmeta")).exists())
        .unwrap_or(false)
}

fn is_zmeta(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.ends_with(".zmeta"))
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn stable_label_for_path(asset_root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(asset_root).unwrap_or(path);
    let normalized = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    format!("asset-scan://{normalized}")
}

fn content_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests;
