use std::{cell::Cell, fs};

use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantKey,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
    GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::ResourceId;
use crate::graphics::shader::{ShaderVariantCacheDisk, ShaderVariantCacheDiskKey};

use super::super::prewarm_shader_variants_to_disk_with_module_and_pipeline_validation;

const VALID_WGSL: &str = "fn main() {}";

#[test]
fn render_shader_variant_prewarm_records_combined_wgpu_validation_success() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_prewarm_wgpu_combined_validation_success_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = test_source(
        "res://materials/prewarm-wgpu-combined-success.wgsl",
        vec!["include-wgpu-combined-validation-success".to_string()],
    );
    let request = test_request(&source);
    let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
        &request.key,
        source.include_content_hashes.iter().map(String::as_str),
    );
    let manifest = ShaderVariantPrewarmManifest::new(vec![source], vec![request]);
    let module_validation_calls = Cell::new(0);
    let pipeline_validation_calls = Cell::new(0);

    let report = prewarm_shader_variants_to_disk_with_module_and_pipeline_validation(
        &manifest,
        &root,
        |_, _| {
            module_validation_calls.set(module_validation_calls.get() + 1);
            Ok(())
        },
        |_, _| {
            pipeline_validation_calls.set(pipeline_validation_calls.get() + 1);
            Ok(())
        },
    );

    assert_eq!(module_validation_calls.get(), 1);
    assert_eq!(pipeline_validation_calls.get(), 1);
    assert_eq!(report.requested_count, 1);
    assert_eq!(report.written_count, 1);
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.written_variants[0].cache_hash, disk_key.hash);
    assert!(report.wgpu_module_validation.enabled);
    assert_eq!(report.wgpu_module_validation.requested_count, 1);
    assert_eq!(report.wgpu_module_validation.validated_count, 1);
    assert_eq!(report.wgpu_module_validation.failed_count, 0);
    assert_eq!(report.wgpu_module_validation.skipped_count, 0);
    assert!(report.wgpu_pipeline_validation.enabled);
    assert_eq!(report.wgpu_pipeline_validation.requested_count, 1);
    assert_eq!(report.wgpu_pipeline_validation.validated_count, 1);
    assert_eq!(report.wgpu_pipeline_validation.failed_count, 0);
    assert_eq!(report.wgpu_pipeline_validation.skipped_count, 0);
    assert!(matches!(
        ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
        super::super::super::disk::ShaderVariantCacheDiskLookup::Hit(_)
    ));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn render_shader_variant_prewarm_skips_pipeline_after_module_validation_failure() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_variant_prewarm_wgpu_combined_validation_failure_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source = test_source(
        "res://materials/prewarm-wgpu-combined-failure.wgsl",
        vec!["include-wgpu-combined-validation-failure".to_string()],
    );
    let request = test_request(&source);
    let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
        &request.key,
        source.include_content_hashes.iter().map(String::as_str),
    );
    let manifest = ShaderVariantPrewarmManifest::new(vec![source], vec![request]);

    let report = prewarm_shader_variants_to_disk_with_module_and_pipeline_validation(
        &manifest,
        &root,
        |_, _| Err("mock WGPU module failure".to_string()),
        |_, _| panic!("pipeline validation should not run after module validation fails"),
    );

    assert_eq!(report.requested_count, 1);
    assert_eq!(report.written_count, 0);
    assert_eq!(report.failed_count, 1);
    assert!(report.wgpu_module_validation.enabled);
    assert_eq!(report.wgpu_module_validation.requested_count, 1);
    assert_eq!(report.wgpu_module_validation.validated_count, 0);
    assert_eq!(report.wgpu_module_validation.failed_count, 1);
    assert_eq!(report.wgpu_module_validation.skipped_count, 0);
    assert!(report.wgpu_pipeline_validation.enabled);
    assert_eq!(report.wgpu_pipeline_validation.requested_count, 1);
    assert_eq!(report.wgpu_pipeline_validation.validated_count, 0);
    assert_eq!(report.wgpu_pipeline_validation.failed_count, 0);
    assert_eq!(report.wgpu_pipeline_validation.skipped_count, 1);
    assert!(matches!(
        ShaderVariantCacheDisk::new(&root).lookup(&disk_key),
        super::super::super::disk::ShaderVariantCacheDiskLookup::Miss
    ));
    let _ = fs::remove_dir_all(root);
}

fn test_source(
    source_label: &str,
    include_content_hashes: Vec<String>,
) -> ShaderVariantPrewarmSource {
    ShaderVariantPrewarmSource::new(
        source_label,
        VALID_WGSL,
        include_content_hashes,
        "template-r1",
        "naga-test",
        "wgpu-test",
    )
}

fn test_request(source: &ShaderVariantPrewarmSource) -> ShaderVariantPrewarmRequest {
    ShaderVariantPrewarmRequest {
        key: variant_key(),
        pipeline_state: None,
        source_id: source.id.clone(),
    }
}

fn variant_key() -> ShaderVariantKey {
    ShaderVariantKey {
        material_shader: ResourceId::from_stable_label("res://materials/prewarm-test.wgsl"),
        material_revision: 3,
        material_layout_hash: 0,
        material_option_bits: 0,
        geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
        shading_model: SHADING_MODEL_ID_STANDARD_PBR,
        pass_type: ShaderPassType::Forward,
        features: ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST),
        quality: ShaderQualityTier::Medium,
        platform_token: "wgpu-test".to_string(),
    }
}
