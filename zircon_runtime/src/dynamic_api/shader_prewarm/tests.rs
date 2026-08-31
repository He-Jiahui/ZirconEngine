use std::borrow::Cow;
use std::fs;

use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShaderVariantPrewarmExecutionBudget,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH, SHADING_MODEL_ID_BLINN_PHONG,
};
use crate::graphics::shader::{
    ShaderVariantCacheDisk, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
};

use super::{
    builtin_fallback_shader_prewarm_manifest, builtin_standard_material_shader_prewarm_manifest,
    builtin_standard_material_shader_prewarm_manifest_for_geometry, prewarm_shader_variants,
    prewarm_shader_variants_with_execution_budget, register_prewarm_source,
};

fn source_for<'a>(
    manifest: &'a ShaderVariantPrewarmManifest,
    request: &ShaderVariantPrewarmRequest,
) -> &'a ShaderVariantPrewarmSource {
    manifest
        .source_for(request)
        .expect("prewarm manifest source for request")
}

#[test]
fn prewarm_source_registry_deduplicates_content_across_provenance_labels() {
    let source = ShaderVariantPrewarmSource::new(
        "builtin://shader/pbr.wgsl",
        "fn main() {}",
        vec!["include-a".to_string()],
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    );
    let renamed = source.with_source_label("res://renamed/pbr.wgsl");
    let mut sources = Vec::new();

    let first_id = register_prewarm_source(&mut sources, source);
    let second_id = register_prewarm_source(&mut sources, renamed);

    assert_eq!(first_id, second_id);
    assert_eq!(sources.len(), 1);
}

#[test]
fn builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source() {
    let manifest = builtin_fallback_shader_prewarm_manifest();

    assert_eq!(manifest.variants.len(), 6);
    assert_eq!(
        manifest
            .variants
            .iter()
            .map(|request| request.key.pass_type.token())
            .collect::<Vec<_>>(),
        vec![
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask"
        ]
    );
    assert!(manifest.variants.iter().all(|request| {
        let source = source_for(&manifest, request);
        source.template_revision == "zr-material-template-v1"
            && source.source_label == "builtin://shader/pbr.wgsl"
            && source.include_content_hashes.len() > 1
    }));

    let forward_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::Forward)
        .expect("forward builtin fallback prewarm request");
    let forward_source = source_for(&manifest, forward_request);
    assert!(forward_source
        .wgsl_source
        .contains("fn zr_material_surface("));
    assert!(forward_source.wgsl_source.contains("fn vs_main("));
    assert!(forward_source.wgsl_source.contains("fn fs_main("));
    assert!(forward_source
        .wgsl_source
        .contains("// include: zr_light_grid.wgsl"));
    assert!(forward_source
        .wgsl_source
        .contains("// include: zr_shadow.wgsl"));

    let depth_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::DepthPrepass)
        .expect("depth-only builtin fallback prewarm request");
    let depth_source = source_for(&manifest, depth_request);
    assert!(depth_source
        .wgsl_source
        .contains("// include: zr_template_depth.wgsl"));
    assert!(!depth_source.wgsl_source.contains("fn fs_main("));
    assert!(!depth_source.wgsl_source.contains("zr_material_surface"));
    assert!(!depth_source.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(!depth_source
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));

    let velocity_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::Velocity)
        .expect("velocity builtin fallback prewarm request");
    let velocity_source = source_for(&manifest, velocity_request);
    assert!(velocity_source.wgsl_source.contains("fetch_prev_position"));
    assert!(!velocity_source
        .wgsl_source
        .contains("fn vs_velocity_object"));

    let taa_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::TaaReactiveMask)
        .expect("TAA reactive mask builtin fallback prewarm request");
    let taa_source = source_for(&manifest, taa_request);
    assert!(taa_source
        .wgsl_source
        .contains("// include: zr_template_taa_reactive_mask.wgsl"));
    assert!(taa_source.wgsl_source.contains("fn fs_taa_reactive_mask("));
    assert!(taa_source
        .wgsl_source
        .contains("fn fs_taa_reactive_material_mask("));
    assert!(!taa_source
        .wgsl_source
        .contains("// include: zr_light_grid.wgsl"));
}

#[test]
fn builtin_standard_material_shader_prewarm_manifest_projects_material_features() {
    let manifest = builtin_standard_material_shader_prewarm_manifest(
        ShaderFeatureBits::new(
            ShaderFeatureBits::ALPHA_TEST
                | ShaderFeatureBits::DOUBLE_SIDED
                | ShaderFeatureBits::RECEIVE_SHADOWS,
        ),
        SHADING_MODEL_ID_BLINN_PHONG,
        Some(0.5),
        &[ShaderQualityTier::High, ShaderQualityTier::High],
    );

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        let source = source_for(&manifest, request);
        request.key.quality == ShaderQualityTier::High
            && request.key.shading_model == SHADING_MODEL_ID_BLINN_PHONG
            && request.key.features.contains(ShaderFeatureBits::ALPHA_TEST)
            && request
                .key
                .features
                .contains(ShaderFeatureBits::DOUBLE_SIDED)
            && request
                .key
                .features
                .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
            && source.template_revision == "zr-material-template-v1"
            && source.include_content_hashes.len() > 1
    }));
    assert_eq!(
        manifest
            .variants
            .iter()
            .map(|request| request.key.pass_type.token())
            .collect::<Vec<_>>(),
        vec![
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask"
        ]
    );

    let forward_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::Forward)
        .expect("forward standard material prewarm request");
    let forward_source = source_for(&manifest, forward_request);
    assert!(forward_source
        .wgsl_source
        .contains("fn zr_material_surface("));
    assert!(forward_source
        .wgsl_source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));

    let depth_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::DepthPrepass)
        .expect("alpha depth-only standard material prewarm request");
    let depth_source = source_for(&manifest, depth_request);
    assert!(depth_source
        .wgsl_source
        .contains("// include: zr_template_depth_alpha.wgsl"));
    assert!(depth_source.wgsl_source.contains("fn zr_material_surface("));
    assert!(depth_source
        .wgsl_source
        .contains("zr_apply_alpha_clip(surface);"));
    assert!(depth_source
        .wgsl_source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
    assert!(!depth_source.wgsl_source.contains("surface.normal_ws * 0.5"));
    assert!(!depth_source
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));
    assert_ne!(
        forward_source.include_content_hashes,
        depth_source.include_content_hashes
    );

    let taa_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::TaaReactiveMask)
        .expect("alpha TAA reactive mask standard material prewarm request");
    let taa_source = source_for(&manifest, taa_request);
    assert!(taa_source
        .wgsl_source
        .contains("// include: zr_template_taa_reactive_mask.wgsl"));
    assert!(taa_source
        .wgsl_source
        .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
}

#[test]
fn builtin_standard_material_shader_prewarm_manifest_projects_geometry_source() {
    let manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_BLINN_PHONG,
        None,
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        &[ShaderQualityTier::Medium],
    );

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        let source = source_for(&manifest, request);
        request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH
            && request.key.shading_model == SHADING_MODEL_ID_BLINN_PHONG
            && request
                .key
                .features
                .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
            && source
                .wgsl_source
                .contains("// include: zr_geometry_skinned.wgsl")
            && source
                .wgsl_source
                .contains("const ZR_GEOMETRY_SOURCE_SKINNED_MESH: bool = true;")
            && source
                .wgsl_source
                .contains("zr_skinned_joint_matrix(v.joints.x)")
            && source.template_revision == "zr-material-template-v1"
    }));
    assert_eq!(
        manifest
            .variants
            .iter()
            .map(|request| request.key.pass_type.token())
            .collect::<Vec<_>>(),
        vec![
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask"
        ]
    );

    let depth_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::DepthPrepass)
        .expect("skinned depth-only standard material prewarm request");
    let depth_source = source_for(&manifest, depth_request);
    assert!(depth_source
        .wgsl_source
        .contains("// include: zr_template_depth.wgsl"));
    assert!(!depth_source.wgsl_source.contains("fn fs_main("));
    assert!(!depth_source.wgsl_source.contains("zr_material_surface"));
    assert!(!depth_source.wgsl_source.contains("surface.normal_ws * 0.5"));
}

#[test]
fn builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules() {
    let manifest = builtin_standard_material_cache_validation_manifest();
    let root = std::env::temp_dir().join(format!(
        "zircon_builtin_standard_material_prewarm_cache_test_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);

    let report = prewarm_shader_variants(&manifest, &root);

    assert_eq!(report.requested_count, manifest.variants.len());
    assert_eq!(report.written_count, manifest.variants.len());
    assert_eq!(report.failed_count, 0);
    assert!(report.failures.is_empty());

    let restarted_cache = ShaderVariantCacheDisk::new(&root);
    let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
        let _ = fs::remove_dir_all(root);
        return;
    };
    let device = &backend.device;
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);

    for request in &manifest.variants {
        let source = source_for(&manifest, request);
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            &request.key,
            &source.source_hash,
            &source.include_content_hashes,
            &source.template_revision,
            &source.naga_version,
            &source.wgpu_version,
        );
        let entry = match restarted_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) => entry,
            other => panic!(
                "expected staged prewarm cache hit after restart for {}; got {other:?}",
                request.key.canonical_string()
            ),
        };
        assert_eq!(entry.wgsl_source, source.wgsl_source);
        assert_eq!(entry.meta.template_revision, source.template_revision);
        let _shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-staged-builtin-standard-material-prewarm-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(entry.wgsl_source)),
        });
    }

    let error = pollster::block_on(error_scope.pop());
    assert!(
        error.is_none(),
        "staged builtin standard material prewarm WGSL should create WGPU shader modules: {error:?}"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_execution_budget_is_reported_without_inventing_variant_failure() {
    let manifest = ShaderVariantPrewarmManifest::empty();
    let budget = ShaderVariantPrewarmExecutionBudget {
        max_in_flight_variants: 0,
        ..Default::default()
    };

    let report = prewarm_shader_variants_with_execution_budget(
        &manifest,
        std::env::temp_dir(),
        budget,
        false,
        false,
    );

    assert_eq!(report.requested_count, 0);
    assert_eq!(report.written_count, 0);
    assert_eq!(report.failed_count, 0);
    assert!(report.failures.is_empty());
    assert_eq!(
        report.preflight_error.as_deref(),
        Some("shader prewarm uses one serial WGPU worker; max_in_flight_variants must be 1, got 0")
    );
    assert_eq!(report.execution_budget.rejected_count, 1);
}

fn builtin_standard_material_cache_validation_manifest() -> ShaderVariantPrewarmManifest {
    let mut manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_BLINN_PHONG,
        None,
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        &[ShaderQualityTier::Medium],
    );
    let extra = builtin_standard_material_shader_prewarm_manifest_for_geometry(
        ShaderFeatureBits::new(ShaderFeatureBits::ALPHA_TEST | ShaderFeatureBits::RECEIVE_SHADOWS),
        SHADING_MODEL_ID_BLINN_PHONG,
        Some(0.42),
        GEOMETRY_SOURCE_ID_SKINNED_MESH,
        &[ShaderQualityTier::Medium],
    );
    manifest.sources.extend(extra.sources);
    manifest.variants.extend(extra.variants);
    manifest
}
