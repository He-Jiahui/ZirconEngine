use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use zircon_runtime::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderPipelinePrewarmState, ShaderQualityTier,
    ShaderVariantPrewarmManifest, ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
    ShadingModelId, GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR, SHADING_MODEL_ID_UNLIT,
    SHADING_MODEL_PLUGIN_ID_START,
};

mod asset_scan_errors;
mod geometry_sources;
mod io;
mod module_dependencies;
mod raw_revision;
mod resource_registry;

use super::{
    asset_root_manifest, asset_root_manifest_for_quality_tiers_and_geometry_sources,
    asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids,
    dedupe_prewarm_manifest,
};

const BUILTIN_MATERIAL_PASS_TYPES: [ShaderPassType; 6] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
    ShaderPassType::TaaReactiveMask,
];

fn source_for<'a>(
    manifest: &'a ShaderVariantPrewarmManifest,
    request: &ShaderVariantPrewarmRequest,
) -> &'a ShaderVariantPrewarmSource {
    manifest
        .source_for(request)
        .expect("prewarm manifest source for request")
}

fn request_for_source_label<'a>(
    manifest: &'a ShaderVariantPrewarmManifest,
    suffix: &str,
) -> &'a ShaderVariantPrewarmRequest {
    manifest
        .variants
        .iter()
        .find(|request| source_for(manifest, request).source_label.ends_with(suffix))
        .expect("prewarm manifest request for source label")
}

#[test]
fn shader_prewarm_asset_root_manifest_reads_compound_zshader_package() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_asset_scan_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/example")).unwrap();
    fs::write(
        root.join("shaders/example.zmeta"),
        r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000041"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_digest = "scan-test-hash"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/example/example.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["base.wgsl", "variant.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(root.join("shaders/example/base.wgsl"), "fn base() {}\n").unwrap();
    fs::write(
        root.join("shaders/example/variant.wgsl"),
        "fn variant() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("materials")).unwrap();
    fs::write(
        root.join("materials/example.zmaterial"),
        r#"version = 2
name = "Example"

[shader]
uuid = "00000000-0000-0000-0000-000000000041"
url = "res://shaders/example"

[overrides]
double_sided = true
lighting_model = "blinn_phong"

[overrides.alpha_mode]
mode = "mask"
cutoff = 0.5
"#,
    )
    .unwrap();
    fs::write(
        root.join("materials/transparent.zmaterial"),
        r#"version = 2
name = "Transparent"

[shader]
uuid = "00000000-0000-0000-0000-000000000041"
url = "res://shaders/example"

[overrides]
double_sided = true
lighting_model = "unlit"

[overrides.alpha_mode]
mode = "blend"
"#,
    )
    .unwrap();

    let manifest = asset_root_manifest(&root).unwrap();

    assert_eq!(manifest.variants.len(), 13);
    let request = &manifest.variants[0];
    let source = source_for(&manifest, request);
    assert_eq!(source.source_label, "res://shaders/example");
    assert!(source.wgsl_source.contains("fn base() {}"));
    assert!(source.wgsl_source.contains("fn variant() {}"));
    assert!(
        source.include_content_hashes.len() >= 2,
        "compound shader package should keep hashes for its primary WGSL files"
    );
    assert!(
        source.include_content_hashes.len() > 2,
        "compound shader package should include module dependency hashes in the revision surface"
    );
    assert_eq!(request.key.platform_token, "wgpu-runtime");
    assert_ne!(request.key.material_revision, 0);
    let passes = manifest
        .variants
        .iter()
        .map(|request| request.key.pass_type.token())
        .collect::<Vec<_>>();
    assert_eq!(
        &passes[..6],
        vec![
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask"
        ]
    );
    assert_eq!(
        &passes[6..12],
        vec![
            "forward",
            "gbuffer",
            "depth_prepass",
            "shadow",
            "velocity",
            "taa_reactive_mask"
        ]
    );
    assert_eq!(passes[12], "forward");
    let material_feature_bits = ShaderFeatureBits::ALPHA_TEST
        | ShaderFeatureBits::DOUBLE_SIDED
        | ShaderFeatureBits::RECEIVE_SHADOWS;
    assert!(manifest.variants[..6]
        .iter()
        .all(|request| request.key.features.bits() == 0));
    assert!(manifest.variants[..6]
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_STANDARD_PBR));
    assert!(manifest.variants[6..12]
        .iter()
        .all(|request| request.key.features.bits() == material_feature_bits));
    assert!(manifest.variants[6..12]
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_BLINN_PHONG));
    assert_eq!(
        manifest.variants[12].key.features.bits(),
        ShaderFeatureBits::DOUBLE_SIDED | ShaderFeatureBits::RECEIVE_SHADOWS
    );
    assert_eq!(
        manifest.variants[12].key.shading_model, SHADING_MODEL_ID_UNLIT,
        "transparent fixture uses the built-in Unlit shading model"
    );
    assert!(manifest.variants[..6]
        .iter()
        .all(|request| request.pipeline_state == Some(ShaderPipelinePrewarmState::default())));
    let masked_state = manifest.variants[6].pipeline_state.unwrap();
    assert!(!masked_state.alpha_blend);
    assert_eq!(masked_state.alpha_cutoff_bits, Some(0.5_f32.to_bits()));
    let transparent_state = manifest.variants[12].pipeline_state.unwrap();
    assert!(transparent_state.alpha_blend);
    assert!(transparent_state.unlit);
    assert_eq!(transparent_state.alpha_cutoff_bits, None);
    let mut same_shader_transparent = manifest.variants[0].clone();
    same_shader_transparent.pipeline_state = Some(ShaderPipelinePrewarmState {
        alpha_blend: true,
        ..Default::default()
    });
    let source = source_for(&manifest, &manifest.variants[0]).clone();
    let deduped = dedupe_prewarm_manifest(ShaderVariantPrewarmManifest::new(
        vec![source],
        vec![manifest.variants[0].clone(), same_shader_transparent],
    ));
    assert_eq!(
        deduped.variants.len(),
        2,
        "runtime PSO inventory must not merge equal shader keys with different pipeline state"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_uses_sparse_material_option_keys() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_sparse_options_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/optioned")).unwrap();
    fs::write(
        root.join("shaders/optioned.zmeta"),
        r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000051"
url = "res://shaders/optioned"
asset_kind = "Shader"
unit = "compound"
source_digest = "sparse-options-hash"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/optioned/optioned.zshader"),
        r#"version = 2
kind = "surface"
name = "Optioned"
wgsl_files = ["surface.wgsl"]
shading_model = "standard_pbr"

[[properties]]
name = "tint"
kind = "vec4"
default = [1.0, 1.0, 1.0, 1.0]

[[options]]
name = "clearcoat"
kind = "bool"
default = false

[[options]]
name = "blend_mode"
kind = "enum"
default = "solid"
editor = { values = "solid, foliage, glass" }
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/optioned/surface.wgsl"),
        "fn surface() {}\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("materials")).unwrap();
    for (name, blend_mode) in [
        ("foliage_a", "foliage"),
        ("foliage_b", "foliage"),
        ("glass", "glass"),
    ] {
        fs::write(
            root.join("materials").join(format!("{name}.zmaterial")),
            format!(
                r#"version = 2
name = "{name}"

[shader]
uuid = "00000000-0000-0000-0000-000000000051"
url = "res://shaders/optioned"

[options]
clearcoat = true
blend_mode = "{blend_mode}"
"#
            ),
        )
        .unwrap();
    }

    let manifest = asset_root_manifest(&root).unwrap();

    assert_eq!(
        manifest.variants.len(),
        18,
        "one source-default variant set plus two unique material option sets"
    );
    let material_requests = manifest
        .variants
        .iter()
        .filter(|request| {
            request
                .key
                .features
                .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
        })
        .collect::<Vec<_>>();
    assert_eq!(material_requests.len(), 12);
    let material_option_bits = material_requests
        .iter()
        .map(|request| request.key.material_option_bits)
        .collect::<BTreeSet<_>>();
    assert_eq!(material_option_bits, BTreeSet::from([3, 5]));
    assert!(material_requests
        .iter()
        .all(|request| request.key.material_layout_hash != 0));

    let source_requests = manifest
        .variants
        .iter()
        .filter(|request| {
            !request
                .key
                .features
                .contains(ShaderFeatureBits::RECEIVE_SHADOWS)
        })
        .collect::<Vec<_>>();
    assert_eq!(source_requests.len(), 6);
    assert!(source_requests
        .iter()
        .all(|request| request.key.material_option_bits == 0));
    assert!(source_requests
        .iter()
        .all(|request| request.key.material_layout_hash
            == material_requests[0].key.material_layout_hash));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_builtin_standard_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("materials")).unwrap();
    fs::write(
        root.join("materials/builtin.zmaterial"),
        r#"version = 2
name = "Builtin"

[shader]
uuid = "00000000-0000-0000-0000-000000000042"
url = "builtin://shader/pbr.wgsl"

[overrides]
double_sided = true
lighting_model = "blinn_phong"

[overrides.alpha_mode]
mode = "mask"
cutoff = 0.5
"#,
    )
    .unwrap();

    let manifest = asset_root_manifest_for_quality_tiers_and_geometry_sources(
        &root,
        &[ShaderQualityTier::High],
        &[
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ],
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 12);
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH)
            .count(),
        6
    );
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH)
            .count(),
        6
    );
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert_eq!(
            manifest
                .variants
                .iter()
                .filter(|request| request.key.pass_type == pass_type)
                .count(),
            2
        );
    }
    let static_forward_request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH
                && request.key.pass_type == ShaderPassType::Forward
        })
        .expect("static builtin standard material forward request");
    let skinned_forward_request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH
                && request.key.pass_type == ShaderPassType::Forward
        })
        .expect("skinned builtin standard material forward request");
    let skinned_depth_request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH
                && request.key.pass_type == ShaderPassType::DepthPrepass
        })
        .expect("skinned builtin standard material depth-only request");

    for request in &manifest.variants {
        let source = source_for(&manifest, request);
        assert_eq!(source.source_label, "builtin://shader/pbr.wgsl");
        assert_eq!(request.key.quality, ShaderQualityTier::High);
        assert_eq!(request.key.shading_model, SHADING_MODEL_ID_BLINN_PHONG);
        assert_eq!(
            request.key.features.bits(),
            ShaderFeatureBits::ALPHA_TEST
                | ShaderFeatureBits::DOUBLE_SIDED
                | ShaderFeatureBits::RECEIVE_SHADOWS
        );
        assert_eq!(source.template_revision, "zr-material-template-v1");
        assert!(source.include_content_hashes.len() > 1);
    }

    for request in [static_forward_request, skinned_forward_request] {
        let source = source_for(&manifest, request);
        assert!(source.wgsl_source.contains("fn zr_material_surface("));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_main("));
        assert!(source
            .wgsl_source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
    }

    let static_forward_source = source_for(&manifest, static_forward_request);
    let skinned_forward_source = source_for(&manifest, skinned_forward_request);
    let skinned_depth_source = source_for(&manifest, skinned_depth_request);
    assert!(static_forward_source
        .wgsl_source
        .contains("// include: zr_geometry_static.wgsl"));
    assert!(skinned_forward_source
        .wgsl_source
        .contains("// include: zr_geometry_skinned.wgsl"));
    assert!(skinned_forward_source
        .wgsl_source
        .contains("const ZR_GEOMETRY_SOURCE_SKINNED_MESH: bool = true;"));
    assert!(skinned_depth_source
        .wgsl_source
        .contains("// include: zr_template_depth_alpha.wgsl"));
    assert!(skinned_depth_source
        .wgsl_source
        .contains("// include: zr_geometry_skinned.wgsl"));
    assert!(skinned_depth_source
        .wgsl_source
        .contains("zr_apply_alpha_clip(surface);"));
    assert!(!skinned_depth_source
        .wgsl_source
        .contains("surface.normal_ws * 0.5"));
    assert!(!skinned_depth_source
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));
    assert_ne!(
        skinned_forward_source.wgsl_source,
        skinned_depth_source.wgsl_source
    );
    assert_ne!(
        skinned_forward_source.include_content_hashes,
        skinned_depth_source.include_content_hashes
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_maps_custom_shading_model_plugin_ids() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_custom_shading_model_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("materials")).unwrap();
    fs::write(
        root.join("materials/custom.zmaterial"),
        r#"version = 2
name = "Custom"

[shader]
uuid = "00000000-0000-0000-0000-000000000043"
url = "builtin://shader/pbr.wgsl"

[overrides]
lighting_model = "custom:subsurface"
"#,
    )
    .unwrap();

    let custom_id = ShadingModelId::new(SHADING_MODEL_PLUGIN_ID_START);
    let mut shading_model_ids = BTreeMap::new();
    shading_model_ids.insert("custom:subsurface".to_string(), custom_id);

    let manifest = asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &shading_model_ids,
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        request.key.shading_model == custom_id && request.key.shading_model.is_plugin_range()
    }));
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert!(manifest
            .variants
            .iter()
            .any(|request| request.key.pass_type == pass_type));
    }
    let forward_request = manifest
        .variants
        .iter()
        .find(|request| request.key.pass_type == ShaderPassType::Forward)
        .expect("custom shading model forward prewarm request");
    assert!(source_for(&manifest, forward_request)
        .wgsl_source
        .contains("fn zr_material_surface("));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_uses_zmeta_source_digest_revision() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_zmeta_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/example")).unwrap();
    fs::write(
        root.join("shaders/example.zmeta"),
        r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000044"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_digest = "source-hash-a"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/example/example.zshader"),
        r#"version = 2
kind = "surface"
wgsl_files = ["base.wgsl"]
shading_model = "standard_pbr"
"#,
    )
    .unwrap();
    fs::write(root.join("shaders/example/base.wgsl"), "fn base_a() {}\n").unwrap();

    let first_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;
    fs::write(
        root.join("shaders/example.zmeta"),
        r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000044"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_digest = "source-hash-b"
"#,
    )
    .unwrap();

    let second_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;

    assert_ne!(first_revision, 0);
    assert_ne!(second_revision, 0);
    assert_ne!(
        first_revision, second_revision,
        "zmeta source_digest edits must export a new shader prewarm material revision"
    );
    let _ = fs::remove_dir_all(root);
}
