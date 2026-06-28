use std::path::Path;

use crate::core::framework::render::{
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport, ShaderVariantPrewarmRequest,
};
use crate::graphics::shader::template::validate_shader_variant_prewarm_wgsl;

use super::disk::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

pub(crate) fn prewarm_shader_variants_to_disk(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_inner(manifest, cache_root, None)
}

pub(crate) fn prewarm_shader_variants_to_disk_with_module_validation(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    validate_module: impl Fn(&ShaderVariantPrewarmRequest) -> Result<(), String>,
) -> ShaderVariantPrewarmReport {
    prewarm_shader_variants_to_disk_inner(manifest, cache_root, Some(&validate_module))
}

fn prewarm_shader_variants_to_disk_inner(
    manifest: &ShaderVariantPrewarmManifest,
    cache_root: impl AsRef<Path>,
    validate_module: Option<&dyn Fn(&ShaderVariantPrewarmRequest) -> Result<(), String>>,
) -> ShaderVariantPrewarmReport {
    let mut report = ShaderVariantPrewarmReport::default();
    let wgpu_module_validation_enabled = validate_module.is_some();
    if wgpu_module_validation_enabled {
        report.enable_wgpu_module_validation(manifest.variants.len());
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

    let cache = ShaderVariantCacheDisk::new(cache_root.as_ref());
    for (variant_index, request) in manifest.variants.iter().enumerate() {
        if let Err(error) = validate_shader_variant_prewarm_wgsl(&request.wgsl_source) {
            report.record_failure_request(
                variant_index,
                request,
                format!("shader variant WGSL validation failed: {error:?}"),
            );
            if wgpu_module_validation_enabled {
                report.record_wgpu_module_validation_skipped();
            }
            continue;
        }

        if let Some(validate_module) = validate_module {
            if let Err(error) = validate_module(request) {
                report.record_failure_request(
                    variant_index,
                    request,
                    format!("WGPU shader module validation failed: {error}"),
                );
                report.record_wgpu_module_validation_failed();
                continue;
            }
            report.record_wgpu_module_validation_passed();
        }

        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        match cache.write(
            &disk_key,
            &request.wgsl_source,
            &request.template_revision,
            &request.naga_version,
            &request.wgpu_version,
        ) {
            Ok(_) => {
                report.record_written_cache_entry(request, disk_key.hash, disk_key.canonical_string)
            }
            Err(error) => {
                report.record_failure_request(variant_index, request, format!("{error:?}"))
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::core::framework::render::{
        GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
        ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShadingModelId,
        GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
        SHADING_MODEL_ID_STANDARD_PBR,
    };
    use crate::core::resource::ResourceId;
    use crate::graphics::shader::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

    use super::{
        prewarm_shader_variants_to_disk, prewarm_shader_variants_to_disk_with_module_validation,
    };

    const VALID_WGSL: &str = "fn main() {}";

    #[test]
    fn render_shader_variant_prewarm_writes_disk_entries() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let request = ShaderVariantPrewarmRequest {
            key: variant_key(),
            source_label: "res://materials/prewarm-test.wgsl".to_string(),
            wgsl_source: "fn main() {}".to_string(),
            include_content_hashes: vec!["include-a".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

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
    fn render_shader_variant_prewarm_custom_ids_survive_disk_lookup() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_custom_id_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let request = ShaderVariantPrewarmRequest {
            key: variant_key_for_custom_ids(4, 16),
            source_label: "res://materials/prewarm-custom-id.wgsl".to_string(),
            wgsl_source: VALID_WGSL.to_string(),
            include_content_hashes: vec!["include-custom-id".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

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
        let request = ShaderVariantPrewarmRequest {
            key: variant_key_for_custom_ids(4, 16),
            source_label: "res://materials/prewarm-custom-id-fallback.wgsl".to_string(),
            wgsl_source: VALID_WGSL.to_string(),
            include_content_hashes: vec!["include-custom-id-fallback".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

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
        let request = ShaderVariantPrewarmRequest {
            key: variant_key(),
            source_label: "res://materials/prewarm-invalid.wgsl".to_string(),
            wgsl_source: "fn main(".to_string(),
            include_content_hashes: vec!["include-invalid".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

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
        let request = ShaderVariantPrewarmRequest {
            key: variant_key(),
            source_label: "res://materials/prewarm-wgpu-failure.wgsl".to_string(),
            wgsl_source: VALID_WGSL.to_string(),
            include_content_hashes: vec!["include-wgpu-validation".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

        let report =
            prewarm_shader_variants_to_disk_with_module_validation(&manifest, &root, |_| {
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
        let request = ShaderVariantPrewarmRequest {
            key: variant_key(),
            source_label: "res://materials/prewarm-wgpu-success.wgsl".to_string(),
            wgsl_source: VALID_WGSL.to_string(),
            include_content_hashes: vec!["include-wgpu-validation-success".to_string()],
            template_revision: "template-r1".to_string(),
            naga_version: "naga-test".to_string(),
            wgpu_version: "wgpu-test".to_string(),
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            request.include_content_hashes.iter().map(String::as_str),
        );
        let manifest = ShaderVariantPrewarmManifest::new(vec![request]);

        let report =
            prewarm_shader_variants_to_disk_with_module_validation(&manifest, &root, |_| Ok(()));

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
    fn render_shader_variant_prewarm_report_groups_written_and_failed_dimensions() {
        let root = std::env::temp_dir().join(format!(
            "zircon_shader_variant_prewarm_dimensions_test_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let manifest = ShaderVariantPrewarmManifest::new(vec![
            ShaderVariantPrewarmRequest {
                key: variant_key(),
                source_label: "res://materials/prewarm-valid.wgsl".to_string(),
                wgsl_source: "fn main() {}".to_string(),
                include_content_hashes: vec!["include-valid".to_string()],
                template_revision: "template-r1".to_string(),
                naga_version: "naga-test".to_string(),
                wgpu_version: "wgpu-test".to_string(),
            },
            ShaderVariantPrewarmRequest {
                key: variant_key_for(
                    GEOMETRY_SOURCE_ID_SKINNED_MESH,
                    ShaderPassType::Shadow,
                    ShaderQualityTier::High,
                ),
                source_label: "res://materials/prewarm-invalid.wgsl".to_string(),
                wgsl_source: "fn main(".to_string(),
                include_content_hashes: vec!["include-invalid".to_string()],
                template_revision: "template-r1".to_string(),
                naga_version: "naga-test".to_string(),
                wgpu_version: "wgpu-test".to_string(),
            },
        ]);

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
