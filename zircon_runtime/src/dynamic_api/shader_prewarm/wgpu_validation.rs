use std::borrow::Cow;
use std::path::Path;

use crate::core::framework::render::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
};
use crate::graphics::scene::{
    create_mesh_prewarm_validation_pipeline_layout, validate_mesh_prewarm_request_render_pipeline,
};
use crate::graphics::shader::{
    prewarm_shader_variants_to_disk_with_budget,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation,
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget,
    prewarm_shader_variants_to_disk_with_module_validation,
    prewarm_shader_variants_to_disk_with_module_validation_and_budget,
    prewarm_shader_variants_to_disk_with_pipeline_validation,
    prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget,
};

use super::execution_budget::{
    execution_budget_preflight_failure_report, preflight_execution_budget, with_execution_budget,
};
use super::module_validation_cache::ShaderPrewarmModuleValidationCache;

pub fn prewarm_shader_variants_with_execution_budget(
    manifest: &ShaderVariantPrewarmManifest,
    cache_dir: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
    validate_wgpu_modules: bool,
    validate_wgpu_pipelines: bool,
) -> ShaderVariantPrewarmReport {
    if let Err(error) = preflight_execution_budget(manifest, budget) {
        return execution_budget_preflight_failure_report(manifest, budget, error);
    }
    match (validate_wgpu_modules, validate_wgpu_pipelines) {
        (false, false) => prewarm_shader_variants_to_disk_with_budget(manifest, cache_dir, budget),
        (true, false) => {
            let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
                Ok(backend) => backend,
                Err(error) => {
                    return with_execution_budget(
                        wgpu_module_validation_setup_failure_report(
                            manifest,
                            format!("failed to create offscreen WGPU backend: {error:?}"),
                        ),
                        manifest,
                        budget,
                    );
                }
            };
            let module_validation_cache =
                ShaderPrewarmModuleValidationCache::new(&manifest.sources);
            prewarm_shader_variants_to_disk_with_module_validation_and_budget(
                manifest,
                cache_dir,
                budget,
                |_request, source| {
                    module_validation_cache.validate(source, || {
                        validate_mesh_prewarm_source_module(&backend.device, source)
                    })
                },
            )
        }
        (false, true) => {
            let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
                Ok(backend) => backend,
                Err(error) => {
                    return with_execution_budget(
                        wgpu_pipeline_validation_setup_failure_report(
                            manifest,
                            format!("failed to create offscreen WGPU backend: {error:?}"),
                        ),
                        manifest,
                        budget,
                    );
                }
            };
            let pipeline_layout = create_mesh_prewarm_validation_pipeline_layout(&backend.device);
            prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget(
                manifest,
                cache_dir,
                budget,
                |request, source| {
                    validate_mesh_prewarm_request_render_pipeline(
                        &backend.device,
                        &pipeline_layout,
                        request,
                        source,
                    )
                },
            )
        }
        (true, true) => {
            let backend = match crate::graphics::backend::RenderBackend::new_offscreen() {
                Ok(backend) => backend,
                Err(error) => {
                    return with_execution_budget(
                        wgpu_module_and_pipeline_validation_setup_failure_report(
                            manifest,
                            format!("failed to create offscreen WGPU backend: {error:?}"),
                        ),
                        manifest,
                        budget,
                    );
                }
            };
            let pipeline_layout = create_mesh_prewarm_validation_pipeline_layout(&backend.device);
            let module_validation_cache =
                ShaderPrewarmModuleValidationCache::new(&manifest.sources);
            prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget(
                manifest,
                cache_dir,
                budget,
                |_request, source| {
                    module_validation_cache.validate(source, || {
                        validate_mesh_prewarm_source_module(&backend.device, source)
                    })
                },
                |request, source| {
                    validate_mesh_prewarm_request_render_pipeline(
                        &backend.device,
                        &pipeline_layout,
                        request,
                        source,
                    )
                },
            )
        }
    }
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
    let module_validation_cache = ShaderPrewarmModuleValidationCache::new(&manifest.sources);
    prewarm_shader_variants_to_disk_with_module_validation(
        manifest,
        cache_dir,
        |_request, source| {
            module_validation_cache.validate(source, || {
                validate_mesh_prewarm_source_module(device, source)
            })
        },
    )
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
    prewarm_shader_variants_to_disk_with_pipeline_validation(
        manifest,
        cache_dir,
        |request, source| {
            validate_mesh_prewarm_request_render_pipeline(device, &pipeline_layout, request, source)
        },
    )
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
    let module_validation_cache = ShaderPrewarmModuleValidationCache::new(&manifest.sources);
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation(
        manifest,
        cache_dir,
        |_request, source| {
            module_validation_cache.validate(source, || {
                validate_mesh_prewarm_source_module(device, source)
            })
        },
        |request, source| {
            validate_mesh_prewarm_request_render_pipeline(device, &pipeline_layout, request, source)
        },
    )
}

fn validate_mesh_prewarm_source_module(
    device: &wgpu::Device,
    source: &ShaderVariantPrewarmSource,
) -> Result<(), String> {
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-shader-prewarm-validation-module"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source.wgsl_source.as_str())),
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
    let source_table = manifest.source_table();
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        record_wgpu_setup_failure(
            &mut report,
            variant_index,
            request,
            source_table.source_for(request),
            &error,
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
    let source_table = manifest.source_table();
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        record_wgpu_setup_failure(
            &mut report,
            variant_index,
            request,
            source_table.source_for(request),
            &error,
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
    let source_table = manifest.source_table();
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        let failure =
            format!("WGPU shader module and render pipeline validation setup failed: {error}");
        if let Some(source) = source_table.source_for(request) {
            report.record_failure_request(variant_index, request, source, failure);
        } else {
            report.record_failure_variant(variant_index, &request.key, failure);
        }
        report.record_wgpu_module_validation_failed();
        report.record_wgpu_pipeline_validation_failed();
    }
    report
}

fn record_wgpu_setup_failure(
    report: &mut ShaderVariantPrewarmReport,
    variant_index: usize,
    request: &ShaderVariantPrewarmRequest,
    source: Option<&ShaderVariantPrewarmSource>,
    error: &str,
) {
    let failure = format!("WGPU shader validation setup failed: {error}");
    if let Some(source) = source {
        report.record_failure_request(variant_index, request, source, failure);
    } else {
        report.record_failure_variant(variant_index, &request.key, failure);
    }
}
