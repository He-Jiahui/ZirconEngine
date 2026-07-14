use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::asset::ProjectAssetManager;
use crate::core::framework::render::{
    builtin_geometry_source_descriptor, GeometrySourceDescriptor, GeometrySourceId,
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantPrewarmManifest,
    ShaderVariantPrewarmReport, ShaderVariantPrewarmRequest, ShadingModelDescriptor,
    ShadingModelId, GEOMETRY_SOURCE_ID_STATIC_MESH,
};
use crate::graphics::material::ShadingModelIncludeSourceSet;
use crate::graphics::scene::{
    create_mesh_prewarm_validation_pipeline_layout, default_pipeline_key,
    mesh_pipeline_standard_material_template_source_for_shader_pass,
    mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor,
    validate_mesh_prewarm_request_render_pipeline, MeshPipelineShaderSource, PipelineKey,
};
use crate::graphics::shader::{
    assemble_deferred_gbuffer_shader_template, assemble_material_shader_template,
    assemble_taa_reactive_mask_shader_template, prewarm_shader_variants_to_disk,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation,
    prewarm_shader_variants_to_disk_with_module_validation,
    prewarm_shader_variants_to_disk_with_pipeline_validation,
    standard_material_surface_source_for_features, DeferredGBufferShaderTemplateRequest,
    MaterialShaderTemplateAssembly, MaterialShaderTemplateRequest, ShaderTemplateAssemblyError,
    ShaderVariantCacheDisk, TaaReactiveMaskShaderTemplateRequest,
};

const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";
const MESH_SHADER_PLATFORM_TOKEN: &str = "wgpu-runtime";
const BUILTIN_STANDARD_MATERIAL_SOURCE_LABEL: &str = "builtin://shader/pbr.wgsl";
const BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES: [ShaderPassType; 6] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
    ShaderPassType::TaaReactiveMask,
];

pub fn prewarm_shader_variants(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk(manifest, cache_dir)
}

pub fn prewarm_shader_variants_with_wgpu_module_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
        Ok(backend) => backend,
        Err(error) => {
            return wgpu_module_validation_setup_failure_report(
                manifest,
                format!("failed to create offscreen WGPU backend: {error:?}"),
            );
        }
    };
    let device = &backend.device;
    prewarm_shader_variants_to_disk_with_module_validation(manifest, cache_dir, |request| {
        validate_mesh_prewarm_request_shader_module(device, request)
    })
}

pub fn prewarm_shader_variants_with_wgpu_pipeline_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
        Ok(backend) => backend,
        Err(error) => {
            return wgpu_pipeline_validation_setup_failure_report(
                manifest,
                format!("failed to create offscreen WGPU backend: {error:?}"),
            );
        }
    };
    let device = &backend.device;
    let pipeline_layout = create_mesh_prewarm_validation_pipeline_layout(device);
    prewarm_shader_variants_to_disk_with_pipeline_validation(manifest, cache_dir, |request| {
        validate_mesh_prewarm_request_render_pipeline(device, &pipeline_layout, request)
    })
}

pub fn prewarm_shader_variants_with_wgpu_module_and_pipeline_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
        Ok(backend) => backend,
        Err(error) => {
            return wgpu_module_and_pipeline_validation_setup_failure_report(
                manifest,
                format!("failed to create offscreen WGPU backend: {error:?}"),
            );
        }
    };
    let device = &backend.device;
    let pipeline_layout = create_mesh_prewarm_validation_pipeline_layout(device);
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation(
        manifest,
        cache_dir,
        |request| validate_mesh_prewarm_request_shader_module(device, request),
        |request| validate_mesh_prewarm_request_render_pipeline(device, &pipeline_layout, request),
    )
}

fn validate_mesh_prewarm_request_shader_module(
    device: &wgpu::Device,
    request: &ShaderVariantPrewarmRequest,
) -> Result<(), String> {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-shader-prewarm-validation-module"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(request.wgsl_source.as_str())),
    });
    match pollster::block_on(error_scope.pop()) {
        Some(error) => Err(error.to_string()),
        None => Ok(()),
    }
}

fn wgpu_module_validation_setup_failure_report(
    manifest: &ShaderVariantPrewarmManifest,
    error: impl Into<String>,
) -> ShaderVariantPrewarmReport {
    let error = error.into();
    let mut report = ShaderVariantPrewarmReport::default();
    report.enable_wgpu_module_validation(manifest.variants.len());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        report.record_failure_request(
            variant_index,
            request,
            format!("WGPU shader module validation setup failed: {error}"),
        );
        report.record_wgpu_module_validation_failed();
    }
    report
}

fn wgpu_pipeline_validation_setup_failure_report(
    manifest: &ShaderVariantPrewarmManifest,
    error: impl Into<String>,
) -> ShaderVariantPrewarmReport {
    let error = error.into();
    let mut report = ShaderVariantPrewarmReport::default();
    report.enable_wgpu_pipeline_validation(manifest.variants.len());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        report.record_failure_request(
            variant_index,
            request,
            format!("WGPU render pipeline validation setup failed: {error}"),
        );
        report.record_wgpu_pipeline_validation_failed();
    }
    report
}

fn wgpu_module_and_pipeline_validation_setup_failure_report(
    manifest: &ShaderVariantPrewarmManifest,
    error: impl Into<String>,
) -> ShaderVariantPrewarmReport {
    let error = error.into();
    let mut report = ShaderVariantPrewarmReport::default();
    report.enable_wgpu_module_validation(manifest.variants.len());
    report.enable_wgpu_pipeline_validation(manifest.variants.len());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        report.record_failure_request(
            variant_index,
            request,
            format!("WGPU shader module and render pipeline validation setup failed: {error}"),
        );
        report.record_wgpu_module_validation_failed();
        report.record_wgpu_pipeline_validation_failed();
    }
    report
}

pub fn builtin_fallback_shader_prewarm_manifest() -> ShaderVariantPrewarmManifest {
    let pipeline_key = default_pipeline_key();
    builtin_standard_material_shader_prewarm_manifest_for_pipeline_key(
        pipeline_key,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        None,
        &[ShaderQualityTier::Medium],
    )
}

pub fn builtin_standard_material_shader_prewarm_manifest(
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.alpha_mask = features.contains(ShaderFeatureBits::ALPHA_TEST);
    pipeline_key.alpha_cutoff_bits = pipeline_key
        .alpha_mask
        .then(|| alpha_cutoff.unwrap_or(0.0).to_bits());
    pipeline_key.double_sided = features.contains(ShaderFeatureBits::DOUBLE_SIDED);
    pipeline_key.receive_shadows = features.contains(ShaderFeatureBits::RECEIVE_SHADOWS);
    pipeline_key.pbr_clearcoat = features.contains(ShaderFeatureBits::PBR_CLEARCOAT);
    pipeline_key.pbr_anisotropy = features.contains(ShaderFeatureBits::PBR_ANISOTROPY);
    pipeline_key.pbr_transmission = features.contains(ShaderFeatureBits::PBR_TRANSMISSION);
    pipeline_key.shading_model_id = shading_model;

    builtin_standard_material_shader_prewarm_manifest_for_pipeline_key(
        pipeline_key,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        None,
        quality_tiers,
    )
}

pub fn builtin_standard_material_shader_prewarm_manifest_for_geometry(
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    geometry_source: GeometrySourceId,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.alpha_mask = features.contains(ShaderFeatureBits::ALPHA_TEST);
    pipeline_key.alpha_cutoff_bits = pipeline_key
        .alpha_mask
        .then(|| alpha_cutoff.unwrap_or(0.0).to_bits());
    pipeline_key.double_sided = features.contains(ShaderFeatureBits::DOUBLE_SIDED);
    pipeline_key.receive_shadows = features.contains(ShaderFeatureBits::RECEIVE_SHADOWS);
    pipeline_key.pbr_clearcoat = features.contains(ShaderFeatureBits::PBR_CLEARCOAT);
    pipeline_key.pbr_anisotropy = features.contains(ShaderFeatureBits::PBR_ANISOTROPY);
    pipeline_key.pbr_transmission = features.contains(ShaderFeatureBits::PBR_TRANSMISSION);
    pipeline_key.shading_model_id = shading_model;

    builtin_standard_material_shader_prewarm_manifest_for_pipeline_key(
        pipeline_key,
        geometry_source,
        None,
        quality_tiers,
    )
}

pub fn builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor(
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    geometry_source: &GeometrySourceDescriptor,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.alpha_mask = features.contains(ShaderFeatureBits::ALPHA_TEST);
    pipeline_key.alpha_cutoff_bits = pipeline_key
        .alpha_mask
        .then(|| alpha_cutoff.unwrap_or(0.0).to_bits());
    pipeline_key.double_sided = features.contains(ShaderFeatureBits::DOUBLE_SIDED);
    pipeline_key.receive_shadows = features.contains(ShaderFeatureBits::RECEIVE_SHADOWS);
    pipeline_key.pbr_clearcoat = features.contains(ShaderFeatureBits::PBR_CLEARCOAT);
    pipeline_key.pbr_anisotropy = features.contains(ShaderFeatureBits::PBR_ANISOTROPY);
    pipeline_key.pbr_transmission = features.contains(ShaderFeatureBits::PBR_TRANSMISSION);
    pipeline_key.shading_model_id = shading_model;

    builtin_standard_material_shader_prewarm_manifest_for_pipeline_key(
        pipeline_key,
        geometry_source.id,
        Some(geometry_source),
        quality_tiers,
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderPrewarmTemplateSource {
    pub wgsl_source: String,
    pub include_content_hashes: Vec<String>,
    pub template_revision: String,
}

impl ShaderPrewarmTemplateSource {
    fn from_template(
        assembly: MaterialShaderTemplateAssembly,
        source_content_hashes: &[String],
    ) -> Self {
        let mut include_content_hashes = source_content_hashes.to_vec();
        include_content_hashes.extend(assembly.include_content_hashes);
        include_content_hashes.push(shader_prewarm_source_hash(&assembly.wgsl_source));
        Self {
            wgsl_source: assembly.wgsl_source,
            include_content_hashes,
            template_revision: assembly.template_revision,
        }
    }
}

pub fn material_surface_shader_prewarm_template_source(
    material_surface_source: &str,
    pass_type: ShaderPassType,
    geometry_source: GeometrySourceId,
    geometry_source_descriptor: Option<&GeometrySourceDescriptor>,
    features: ShaderFeatureBits,
    alpha_cutoff: Option<f32>,
    source_content_hashes: &[String],
) -> Result<ShaderPrewarmTemplateSource, String> {
    let geometry_source = geometry_source_descriptor
        .cloned()
        .or_else(|| builtin_geometry_source_descriptor(geometry_source))
        .ok_or_else(|| format!("unknown geometry source {}", geometry_source.value()))?;
    let prepared_surface = material_surface_source_for_prewarm_template(
        material_surface_source,
        features,
        alpha_cutoff,
    );

    let assembly = match pass_type {
        ShaderPassType::GBuffer => assemble_deferred_gbuffer_shader_template(
            DeferredGBufferShaderTemplateRequest::new(
                geometry_source,
                prepared_surface.source.as_ref(),
                prepared_surface.entry_point,
            )
            .with_features(features),
        ),
        ShaderPassType::TaaReactiveMask => assemble_taa_reactive_mask_shader_template(
            TaaReactiveMaskShaderTemplateRequest::new(
                geometry_source,
                prepared_surface.source.as_ref(),
                prepared_surface.entry_point,
            )
            .with_features(features),
        ),
        _ => assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                geometry_source,
                pass_type,
                prepared_surface.source.as_ref(),
                prepared_surface.entry_point,
            )
            .with_features(features),
        ),
    }
    .map_err(|error| format!("{error:?}"))?;

    Ok(ShaderPrewarmTemplateSource::from_template(
        assembly,
        source_content_hashes,
    ))
}

struct PreparedMaterialSurfaceSource<'a> {
    source: Cow<'a, str>,
    entry_point: &'static str,
}

fn material_surface_source_for_prewarm_template(
    source: &str,
    features: ShaderFeatureBits,
    alpha_cutoff: Option<f32>,
) -> PreparedMaterialSurfaceSource<'_> {
    if source.contains("fn zr_material_surface(") {
        return PreparedMaterialSurfaceSource {
            source: Cow::Borrowed(source),
            entry_point: "zr_material_surface",
        };
    }

    let fallback =
        standard_material_surface_source_for_features(features, alpha_cutoff.unwrap_or(0.0));
    PreparedMaterialSurfaceSource {
        source: Cow::Owned(format!("{source}\n\n{}", fallback.source)),
        entry_point: fallback.entry_point,
    }
}

pub(crate) fn builtin_standard_material_shader_prewarm_manifest_for_geometry_with_plugin_shading_models(
    asset_manager: &ProjectAssetManager,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    geometry_source: GeometrySourceId,
    quality_tiers: &[ShaderQualityTier],
    plugin_shading_models: &[ShadingModelDescriptor],
) -> Result<ShaderVariantPrewarmManifest, ShaderTemplateAssemblyError> {
    let geometry_source = builtin_geometry_source_descriptor(geometry_source).ok_or_else(|| {
        ShaderTemplateAssemblyError::UnknownGeometryInclude {
            token: format!("geometry_source_{}", geometry_source.value()),
        }
    })?;
    builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor_with_plugin_shading_models(
        asset_manager,
        features,
        shading_model,
        alpha_cutoff,
        &geometry_source,
        quality_tiers,
        plugin_shading_models,
    )
}

fn builtin_standard_material_shader_prewarm_manifest_for_geometry_descriptor_with_plugin_shading_models(
    asset_manager: &ProjectAssetManager,
    features: ShaderFeatureBits,
    shading_model: ShadingModelId,
    alpha_cutoff: Option<f32>,
    geometry_source: &GeometrySourceDescriptor,
    quality_tiers: &[ShaderQualityTier],
    plugin_shading_models: &[ShadingModelDescriptor],
) -> Result<ShaderVariantPrewarmManifest, ShaderTemplateAssemblyError> {
    let mut pipeline_key = default_pipeline_key();
    pipeline_key.alpha_mask = features.contains(ShaderFeatureBits::ALPHA_TEST);
    pipeline_key.alpha_cutoff_bits = pipeline_key
        .alpha_mask
        .then(|| alpha_cutoff.unwrap_or(0.0).to_bits());
    pipeline_key.double_sided = features.contains(ShaderFeatureBits::DOUBLE_SIDED);
    pipeline_key.receive_shadows = features.contains(ShaderFeatureBits::RECEIVE_SHADOWS);
    pipeline_key.pbr_clearcoat = features.contains(ShaderFeatureBits::PBR_CLEARCOAT);
    pipeline_key.pbr_anisotropy = features.contains(ShaderFeatureBits::PBR_ANISOTROPY);
    pipeline_key.pbr_transmission = features.contains(ShaderFeatureBits::PBR_TRANSMISSION);
    pipeline_key.shading_model_id = shading_model;
    let source_set = ShadingModelIncludeSourceSet::from_project_asset_manager(
        asset_manager,
        plugin_shading_models,
    )
    .map_err(|error| ShaderTemplateAssemblyError::UnknownShadingInclude {
        token: error.to_string(),
    })?;

    let quality_tiers = normalized_quality_tiers(quality_tiers);
    let mut requests = Vec::new();
    for pass_type in BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES {
        let source = builtin_standard_material_template_source_for_plugin_shading_model_and_pass(
            &pipeline_key,
            geometry_source,
            pass_type,
            plugin_shading_models,
            &source_set,
        )?;
        requests.extend(quality_tiers.iter().copied().map(|quality| {
            let mut key = pipeline_key.shader_variant_key_for_geometry(
                pass_type,
                geometry_source.id,
                MESH_SHADER_PLATFORM_TOKEN,
            );
            key.quality = quality;
            ShaderVariantPrewarmRequest {
                key,
                source_label: BUILTIN_STANDARD_MATERIAL_SOURCE_LABEL.to_string(),
                wgsl_source: source.wgsl_source.clone(),
                include_content_hashes: source.cache_content_hashes.clone(),
                template_revision: source.template_revision.clone(),
                naga_version: MESH_SHADER_NAGA_VERSION.to_string(),
                wgpu_version: MESH_SHADER_WGPU_VERSION.to_string(),
            }
        }));
    }

    Ok(ShaderVariantPrewarmManifest::new(requests))
}

fn builtin_standard_material_shader_prewarm_manifest_for_pipeline_key(
    pipeline_key: PipelineKey,
    geometry_source: GeometrySourceId,
    geometry_source_descriptor: Option<&GeometrySourceDescriptor>,
    quality_tiers: &[ShaderQualityTier],
) -> ShaderVariantPrewarmManifest {
    let quality_tiers = normalized_quality_tiers(quality_tiers);
    let mut requests = Vec::new();
    for pass_type in BUILTIN_STANDARD_MATERIAL_PREWARM_PASSES {
        let MeshPipelineShaderSource {
            wgsl_source,
            cache_content_hashes,
            template_revision,
            ..
        } = match builtin_standard_material_template_source_for_geometry_descriptor_and_pass(
            &pipeline_key,
            geometry_source,
            geometry_source_descriptor,
            pass_type,
        ) {
            Ok(source) => source,
            Err(_) => return ShaderVariantPrewarmManifest::new(Vec::new()),
        };

        requests.extend(quality_tiers.iter().copied().map(|quality| {
            let mut key = pipeline_key.shader_variant_key_for_geometry(
                pass_type,
                geometry_source,
                MESH_SHADER_PLATFORM_TOKEN,
            );
            key.quality = quality;
            ShaderVariantPrewarmRequest {
                key,
                source_label: BUILTIN_STANDARD_MATERIAL_SOURCE_LABEL.to_string(),
                wgsl_source: wgsl_source.clone(),
                include_content_hashes: cache_content_hashes.clone(),
                template_revision: template_revision.clone(),
                naga_version: MESH_SHADER_NAGA_VERSION.to_string(),
                wgpu_version: MESH_SHADER_WGPU_VERSION.to_string(),
            }
        }));
    }

    ShaderVariantPrewarmManifest::new(requests)
}

fn builtin_standard_material_template_source_for_geometry_and_pass(
    pipeline_key: &PipelineKey,
    geometry_source: GeometrySourceId,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, crate::graphics::shader::ShaderTemplateAssemblyError> {
    mesh_pipeline_standard_material_template_source_for_shader_pass(
        pipeline_key,
        geometry_source,
        pass_type,
    )
}

fn builtin_standard_material_template_source_for_geometry_descriptor_and_pass(
    pipeline_key: &PipelineKey,
    geometry_source: GeometrySourceId,
    geometry_source_descriptor: Option<&GeometrySourceDescriptor>,
    pass_type: ShaderPassType,
) -> Result<MeshPipelineShaderSource, crate::graphics::shader::ShaderTemplateAssemblyError> {
    if let Some(descriptor) = geometry_source_descriptor {
        return mesh_pipeline_standard_material_template_source_for_shader_pass_and_descriptor(
            pipeline_key,
            descriptor,
            pass_type,
        );
    }
    builtin_standard_material_template_source_for_geometry_and_pass(
        pipeline_key,
        geometry_source,
        pass_type,
    )
}

struct PluginShadingModelTemplateSource {
    wgsl_source: String,
    cache_content_hashes: Vec<String>,
    template_revision: String,
}

impl PluginShadingModelTemplateSource {
    fn from_template(assembly: MaterialShaderTemplateAssembly) -> Self {
        let mut cache_content_hashes = assembly.include_content_hashes;
        cache_content_hashes.push(shader_prewarm_source_hash(&assembly.wgsl_source));
        Self {
            wgsl_source: assembly.wgsl_source,
            cache_content_hashes,
            template_revision: assembly.template_revision,
        }
    }
}

fn shader_prewarm_source_hash(source: &str) -> String {
    blake3::hash(source.as_bytes()).to_hex().to_string()
}

fn builtin_standard_material_template_source_for_plugin_shading_model_and_pass(
    pipeline_key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    pass_type: ShaderPassType,
    plugin_shading_models: &[ShadingModelDescriptor],
    source_set: &ShadingModelIncludeSourceSet,
) -> Result<PluginShadingModelTemplateSource, ShaderTemplateAssemblyError> {
    let descriptor =
        plugin_shading_model_descriptor_for_pipeline_key(pipeline_key, plugin_shading_models)?;
    match pass_type {
        ShaderPassType::GBuffer => plugin_shading_model_gbuffer_template_source(
            pipeline_key,
            geometry_source,
            descriptor,
            source_set,
        ),
        ShaderPassType::TaaReactiveMask => {
            plugin_shading_model_taa_reactive_mask_template_source(pipeline_key, geometry_source)
        }
        _ => plugin_shading_model_material_template_source(
            pipeline_key,
            geometry_source,
            pass_type,
            descriptor,
            source_set,
        ),
    }
}

fn plugin_shading_model_material_template_source(
    pipeline_key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    pass_type: ShaderPassType,
    descriptor: Option<&ShadingModelDescriptor>,
    source_set: &ShadingModelIncludeSourceSet,
) -> Result<PluginShadingModelTemplateSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        pipeline_key.shader_feature_bits(),
        prewarm_alpha_cutoff(pipeline_key),
    );
    let mut request = MaterialShaderTemplateRequest::new(
        geometry_source.clone(),
        pass_type,
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    if let Some(descriptor) = descriptor.cloned() {
        request = request
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_forward_include_sources(source_set);
    }
    assemble_material_shader_template(request).map(PluginShadingModelTemplateSource::from_template)
}

fn plugin_shading_model_gbuffer_template_source(
    pipeline_key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
    descriptor: Option<&ShadingModelDescriptor>,
    source_set: &ShadingModelIncludeSourceSet,
) -> Result<PluginShadingModelTemplateSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        pipeline_key.shader_feature_bits(),
        prewarm_alpha_cutoff(pipeline_key),
    );
    let mut request = DeferredGBufferShaderTemplateRequest::new(
        geometry_source.clone(),
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    if let Some(descriptor) = descriptor.cloned() {
        request = request
            .with_shading_model_descriptor(descriptor)
            .with_shading_model_gbuffer_include_sources(source_set);
    }
    assemble_deferred_gbuffer_shader_template(request)
        .map(PluginShadingModelTemplateSource::from_template)
}

fn plugin_shading_model_taa_reactive_mask_template_source(
    pipeline_key: &PipelineKey,
    geometry_source: &GeometrySourceDescriptor,
) -> Result<PluginShadingModelTemplateSource, ShaderTemplateAssemblyError> {
    let material_surface = standard_material_surface_source_for_features(
        pipeline_key.shader_feature_bits(),
        prewarm_alpha_cutoff(pipeline_key),
    );
    let request = TaaReactiveMaskShaderTemplateRequest::new(
        geometry_source.clone(),
        material_surface.source,
        material_surface.entry_point,
    )
    .with_features(material_surface.features);
    assemble_taa_reactive_mask_shader_template(request)
        .map(PluginShadingModelTemplateSource::from_template)
}

fn plugin_shading_model_descriptor_for_pipeline_key<'a>(
    pipeline_key: &PipelineKey,
    plugin_shading_models: &'a [ShadingModelDescriptor],
) -> Result<Option<&'a ShadingModelDescriptor>, ShaderTemplateAssemblyError> {
    if !pipeline_key.shading_model_id.is_plugin_range() {
        return Ok(None);
    }
    plugin_shading_models
        .iter()
        .find(|descriptor| descriptor.id == pipeline_key.shading_model_id)
        .map(Some)
        .ok_or_else(|| ShaderTemplateAssemblyError::UnknownShadingInclude {
            token: format!("shading_model_id_{}", pipeline_key.shading_model_id.value()),
        })
}

fn prewarm_alpha_cutoff(key: &PipelineKey) -> f32 {
    if key.is_alpha_mask() {
        key.alpha_cutoff_bits.map(f32::from_bits).unwrap_or(0.0)
    } else {
        0.0
    }
}

fn normalized_quality_tiers(quality_tiers: &[ShaderQualityTier]) -> Vec<ShaderQualityTier> {
    if quality_tiers.is_empty() {
        return vec![ShaderQualityTier::Medium];
    }
    let mut tiers = Vec::new();
    for quality in quality_tiers {
        if !tiers.contains(quality) {
            tiers.push(*quality);
        }
    }
    tiers
}

pub fn default_shader_variant_cache_root_for_project(project_root: impl AsRef<Path>) -> PathBuf {
    ShaderVariantCacheDisk::default_project_root(project_root.as_ref())
}

pub fn default_staged_shader_variant_cache_root_for_project(
    project_root: impl AsRef<Path>,
) -> PathBuf {
    ShaderVariantCacheDisk::default_staged_project_root(project_root.as_ref())
}

#[cfg(test)]
#[path = "shader_prewarm/tests.rs"]
mod tests;
