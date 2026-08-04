use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::core::framework::render::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
};
use crate::graphics::shader::template::validate_shader_variant_prewarm_wgsl;

use super::super::disk::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

pub(super) fn prewarm_shader_variants_to_disk_inner(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    budget: ShaderVariantPrewarmExecutionBudget,
    validate_module: Option<
        &dyn Fn(&ShaderVariantPrewarmRequest, &ShaderVariantPrewarmSource) -> Result<(), String>,
    >,
    validate_pipeline: Option<
        &dyn Fn(&ShaderVariantPrewarmRequest, &ShaderVariantPrewarmSource) -> Result<(), String>,
    >,
) -> ShaderVariantPrewarmReport {
    let mut report = ShaderVariantPrewarmReport::default();
    report.execution_budget.configure(budget);
    if let Err(error) = budget.validate() {
        report.execution_budget.record_rejected();
        report.record_failure(0, error.to_string());
        return report;
    }
    let Some(resident_source_bytes) = manifest.source_table_resident_bytes() else {
        report.execution_budget.record_rejected();
        report.record_failure(
            0,
            "shader prewarm source table resident byte count overflowed",
        );
        return report;
    };
    report
        .execution_budget
        .record_source_residency(resident_source_bytes);
    if resident_source_bytes > budget.max_resident_source_bytes {
        report.execution_budget.record_rejected();
        report.record_failure(
            0,
            format!(
                "shader prewarm source table requires {resident_source_bytes} resident bytes; budget is {}",
                budget.max_resident_source_bytes
            ),
        );
        return report;
    }
    let wgpu_module_validation_enabled = validate_module.is_some();
    let wgpu_pipeline_validation_enabled = validate_pipeline.is_some();
    if wgpu_module_validation_enabled {
        report.enable_wgpu_module_validation(manifest.variants.len());
    }
    if wgpu_pipeline_validation_enabled {
        report.enable_wgpu_pipeline_validation(manifest.variants.len());
    }
    if manifest.schema_version != ShaderVariantPrewarmManifest::SCHEMA_VERSION {
        report.record_failure(
            0,
            format!(
                "shader variant prewarm manifest schema {} is not supported; expected {}",
                manifest.schema_version,
                ShaderVariantPrewarmManifest::SCHEMA_VERSION
            ),
        );
        return report;
    }
    if let Err(error) = manifest.validate_integrity() {
        report.record_failure(0, error.to_string());
        return report;
    }

    let cache = ShaderVariantCacheDisk::new(cache_root.as_ref());
    let mut written_disk_hashes = HashSet::new();
    let source_table = manifest.source_table();
    let mut source_resident_bytes = HashMap::with_capacity(manifest.sources.len());
    let mut wgsl_validation_results = HashMap::with_capacity(manifest.sources.len());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        let Some(source) = source_table.source_for(request) else {
            report.record_failure_variant(
                variant_index,
                &request.key,
                format!(
                    "shader prewarm variant references missing source {}",
                    request.source_id.as_str()
                ),
            );
            continue;
        };
        let source_bytes = *source_resident_bytes
            .entry(source.id.as_str())
            .or_insert_with(|| source.resident_bytes());
        if source_bytes > budget.max_in_flight_source_bytes {
            report.execution_budget.record_rejected();
            report.record_failure_request(
                variant_index,
                request,
                source,
                format!(
                    "shader prewarm source {} requires {source_bytes} in-flight bytes; budget is {}",
                    source.id.as_str(),
                    budget.max_in_flight_source_bytes
                ),
            );
            if wgpu_module_validation_enabled {
                report.record_wgpu_module_validation_skipped();
            }
            if wgpu_pipeline_validation_enabled {
                report.record_wgpu_pipeline_validation_skipped();
            }
            continue;
        }
        report.execution_budget.record_started_work(source_bytes);
        if let Err(error) = validate_source_once(&mut wgsl_validation_results, source, |source| {
            validate_shader_variant_prewarm_wgsl(&source.wgsl_source)
                .map(|_| ())
                .map_err(|error| format!("{error:?}"))
        }) {
            report.record_failure_request(
                variant_index,
                request,
                source,
                format!("shader variant WGSL validation failed: {error}"),
            );
            if wgpu_module_validation_enabled {
                report.record_wgpu_module_validation_skipped();
            }
            if wgpu_pipeline_validation_enabled {
                report.record_wgpu_pipeline_validation_skipped();
            }
            continue;
        }

        if let Some(validate_module) = validate_module {
            if let Err(error) = validate_module(request, source) {
                report.record_failure_request(
                    variant_index,
                    request,
                    source,
                    format!("WGPU shader module validation failed: {error}"),
                );
                report.record_wgpu_module_validation_failed();
                if wgpu_pipeline_validation_enabled {
                    report.record_wgpu_pipeline_validation_skipped();
                }
                continue;
            }
            report.record_wgpu_module_validation_passed();
        }

        if let Some(validate_pipeline) = validate_pipeline {
            if let Err(error) = validate_pipeline(request, source) {
                report.record_failure_request(
                    variant_index,
                    request,
                    source,
                    format!("WGPU render pipeline validation failed: {error}"),
                );
                report.record_wgpu_pipeline_validation_failed();
                continue;
            }
            report.record_wgpu_pipeline_validation_passed();
        }

        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            source.include_content_hashes.iter().map(String::as_str),
        );
        if written_disk_hashes.contains(&disk_key.hash) {
            report.record_written_cache_entry(
                request,
                source,
                disk_key.hash,
                disk_key.canonical_string,
            );
            continue;
        }
        match cache.write(
            &disk_key,
            &source.wgsl_source,
            &source.template_revision,
            &source.naga_version,
            &source.wgpu_version,
        ) {
            Ok(_) => {
                written_disk_hashes.insert(disk_key.hash.clone());
                report.record_written_cache_entry(
                    request,
                    source,
                    disk_key.hash,
                    disk_key.canonical_string,
                )
            }
            Err(error) => {
                report.record_failure_request(variant_index, request, source, format!("{error:?}"))
            }
        }
    }
    report
}

fn validate_source_once<'a>(
    validation_results: &mut HashMap<&'a str, Result<(), String>>,
    source: &'a ShaderVariantPrewarmSource,
    validate: impl FnOnce(&ShaderVariantPrewarmSource) -> Result<(), String>,
) -> Result<(), String> {
    if let Some(result) = validation_results.get(source.id.as_str()) {
        return result.clone();
    }
    let result = validate(source);
    validation_results.insert(source.id.as_str(), result.clone());
    result
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::collections::HashMap;

    use crate::core::framework::render::ShaderVariantPrewarmSource;

    use super::validate_source_once;

    #[test]
    fn shared_source_wgsl_validation_is_cached_once_per_prewarm_batch() {
        let source = ShaderVariantPrewarmSource::new(
            "res://materials/shared.wgsl",
            "fn main() {}",
            Vec::new(),
            "template-r1",
            "naga-r1",
            "wgpu-r1",
        );
        let mut validation_results = HashMap::new();
        let validation_count = Cell::new(0usize);

        validate_source_once(&mut validation_results, &source, |_| {
            validation_count.set(validation_count.get() + 1);
            Ok(())
        })
        .expect("first validation should pass");
        validate_source_once(&mut validation_results, &source, |_| {
            validation_count.set(validation_count.get() + 1);
            Ok(())
        })
        .expect("cached validation should pass");

        assert_eq!(validation_count.get(), 1);
    }
}
