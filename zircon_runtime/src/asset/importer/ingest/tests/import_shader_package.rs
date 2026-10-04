use std::collections::BTreeMap;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use super::{
    append_shader_module_diagnostics, document_import_path, import_shader_package,
    normalized_relative_path, AssetImportContext, ShaderImportPathDerivationError,
    ShaderImportRedirectAsset, ZShaderDocumentV2,
};
use crate::asset::AssetUri;

#[test]
fn shader_auxiliary_path_containment_preserves_internal_parent_and_rejects_escape() {
    let root = auxiliary_test_root("shader-source-containment");
    let asset_root = root.join("assets");
    let package_dir = asset_root.join("shaders").join("package");
    let shared_dir = asset_root.join("shared");
    fs::create_dir_all(&package_dir).unwrap();
    fs::create_dir_all(&shared_dir).unwrap();
    fs::write(
        shared_dir.join("surface.wgsl"),
        "fn shared_surface() -> vec4f { return vec4f(1.0); }",
    )
    .unwrap();
    let descriptor_path = package_dir.join("package.zshader");
    let source_path = asset_root.join("shaders").join("package.zmeta");
    let valid_descriptor = "kind = \"include\"\nversion = 2\nimport_path = \"project::shared\"\nwgsl_files = [\"../../shared/surface.wgsl\"]\n";
    fs::write(&descriptor_path, "invalid live descriptor").unwrap();
    let context = AssetImportContext::new(
        source_path.clone(),
        AssetUri::parse("res://shaders/package").unwrap(),
        Vec::new(),
        toml::Table::new(),
    )
    .with_source_file_snapshots(BTreeMap::from([
        (
            descriptor_path.clone(),
            valid_descriptor.as_bytes().to_vec(),
        ),
        (
            shared_dir.join("surface.wgsl"),
            b"fn shared_surface() -> vec4f { return vec4f(1.0); }".to_vec(),
        ),
    ]));
    import_shader_package(&context).expect("an internal parent reference must remain valid");

    fs::remove_file(&descriptor_path).unwrap();
    fs::write(
        package_dir.join("00-live.zshader"),
        "invalid live descriptor",
    )
    .unwrap();
    fs::write(package_dir.join("late.wgsl"), "invalid live shader").unwrap();
    import_shader_package(&context)
        .expect("captured descriptor membership survives deletion and live additions");
    fs::remove_file(package_dir.join("00-live.zshader")).unwrap();
    fs::remove_file(package_dir.join("late.wgsl")).unwrap();

    fs::write(
            &descriptor_path,
            "kind = \"include\"\nversion = 2\nimport_path = \"project::shared\"\nwgsl_files = [\"../../../outside.wgsl\"]\n",
        )
        .unwrap();
    let unsnapshotted_context = AssetImportContext::new(
        source_path,
        AssetUri::parse("res://shaders/package").unwrap(),
        Vec::new(),
        toml::Table::new(),
    );
    let error = import_shader_package(&unsnapshotted_context)
        .expect_err("an out-of-root WGSL source path must be rejected");
    assert!(error.to_string().contains("escapes asset root"), "{error}");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn zshader_import_diagnostics_report_undeclared_wgsl_include() {
    let document = ZShaderDocumentV2::from_toml_str(
        r#"
kind = "surface"
version = 2
shading_model = "standard_pbr"
wgsl_files = ["surface.wgsl"]
"#,
    )
    .expect("surface zshader should parse");
    let mut diagnostics = Vec::new();

    append_shader_module_diagnostics(
        &mut diagnostics,
        &document,
        "#include <project::math>\nfn zr_material_surface() {}",
        &[],
    );

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("project::math")));
}

#[test]
fn zshader_package_entry_point_discovery_strips_include_directives() {
    let mut diagnostics = Vec::new();
    let entry_points = super::package_shader_entry_points(
        &AssetUri::parse("res://shaders/include_surface").unwrap(),
        r#"
#include <project::math>
#include <self::material>

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0);
}
"#,
        &mut diagnostics,
    );

    assert_eq!(entry_points.len(), 1);
    assert_eq!(entry_points[0].name, "fs_main");
    assert_eq!(entry_points[0].stage, "fragment");
    assert!(
        diagnostics.is_empty(),
        "include directives should not block package entry point discovery: {diagnostics:?}"
    );
}

#[test]
fn zshader_include_module_diagnostics_reject_entry_points_and_bindings() {
    let document = ZShaderDocumentV2::from_toml_str(
        r#"
kind = "include"
version = 2
import_path = "project::bad"
wgsl_files = ["bad.wgsl"]
"#,
    )
    .expect("include zshader should parse");
    let mut diagnostics = Vec::new();

    append_shader_module_diagnostics(
        &mut diagnostics,
        &document,
        "@group(2) @binding(0) var<uniform> bad: vec4<f32>;\n@fragment\nfn fs_main() {}",
        &[] as &[ShaderImportRedirectAsset],
    );

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("@group binding")));
    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("entry point annotation")));
}

#[test]
fn zshader_import_path_validation_rejects_self_namespace_overrides() {
    let document = ZShaderDocumentV2::from_toml_str(
        r#"
kind = "include"
version = 2
import_path = "self::material"
"#,
    )
    .expect("include zshader should parse before import path validation");
    let mut diagnostics = Vec::new();

    let error = document_import_path(&document, None, &mut diagnostics)
        .expect_err("self namespace must stay generated-local");

    assert_eq!(
        error,
        ShaderImportPathDerivationError::ReservedNamespace {
            namespace: "self".to_string()
        }
    );
}

const SAMPLE_PAIRS: usize = 21;
const PATHS_PER_SAMPLE: usize = 8_192;

#[test]
fn normalized_relative_path_preserves_component_order_and_forward_slashes() {
    let path = PathBuf::from("shaders")
        .join("lighting")
        .join("surface.wgsl");

    assert_eq!(
        normalized_relative_path(&path),
        "shaders/lighting/surface.wgsl"
    );
}

#[test]
#[ignore = "release-only performance contract"]
fn benchmark_single_allocation_shader_relative_path_normalization() {
    let path = benchmark_path();
    let mut legacy_raw = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_raw = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_raw.push(measure_paths(&path, legacy_normalized_relative_path));
            optimized_raw.push(measure_paths(&path, normalized_relative_path));
        } else {
            optimized_raw.push(measure_paths(&path, normalized_relative_path));
            legacy_raw.push(measure_paths(&path, legacy_normalized_relative_path));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_raw, 95);
    let optimized_p95_ns = nearest_rank(&optimized_raw, 95);
    let improvement_percent = legacy_p95_ns
        .saturating_sub(optimized_p95_ns)
        .saturating_mul(100)
        / legacy_p95_ns.max(1);
    assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(85),
            "single-allocation relative path normalization must improve P95 by at least 15%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    println!(
            "PERF_RESULT task=plugins07_single_allocation_shader_relative_path sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 percentile_method=nearest_rank path_components=64 paths_per_sample={PATHS_PER_SAMPLE} legacy_allocations_per_path=2 optimized_allocations_per_path=1 legacy_component_vec_allocations_per_sample={PATHS_PER_SAMPLE} optimized_component_vec_allocations_per_sample=0 threshold_percent=15 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} improvement_percent={improvement_percent} legacy_raw_ns={} optimized_raw_ns={}",
            raw_samples(&legacy_raw),
            raw_samples(&optimized_raw)
        );
}

fn legacy_normalized_relative_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn benchmark_path() -> PathBuf {
    let mut path = PathBuf::new();
    for index in 0..64 {
        path.push(format!("shader_component_{index:02}"));
    }
    path
}

fn measure_paths(path: &Path, normalize: fn(&Path) -> String) -> u64 {
    let started = Instant::now();
    for _ in 0..PATHS_PER_SAMPLE {
        black_box(normalize(black_box(path)));
    }
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn nearest_rank(samples: &[u64], percentile: usize) -> u64 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw_samples(samples: &[u64]) -> String {
    let values = samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{values}]")
}

fn auxiliary_test_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}
