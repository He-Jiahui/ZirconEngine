use std::fs;

use zircon_runtime::core::framework::render::{
    ShaderFeatureBits, SHADING_MODEL_ID_BLINN_PHONG, SHADING_MODEL_ID_STANDARD_PBR,
    SHADING_MODEL_ID_UNLIT,
};

use super::asset_root_manifest;

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
    assert!(request.wgsl_source.contains("fn base() {}"));
    assert!(request.wgsl_source.contains("fn variant() {}"));
    assert_eq!(request.include_content_hashes.len(), 2);
    assert_eq!(request.key.platform_token, "wgpu-runtime");
    assert_eq!(
        request.key.material_revision,
        super::ASSET_SCAN_INITIAL_RESOURCE_REVISION
    );
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
    let material_feature_bits = ShaderFeatureBits::ALPHA_TEST | ShaderFeatureBits::DOUBLE_SIDED;
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
        ShaderFeatureBits::DOUBLE_SIDED
    );
    assert_eq!(
        manifest.variants[10].key.shading_model, SHADING_MODEL_ID_UNLIT,
        "transparent fixture uses the built-in Unlit shading model"
    );
    let _ = fs::remove_dir_all(root);
}
