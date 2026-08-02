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
mod tests {
    use std::fs;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmManifest,
        ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource, ShadingModelId,
        GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;
    use crate::graphics::shader::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

    use super::{
        prewarm_shader_variants_to_disk, prewarm_shader_variants_to_disk_with_budget,
        prewarm_shader_variants_to_disk_with_module_validation,
        prewarm_shader_variants_to_disk_with_pipeline_validation,
    };

    mod combined_validation_tests;

    const VALID_WGSL: &str = "fn main() {}";

    fn test_manifest(
        key: ShaderVariantKey,
        source_label: &str,
        wgsl_source: &str,
        include_content_hashes: Vec<String>,
    ) -> ShaderVariantPrewarmManifest {
        let source = ShaderVariantPrewarmSource::new(
            source_label,
            wgsl_source,
            include_content_hashes,
            "template-r1",
            "naga-test",
            "wgpu-test",
        );
        let request = ShaderVariantPrewarmRequest {
            key,
            pipeline_state: None,
            source_id: source.id.clone(),
        };
        ShaderVariantPrewarmManifest::new(vec![source], vec![request])
    }

    fn test_disk_key(manifest: &ShaderVariantPrewarmManifest) -> ShaderVariantCacheDiskKey {
        let request = manifest.variants.first().expect("prewarm test request");
        let source = manifest.source_for(request).expect("prewarm test source");
        ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            source.include_content_hashes.iter().map(String::as_str),
        )
    }

    #[test]
    fn render_shader_variant_prewarm_writes_disk_entries() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-test.wgsl",
            VALID_WGSL,
            vec!["include-a".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report = prewarm_shader_variants_to_disk(&manifest, &root);

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.written_variants.len(), 1);
        assert_eq!(report.written_variants[0].cache_hash, disk_key.hash);
        assert_eq!(
            report.written_variants[0].canonical_string,
            disk_key.canonical_string
        );
        assert_eq!(
            report.written_variants[0].source_label,
            "res://materials/prewarm-test.wgsl"
        );
        assert_eq!(
            report
                .dimension_summary
                .pass_types
                .get("forward")
                .expect("forward pass count")
                .written_count,
            1
        );
        assert_eq!(
            report
                .dimension_summary
                .geometry_source_ids
                .get("0")
                .expect("static geometry count")
                .requested_count,
            1
        );
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_shares_one_source_artifact_across_variants() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_source_table_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let source = ShaderVariantPrewarmSource::new(
            "res://materials/source-table.wgsl",
            VALID_WGSL,
            vec!["include-source-table".to_string()],
            "template-r1",
            "naga-test",
            "wgpu-test",
        );
        let resident_source_bytes = source.resident_bytes();
        let manifest = ShaderVariantPrewarmManifest::new(
            vec![source.clone()],
            vec![
                ShaderVariantPrewarmRequest {
                    key: variant_key(),
                    pipeline_state: None,
                    source_id: source.id.clone(),
                },
                ShaderVariantPrewarmRequest {
                    key: variant_key_for(
                        GEOMETRY_SOURCE_ID_SKINNED_MESH,
                        ShaderPassType::Forward,
                        ShaderQualityTier::High,
                    ),
                    pipeline_state: None,
                    source_id: source.id.clone(),
                },
            ],
        );

        let report = prewarm_shader_variants_to_disk_with_budget(
            &manifest,
            &root,
            ShaderVariantPrewarmExecutionBudget {
                max_in_flight_variants: 1,
                max_in_flight_source_bytes: resident_source_bytes,
                max_resident_source_bytes: resident_source_bytes,
            },
        );

        assert_eq!(report.requested_count, 2);
        assert_eq!(report.written_count, 2);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.source_provenance.source_count, 1);
        assert_eq!(report.source_provenance.variant_count, 2);
        assert_eq!(
            report.execution_budget.resident_source_bytes,
            resident_source_bytes
        );
        assert_eq!(report.execution_budget.peak_in_flight_variants, 1);
        assert_eq!(
            report.execution_budget.peak_in_flight_source_bytes,
            resident_source_bytes
        );
        assert_eq!(report.execution_budget.rejected_count, 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_rejects_source_table_over_resident_budget() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_budget_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/budget-test.wgsl",
            VALID_WGSL,
            vec!["include-budget".to_string()],
        );
        let resident_source_bytes = manifest.sources[0].resident_bytes();

        let report = prewarm_shader_variants_to_disk_with_budget(
            &manifest,
            &root,
            ShaderVariantPrewarmExecutionBudget {
                max_in_flight_variants: 1,
                max_in_flight_source_bytes: resident_source_bytes,
                max_resident_source_bytes: resident_source_bytes - 1,
            },
        );

        assert_eq!(report.written_count, 0);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.execution_budget.rejected_count, 1);
        assert_eq!(
            report.execution_budget.resident_source_bytes,
            resident_source_bytes
        );
        assert_eq!(report.execution_budget.peak_in_flight_variants, 0);
        assert!(!root.exists());
    }

    #[test]
    fn render_shader_variant_prewarm_custom_ids_survive_disk_lookup() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_custom_id_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key_for_custom_ids(4, 16),
            "res://materials/prewarm-custom-id.wgsl",
            VALID_WGSL,
            vec!["include-custom-id".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report = prewarm_shader_variants_to_disk(&manifest, &root);

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.written_variants.len(), 1);
        assert!(report.written_variants[0]
            .canonical_string
            .contains("|geometry=4|"));
        assert!(report.written_variants[0]
            .canonical_string
            .contains("|shading=16|"));
        assert_eq!(
            report
                .dimension_summary
                .geometry_source_ids
                .get("4")
                .expect("custom geometry source count")
                .written_count,
            1
        );
        assert_eq!(
            report
                .dimension_summary
                .shading_model_ids
                .get("16")
                .expect("custom shading model count")
                .written_count,
            1
        );

        match ShaderVariantCacheDisk::new(&root).lookup(&disk_key) {
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(entry) => {
                assert_eq!(entry.meta.canonical_string, disk_key.canonical_string);
                assert!(entry.meta.canonical_string.contains("|geometry=4|"));
                assert!(entry.meta.canonical_string.contains("|shading=16|"));
            }
            other => panic!("expected custom id staged cache hit, got {other:?}"),
        }
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_custom_ids_hit_staged_fallback_root() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_custom_id_fallback_test_{}",
            std::process::id()
        ));
        let runtime_root = root.join("runtime").join("shader_variants");
        let staged_root = root.join("staged").join("cache").join("shader_variants");
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key_for_custom_ids(4, 16),
            "res://materials/prewarm-custom-id-fallback.wgsl",
            VALID_WGSL,
            vec!["include-custom-id-fallback".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report = prewarm_shader_variants_to_disk(&manifest, &staged_root);

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert!(matches!(
            ShaderVariantCacheDisk::new(&runtime_root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Miss
        ));
        match ShaderVariantCacheDisk::with_fallback_roots(&runtime_root, [&staged_root])
            .lookup(&disk_key)
        {
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(entry) => {
                assert_eq!(entry.meta.canonical_string, disk_key.canonical_string);
                assert!(entry.meta.canonical_string.contains("|geometry=4|"));
                assert!(entry.meta.canonical_string.contains("|shading=16|"));
            }
            other => panic!("expected custom id staged fallback cache hit, got {other:?}"),
        }
        assert!(
            !runtime_root.exists(),
            "fallback cache lookup must not create or write the runtime cache root"
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_rejects_invalid_wgsl_before_disk_write() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_invalid_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-invalid.wgsl",
            "fn main(",
            vec!["include-invalid".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report = prewarm_shader_variants_to_disk(&manifest, &root);

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 0);
        assert_eq!(report.failed_count, 1);
        assert!(report.written_variants.is_empty());
        assert_eq!(
            report
                .dimension_summary
                .pass_types
                .get("forward")
                .expect("forward pass count")
                .failed_count,
            1
        );
        assert_eq!(
            report
                .dimension_summary
                .shading_model_ids
                .get("2")
                .expect("standard pbr count")
                .requested_count,
            1
        );
        assert_eq!(report.failures[0].variant_index, 0);
        assert!(report.failures[0]
            .error
            .contains("shader variant WGSL validation failed"));
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Miss
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_rejects_wgpu_module_validation_failure_before_disk_write() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_wgpu_validation_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-wgpu-failure.wgsl",
            VALID_WGSL,
            vec!["include-wgpu-validation".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report =
            prewarm_shader_variants_to_disk_with_module_validation(&manifest, &root, |_, _| {
                Err("mock WGPU module failure".to_string())
            });

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 0);
        assert_eq!(report.failed_count, 1);
        assert!(report.written_variants.is_empty());
        assert!(report.wgpu_module_validation.enabled);
        assert_eq!(report.wgpu_module_validation.requested_count, 1);
        assert_eq!(report.wgpu_module_validation.validated_count, 0);
        assert_eq!(report.wgpu_module_validation.failed_count, 1);
        assert_eq!(report.wgpu_module_validation.skipped_count, 0);
        assert!(report.failures[0]
            .error
            .contains("WGPU shader module validation failed"));
        assert_eq!(
            report
                .dimension_summary
                .pass_types
                .get("forward")
                .expect("forward pass count")
                .failed_count,
            1
        );
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Miss
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_records_wgpu_module_validation_success() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_wgpu_validation_success_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-wgpu-success.wgsl",
            VALID_WGSL,
            vec!["include-wgpu-validation-success".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report =
            prewarm_shader_variants_to_disk_with_module_validation(&manifest, &root, |_, _| Ok(()));

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.written_variants.len(), 1);
        assert_eq!(report.written_variants[0].cache_hash, disk_key.hash);
        assert_eq!(
            report.written_variants[0].canonical_string,
            disk_key.canonical_string
        );
        assert!(report.wgpu_module_validation.enabled);
        assert_eq!(report.wgpu_module_validation.requested_count, 1);
        assert_eq!(report.wgpu_module_validation.validated_count, 1);
        assert_eq!(report.wgpu_module_validation.failed_count, 0);
        assert_eq!(report.wgpu_module_validation.skipped_count, 0);
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_rejects_wgpu_pipeline_validation_failure_before_disk_write() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_wgpu_pipeline_validation_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-wgpu-pipeline-failure.wgsl",
            VALID_WGSL,
            vec!["include-wgpu-pipeline-validation".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report =
            prewarm_shader_variants_to_disk_with_pipeline_validation(&manifest, &root, |_, _| {
                Err("mock WGPU pipeline failure".to_string())
            });

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 0);
        assert_eq!(report.failed_count, 1);
        assert!(report.written_variants.is_empty());
        assert!(report.wgpu_pipeline_validation.enabled);
        assert_eq!(report.wgpu_pipeline_validation.requested_count, 1);
        assert_eq!(report.wgpu_pipeline_validation.validated_count, 0);
        assert_eq!(report.wgpu_pipeline_validation.failed_count, 1);
        assert_eq!(report.wgpu_pipeline_validation.skipped_count, 0);
        assert!(report.failures[0]
            .error
            .contains("WGPU render pipeline validation failed"));
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Miss
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_records_wgpu_pipeline_validation_success() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_wgpu_pipeline_validation_success_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = test_manifest(
            variant_key(),
            "res://materials/prewarm-wgpu-pipeline-success.wgsl",
            VALID_WGSL,
            vec!["include-wgpu-pipeline-validation-success".to_string()],
        );
        let disk_key = test_disk_key(&manifest);

        let report =
            prewarm_shader_variants_to_disk_with_pipeline_validation(&manifest, &root, |_, _| {
                Ok(())
            });

        assert_eq!(report.requested_count, 1);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.written_variants.len(), 1);
        assert_eq!(report.written_variants[0].cache_hash, disk_key.hash);
        assert!(report.wgpu_pipeline_validation.enabled);
        assert_eq!(report.wgpu_pipeline_validation.requested_count, 1);
        assert_eq!(report.wgpu_pipeline_validation.validated_count, 1);
        assert_eq!(report.wgpu_pipeline_validation.failed_count, 0);
        assert_eq!(report.wgpu_pipeline_validation.skipped_count, 0);
        assert!(matches!(
            ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
            super::super::disk::ShaderVariantCacheDiskLookup::Hit(_)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_shader_variant_prewarm_report_groups_written_and_failed_dimensions() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_dimensions_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let valid_source = ShaderVariantPrewarmSource::new(
            "res://materials/prewarm-valid.wgsl",
            VALID_WGSL,
            vec!["include-valid".to_string()],
            "template-r1",
            "naga-test",
            "wgpu-test",
        );
        let invalid_source = ShaderVariantPrewarmSource::new(
            "res://materials/prewarm-invalid.wgsl",
            "fn main(",
            vec!["include-invalid".to_string()],
            "template-r1",
            "naga-test",
            "wgpu-test",
        );
        let manifest = ShaderVariantPrewarmManifest::new(
            vec![valid_source.clone(), invalid_source.clone()],
            vec![
                ShaderVariantPrewarmRequest {
                    key: variant_key(),
                    pipeline_state: None,
                    source_id: valid_source.id,
                },
                ShaderVariantPrewarmRequest {
                    key: variant_key_for(
                        GEOMETRY_SOURCE_ID_SKINNED_MESH,
                        ShaderPassType::Shadow,
                        ShaderQualityTier::High,
                    ),
                    pipeline_state: None,
                    source_id: invalid_source.id,
                },
            ],
        );

        let report = prewarm_shader_variants_to_disk(&manifest, &root);

        assert_eq!(report.requested_count, 2);
        assert_eq!(report.written_count, 1);
        assert_eq!(report.failed_count, 1);
        assert_eq!(report.written_variants.len(), 1);
        assert_eq!(
            report.written_variants[0].source_label,
            "res://materials/prewarm-valid.wgsl"
        );
        assert_eq!(report.written_variants[0].template_revision, "template-r1");
        assert_eq!(
            report.dimension_summary.pass_types["forward"].written_count,
            1
        );
        assert_eq!(
            report.dimension_summary.pass_types["shadow"].failed_count,
            1
        );
        assert_eq!(
            report.dimension_summary.geometry_source_ids["0"].written_count,
            1
        );
        assert_eq!(
            report.dimension_summary.geometry_source_ids["1"].failed_count,
            1
        );
        assert_eq!(
            report.dimension_summary.shading_model_ids["2"].requested_count,
            2
        );
        assert_eq!(
            report.dimension_summary.quality_tiers["medium"].written_count,
            1
        );
        assert_eq!(
            report.dimension_summary.quality_tiers["high"].failed_count,
            1
        );
        assert_eq!(report.source_provenance.source_count, 2);
        assert_eq!(report.source_provenance.variant_count, 2);
        let written_source = report
            .source_provenance
            .sources
            .values()
            .find(|entry| entry.source_label == "res://materials/prewarm-valid.wgsl")
            .expect("written source provenance");
        assert_eq!(written_source.requested_count, 1);
        assert_eq!(written_source.written_count, 1);
        assert_eq!(written_source.failed_count, 0);
        assert_eq!(written_source.include_content_hashes, ["include-valid"]);
        assert_eq!(written_source.template_revision, "template-r1");
        assert_eq!(written_source.source_hash.len(), 64);
        let failed_source = report
            .source_provenance
            .sources
            .values()
            .find(|entry| entry.source_label == "res://materials/prewarm-invalid.wgsl")
            .expect("failed source provenance");
        assert_eq!(failed_source.requested_count, 1);
        assert_eq!(failed_source.written_count, 0);
        assert_eq!(failed_source.failed_count, 1);
        assert_eq!(failed_source.include_content_hashes, ["include-invalid"]);
        assert_eq!(failed_source.template_revision, "template-r1");
        assert_ne!(written_source.source_hash, failed_source.source_hash);
        let _ = fs::remove_dir_all(root);
    }

    fn variant_key() -> ShaderVariantKey {
        variant_key_for(
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            ShaderPassType::Forward,
            ShaderQualityTier::Medium,
        )
    }

    fn variant_key_for(
        geometry_source: GeometrySourceId,
        pass_type: ShaderPassType,
        quality: ShaderQualityTier,
    ) -> ShaderVariantKey {
        ShaderVariantKey {
            material_shader: ResourceId::from_stable_label("res://materials/prewarm-test.wgsl"),
            material_revision: 3,
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source,
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type,
            features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
            quality,
            platform_token: "wgpu-test".to_string(),
        }
    }

    fn variant_key_for_custom_ids(geometry_source: u8, shading_model: u8) -> ShaderVariantKey {
        let mut key = variant_key();
        key.geometry_source = GeometrySourceId::new(geometry_source);
        key.shading_model = ShadingModelId::new(shading_model);
        key
    }
}
