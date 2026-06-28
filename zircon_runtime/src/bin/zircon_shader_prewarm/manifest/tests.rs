use std::collections::BTreeMap;
use std::fs;

use zircon_runtime::core::framework::render::{
    GeometrySourceId, ShaderFeatureBits, ShaderPassType, ShaderQualityTier, ShadingModelId,
    GEOMETRY_SOURCE_ID_SKINNED_MESH, GEOMETRY_SOURCE_ID_STATIC_MESH,
    GEOMETRY_SOURCE_PLUGIN_ID_START, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
    SHADING_MODEL_ID_UNLIT, SHADING_MODEL_PLUGIN_ID_START,
};
use zircon_runtime::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord, ResourceState,
};

use super::{
    asset_root_manifest, asset_root_manifest_for_quality_tiers_and_geometry_sources,
    asset_root_manifest_for_quality_tiers_geometry_sources_and_shading_model_ids,
    asset_root_manifest_with_resource_registry_revisions,
    builtin_fallback_manifest_for_quality_tiers_and_geometry_sources,
    resource_registry::{
        shader_resource_records_from_asset_root, shader_resource_records_from_manager,
        ShaderPrewarmResourceRegistryOverlay,
    },
};

const BUILTIN_MATERIAL_PASS_TYPES: [ShaderPassType; 6] = [
    ShaderPassType::Forward,
    ShaderPassType::GBuffer,
    ShaderPassType::DepthPrepass,
    ShaderPassType::Shadow,
    ShaderPassType::Velocity,
    ShaderPassType::TaaReactiveMask,
];

#[test]
fn shader_prewarm_builtin_fallback_manifest_expands_requested_geometry_sources() {
    let manifest = builtin_fallback_manifest_for_quality_tiers_and_geometry_sources(
        &[ShaderQualityTier::Medium, ShaderQualityTier::High],
        &[
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            GEOMETRY_SOURCE_ID_SKINNED_MESH,
        ],
    );

    assert_eq!(manifest.variants.len(), 24);
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_STATIC_MESH)
            .count(),
        12
    );
    assert_eq!(
        manifest
            .variants
            .iter()
            .filter(|request| request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH)
            .count(),
        12
    );
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert_eq!(
            manifest
                .variants
                .iter()
                .filter(|request| request.key.pass_type == pass_type)
                .count(),
            4
        );
    }
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_STANDARD_PBR));
    assert!(manifest.variants.iter().all(|request| request
        .key
        .features
        .contains(ShaderFeatureBits::RECEIVE_SHADOWS)));
    assert!(manifest
        .variants
        .iter()
        .any(|request| request.key.quality == ShaderQualityTier::High
            && request
                .wgsl_source
                .contains("// include: zr_geometry_skinned.wgsl")));
    let skinned_depth_request = manifest
        .variants
        .iter()
        .find(|request| {
            request.key.geometry_source == GEOMETRY_SOURCE_ID_SKINNED_MESH
                && request.key.quality == ShaderQualityTier::High
                && request.key.pass_type == ShaderPassType::DepthPrepass
        })
        .expect("high-quality skinned depth-only builtin fallback request");
    assert!(skinned_depth_request
        .wgsl_source
        .contains("// include: zr_template_depth.wgsl"));
    assert!(!skinned_depth_request
        .wgsl_source
        .contains("zr_material_surface"));
    assert!(!skinned_depth_request
        .wgsl_source
        .contains("surface.normal_ws * 0.5"));
    assert!(!skinned_depth_request
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));
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
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000041"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_hash = "scan-test-hash"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/example/example.zshader"),
        r#"version = 1
wgsl_files = ["base.wgsl", "variant.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"

[[entry_points]]
name = "fs_main"
stage = "fragment"
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
        r#"version = 1
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
        r#"version = 1
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

    assert_eq!(manifest.variants.len(), 11);
    let request = &manifest.variants[0];
    assert_eq!(request.source_label, "res://shaders/example");
    assert!(request.wgsl_source.contains("fn base() {}"));
    assert!(request.wgsl_source.contains("fn variant() {}"));
    assert_eq!(request.include_content_hashes.len(), 2);
    assert_eq!(request.key.platform_token, "wgpu-runtime");
    assert_ne!(request.key.material_revision, 0);
    let passes = manifest
        .variants
        .iter()
        .map(|request| request.key.pass_type.token())
        .collect::<Vec<_>>();
    assert_eq!(
        &passes[..5],
        vec!["forward", "gbuffer", "depth_prepass", "shadow", "velocity"]
    );
    assert_eq!(
        &passes[5..10],
        vec!["forward", "gbuffer", "depth_prepass", "shadow", "velocity"]
    );
    assert_eq!(passes[10], "forward");
    let material_feature_bits = ShaderFeatureBits::ALPHA_TEST
        | ShaderFeatureBits::DOUBLE_SIDED
        | ShaderFeatureBits::RECEIVE_SHADOWS;
    assert!(manifest.variants[..5]
        .iter()
        .all(|request| request.key.features.bits() == 0));
    assert!(manifest.variants[..5]
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_STANDARD_PBR));
    assert!(manifest.variants[5..10]
        .iter()
        .all(|request| request.key.features.bits() == material_feature_bits));
    assert!(manifest.variants[5..10]
        .iter()
        .all(|request| request.key.shading_model == SHADING_MODEL_ID_BLINN_PHONG));
    assert_eq!(
        manifest.variants[10].key.features.bits(),
        ShaderFeatureBits::DOUBLE_SIDED | ShaderFeatureBits::RECEIVE_SHADOWS
    );
    assert_eq!(
        manifest.variants[10].key.shading_model, SHADING_MODEL_ID_UNLIT,
        "transparent fixture uses the built-in Unlit shading model"
    );
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
        r#"version = 1
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
        assert_eq!(request.source_label, "builtin://shader/pbr.wgsl");
        assert_eq!(request.key.quality, ShaderQualityTier::High);
        assert_eq!(request.key.shading_model, SHADING_MODEL_ID_BLINN_PHONG);
        assert_eq!(
            request.key.features.bits(),
            ShaderFeatureBits::ALPHA_TEST
                | ShaderFeatureBits::DOUBLE_SIDED
                | ShaderFeatureBits::RECEIVE_SHADOWS
        );
        assert_eq!(request.template_revision, "zr-material-template-v1");
        assert!(request.include_content_hashes.len() > 1);
    }

    for request in [static_forward_request, skinned_forward_request] {
        assert!(request.wgsl_source.contains("fn zr_material_surface("));
        assert!(request.wgsl_source.contains("fn vs_main("));
        assert!(request.wgsl_source.contains("fn fs_main("));
        assert!(request
            .wgsl_source
            .contains("const ZR_STANDARD_MATERIAL_ALPHA_CUTOFF: f32 = 0.50000000;"));
    }

    assert!(static_forward_request
        .wgsl_source
        .contains("// include: zr_geometry_static.wgsl"));
    assert!(skinned_forward_request
        .wgsl_source
        .contains("// include: zr_geometry_skinned.wgsl"));
    assert!(skinned_forward_request
        .wgsl_source
        .contains("const ZR_GEOMETRY_SOURCE_SKINNED_MESH: bool = true;"));
    assert!(skinned_depth_request
        .wgsl_source
        .contains("// include: zr_template_depth_alpha.wgsl"));
    assert!(skinned_depth_request
        .wgsl_source
        .contains("// include: zr_geometry_skinned.wgsl"));
    assert!(skinned_depth_request
        .wgsl_source
        .contains("zr_apply_alpha_clip(surface);"));
    assert!(!skinned_depth_request
        .wgsl_source
        .contains("surface.normal_ws * 0.5"));
    assert!(!skinned_depth_request
        .wgsl_source
        .contains("// include: zr_template_gbuffer.wgsl"));
    assert_ne!(
        skinned_forward_request.wgsl_source,
        skinned_depth_request.wgsl_source
    );
    assert_ne!(
        skinned_forward_request.include_content_hashes,
        skinned_depth_request.include_content_hashes
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
        r#"version = 1
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
    assert!(forward_request
        .wgsl_source
        .contains("fn zr_material_surface("));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_expands_custom_geometry_source_plugin_ids() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_custom_geometry_source_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("procedural.wgsl"), "fn procedural() {}\n").unwrap();

    let custom_geometry_source = GeometrySourceId::new(GEOMETRY_SOURCE_PLUGIN_ID_START);
    let manifest = asset_root_manifest_for_quality_tiers_and_geometry_sources(
        &root,
        &[ShaderQualityTier::Medium],
        &[custom_geometry_source],
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest.variants.iter().all(|request| {
        request.key.geometry_source == custom_geometry_source
            && request.key.geometry_source.is_plugin_range()
    }));
    for pass_type in BUILTIN_MATERIAL_PASS_TYPES {
        assert!(manifest
            .variants
            .iter()
            .any(|request| request.key.pass_type == pass_type));
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_uses_zmeta_source_hash_revision() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_zmeta_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders/example")).unwrap();
    fs::write(
        root.join("shaders/example.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000044"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_hash = "source-hash-a"
"#,
    )
    .unwrap();
    fs::write(
        root.join("shaders/example/example.zshader"),
        r#"version = 1
wgsl_files = ["base.wgsl"]

[[entry_points]]
name = "vs_main"
stage = "vertex"
"#,
    )
    .unwrap();
    fs::write(root.join("shaders/example/base.wgsl"), "fn base_a() {}\n").unwrap();

    let first_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;
    fs::write(
        root.join("shaders/example.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000044"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "compound"
source_hash = "source-hash-b"
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
        "zmeta source_hash edits must export a new shader prewarm material revision"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_uses_resource_registry_revision_overlay() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_registry_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/example.wgsl"), "fn example() {}\n").unwrap();
    fs::write(
        root.join("shaders/example.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000045"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-registry-fallback"
"#,
    )
    .unwrap();

    let mut record = ResourceRecord::new(
        ResourceId::from_stable_label("registry-shader"),
        ResourceKind::Shader,
        ResourceLocator::parse("res://shaders/example").unwrap(),
    );
    record.revision = 77;
    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records([record]);

    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == 77));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_exports_shader_resource_records() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_registry_export_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/example.wgsl"), "fn example() {}\n").unwrap();
    fs::write(
        root.join("shaders/example.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000046"
url = "res://shaders/example"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-registry-export"
"#,
    )
    .unwrap();

    let records = shader_resource_records_from_asset_root(&root).unwrap();

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.kind, ResourceKind::Shader);
    assert_eq!(record.state, ResourceState::Ready);
    assert_eq!(
        record.primary_locator,
        ResourceLocator::parse("res://shaders/example").unwrap()
    );
    assert_ne!(record.revision, 0);

    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(records.clone());
    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == record.revision));
    let _ = fs::remove_dir_all(root);
}

#[derive(Debug)]
struct ShaderPayload;

#[test]
fn shader_prewarm_resource_registry_overlay_uses_live_resource_manager_shader_revisions() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_live_registry_export_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("shaders")).unwrap();
    fs::write(root.join("shaders/live.wgsl"), "fn live() {}\n").unwrap();
    fs::write(
        root.join("shaders/live.wgsl.zmeta"),
        r#"format_version = 6
uuid = "00000000-0000-0000-0000-000000000047"
url = "res://shaders/live"
asset_kind = "Shader"
unit = "single"
source_hash = "source-hash-live-manager-fallback"
"#,
    )
    .unwrap();

    let manager = ResourceManager::new();
    let live_locator = ResourceLocator::parse("res://shaders/live").unwrap();
    let live_id = ResourceId::from_locator(&live_locator);
    manager.register_ready(
        ResourceRecord::new(live_id, ResourceKind::Shader, live_locator.clone())
            .with_source_hash("live-manager-shader-a"),
        ShaderPayload,
    );
    manager.register_ready(
        ResourceRecord::new(live_id, ResourceKind::Shader, live_locator)
            .with_source_hash("live-manager-shader-b"),
        ShaderPayload,
    );
    let model_locator = ResourceLocator::parse("res://models/mesh.glb").unwrap();
    manager.register_ready(
        ResourceRecord::new(
            ResourceId::from_locator(&model_locator),
            ResourceKind::Model,
            model_locator,
        )
        .with_source_hash("live-manager-model"),
        ShaderPayload,
    );
    let pending_locator = ResourceLocator::parse("res://shaders/pending").unwrap();
    manager.register_record(ResourceRecord::new(
        ResourceId::from_locator(&pending_locator),
        ResourceKind::Shader,
        pending_locator,
    ));

    let records = shader_resource_records_from_manager(&manager);

    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id, live_id);
    assert_eq!(records[0].kind, ResourceKind::Shader);
    assert_eq!(records[0].state, ResourceState::Ready);
    assert_eq!(records[0].revision, 2);
    let live_revision = records[0].revision;

    let overlay = ShaderPrewarmResourceRegistryOverlay::from_records(records);
    let manifest = asset_root_manifest_with_resource_registry_revisions(
        &root,
        &[ShaderQualityTier::Medium],
        &[GEOMETRY_SOURCE_ID_STATIC_MESH],
        &BTreeMap::new(),
        Some(&overlay),
    )
    .unwrap();

    assert_eq!(manifest.variants.len(), 6);
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.material_revision == live_revision));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_uses_raw_source_hash_revision() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_raw_revision_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let shader_path = root.join("simple.wgsl");
    fs::write(&shader_path, "fn simple_a() {}\n").unwrap();

    let first_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;
    fs::write(&shader_path, "fn simple_b() {}\n").unwrap();
    let second_revision = asset_root_manifest(&root).unwrap().variants[0]
        .key
        .material_revision;

    assert_ne!(first_revision, 0);
    assert_ne!(second_revision, 0);
    assert_ne!(
        first_revision, second_revision,
        "raw shader source edits must export a new shader prewarm material revision"
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn shader_prewarm_asset_root_manifest_expands_requested_geometry_sources() {
    let root = std::env::temp_dir().join(format!(
        "zircon_shader_prewarm_geometry_scan_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("simple.wgsl"), "fn simple() {}\n").unwrap();

    let manifest = asset_root_manifest_for_quality_tiers_and_geometry_sources(
        &root,
        &[ShaderQualityTier::Medium],
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
    assert!(manifest
        .variants
        .iter()
        .all(|request| request.key.quality == ShaderQualityTier::Medium));

    let _ = fs::remove_dir_all(root);
}
