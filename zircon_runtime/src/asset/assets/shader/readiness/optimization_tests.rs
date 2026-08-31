use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::ShaderAssetKind;

use super::*;

#[test]
fn optimization_batch_20260826l_runtime91_borrowed_define_index_preserves_diagnostics() {
    let shader = shader_with_definitions(vec![
        RenderShaderDefinitionValue::from("FEATURE_A"),
        RenderShaderDefinitionValue::from("  "),
        RenderShaderDefinitionValue::uint("FEATURE_B", 7),
        RenderShaderDefinitionValue::bool(" FEATURE_A ", false),
    ]);

    let readiness = shader_definition_readiness(&shader);

    assert_eq!(readiness[0].normalized_name, "FEATURE_A");
    assert!(readiness[0].diagnostic.is_none());
    assert_eq!(readiness[1].normalized_name, "");
    assert!(readiness[1]
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("empty after trimming")));
    assert_eq!(readiness[2].normalized_name, "FEATURE_B");
    assert!(readiness[2].diagnostic.is_none());
    assert_eq!(readiness[3].normalized_name, "FEATURE_A");
    assert!(readiness[3]
        .diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("duplicated")));
}

#[test]
fn optimization_batch_20260826l_runtime91_define_index_borrows_normalized_names() {
    let source = include_str!("../readiness.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("shader readiness production source");

    assert!(!production.contains("BTreeSet"));
    assert!(production.contains("HashSet::with_capacity(shader.shader_defs.len())"));
    assert!(production.contains("let normalized_name = definition.name().trim()"));
    assert!(production.contains("seen.insert(normalized_name)"));
    assert!(!production.contains("seen.insert(normalized_name.clone())"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260826l_runtime91_shader_define_borrowed_hash_performance_evidence() {
    let definitions = (0..32_768)
        .map(|index| format!("  FEATURE_RUNTIME_MATERIAL_{index:05}_LONG_NAME  "))
        .collect::<Vec<_>>();
    let copied_normalized_bytes = definitions
        .iter()
        .map(|definition| definition.trim().len())
        .sum::<usize>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut hash_samples = Vec::with_capacity(17);
    for _ in 0..17 {
        let started = Instant::now();
        let mut seen = BTreeSet::new();
        let mut output = Vec::with_capacity(definitions.len());
        for definition in black_box(&definitions) {
            let normalized_name = definition.trim().to_string();
            black_box(seen.insert(normalized_name.clone()));
            output.push(normalized_name);
        }
        black_box((seen, output));
        legacy_samples.push(started.elapsed().as_nanos());

        let started = Instant::now();
        let mut seen = HashSet::with_capacity(definitions.len());
        let mut output = Vec::with_capacity(definitions.len());
        for definition in black_box(&definitions) {
            let normalized_name = definition.trim();
            black_box(seen.insert(normalized_name));
            output.push(normalized_name.to_string());
        }
        black_box((seen, output));
        hash_samples.push(started.elapsed().as_nanos());
    }

    legacy_samples.sort_unstable();
    hash_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let hash_p95 = hash_samples[16];
    println!(
        "RUNTIME91_SHADER_DEFINE_BORROWED_HASH_INDEX_BENCH_V1 definitions={} legacy_p95_ns={} hash_p95_ns={} legacy_normalized_string_allocations={} hash_normalized_string_allocations={} legacy_copied_bytes={} hash_copied_bytes={} target_ratio_bp=6000",
        definitions.len(),
        legacy_p95,
        hash_p95,
        definitions.len() * 2,
        definitions.len(),
        copied_normalized_bytes * 2,
        copied_normalized_bytes,
    );
    assert!(
        hash_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(6_000),
        "borrowed shader define hash P95 {hash_p95} ns exceeded 60% of legacy {legacy_p95} ns"
    );
}

fn shader_with_definitions(shader_defs: Vec<RenderShaderDefinitionValue>) -> ShaderAsset {
    ShaderAsset {
        uri: AssetUri::parse("res://shaders/optimization_batch_20260826l.zshader")
            .expect("optimization shader URI"),
        kind: ShaderAssetKind::Surface,
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs,
        property_schema: Vec::new(),
        options: Vec::new(),
        texture_slots: Vec::new(),
        shading_model: None,
        render_state: Default::default(),
        queue: None,
        disabled_passes: Vec::new(),
        resources: Vec::new(),
        material_property_layout: Default::default(),
        material_option_table: Default::default(),
        generated_material_wgsl: String::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}
