use std::path::Path;

use crate::core::framework::render::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
};

mod worker;

pub(crate) fn prewarm_shader_variants_to_disk(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_with_budget(
        manifest,
        cache_root,
        ShaderVariantPrewarmExecutionBudget::default(),
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_budget(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
) -> ShaderVariantPrewarmReport {
    worker::prewarm_shader_variants_to_disk_inner(manifest, cache_root, budget, None, None)
}

pub(crate) fn prewarm_shader_variants_to_disk_with_module_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    validate_module: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_with_module_validation_and_budget(
        manifest,
        cache_root,
        ShaderVariantPrewarmExecutionBudget::default(),
        validate_module,
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_module_validation_and_budget(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
    validate_module: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    worker::prewarm_shader_variants_to_disk_inner(
        manifest,
        cache_root,
        budget,
        Some(&validate_module),
        None,
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_pipeline_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    validate_pipeline: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget(
        manifest,
        cache_root,
        ShaderVariantPrewarmExecutionBudget::default(),
        validate_pipeline,
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_pipeline_validation_and_budget(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
    validate_pipeline: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    worker::prewarm_shader_variants_to_disk_inner(
        manifest,
        cache_root,
        budget,
        None,
        Some(&validate_pipeline),
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_module_and_pipeline_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    validate_module: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
    validate_pipeline: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget(
        manifest,
        cache_root,
        ShaderVariantPrewarmExecutionBudget::default(),
        validate_module,
        validate_pipeline,
    )
}

pub(crate) fn prewarm_shader_variants_to_disk_with_module_and_pipeline_validation_and_budget(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
    validate_module: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
    validate_pipeline: impl Fn(
        &ShaderVariantPrewarmRequest,
        &ShaderVariantPrewarmSource,
    ) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    worker::prewarm_shader_variants_to_disk_inner(
        manifest,
        cache_root,
        budget,
        Some(&validate_module),
        Some(&validate_pipeline),
    )
}

#[cfg(test)]
#[path = "prewarm/tests.rs"]
mod tests;
