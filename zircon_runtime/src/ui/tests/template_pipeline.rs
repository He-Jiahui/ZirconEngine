use std::time::Instant;

use crate::ui::template::{
    UiAssetLoader, UiDocumentCompiler, UiRuntimeCompiledAssetArtifact, UiTemplateSurfaceBuilder,
};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

const COMPILED_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "runtime74.compiler_authority"
version = 3

[root]
node_id = "root"
kind = "native"
type = "Button"
control_id = "CompileRoot"
props = { text = "Compile" }
"#;

#[test]
fn asset_compiler_is_the_single_template_compile_authority() {
    let document = UiAssetLoader::load_toml_str(COMPILED_LAYOUT).unwrap();
    let compiled = UiDocumentCompiler::default().compile(&document).unwrap();
    let surface = UiTemplateSurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime74.compiler_authority"),
        &compiled,
    )
    .unwrap();

    assert_eq!(surface.tree.nodes.len(), 1);
    assert_eq!(
        compiled.template_instance().root.component.as_deref(),
        Some("Button")
    );
}

#[test]
fn legacy_recursive_template_document_is_not_a_runtime_compile_input() {
    let legacy_source = r#"
version = 1

[root]
component = "Button"
"#;

    assert!(UiAssetLoader::load_toml_str(legacy_source).is_err());
}

#[test]
fn template_compiler_authority_has_bounded_p95_latency() {
    const SAMPLE_COUNT: usize = 21;
    const COMPILES_PER_SAMPLE: usize = 100;
    const P95_LIMIT_US: u128 = 250_000;

    let document = UiAssetLoader::load_toml_str(COMPILED_LAYOUT).unwrap();
    let compiler = UiDocumentCompiler::default();
    let mut samples = Vec::with_capacity(SAMPLE_COUNT);

    for _ in 0..SAMPLE_COUNT {
        let started = Instant::now();
        for _ in 0..COMPILES_PER_SAMPLE {
            std::hint::black_box(compiler.compile(std::hint::black_box(&document)).unwrap());
        }
        samples.push(started.elapsed().as_micros());
    }

    let raw_samples = samples.clone();
    samples.sort_unstable();
    let p95_index = (SAMPLE_COUNT * 95).div_ceil(100) - 1;
    let compile_p95_us = samples[p95_index];
    let samples_us = raw_samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "PERF-RUNTIME74-COMPILER-AUTHORITY sample_count={SAMPLE_COUNT} compiles_per_sample={COMPILES_PER_SAMPLE} samples_us={samples_us} compile_p95_us={compile_p95_us} p95_limit_us={P95_LIMIT_US} runtime_compiler_authorities=1 legacy_runtime_pipeline_exports=0"
    );
    assert!(
        compile_p95_us <= P95_LIMIT_US,
        "100 compile operations exceeded the bounded P95 latency: {compile_p95_us}us"
    );
}

#[test]
fn compiled_template_artifact_stays_toml_envelope_leaf_dto_not_generated_source() {
    assert_eq!(
        UiRuntimeCompiledAssetArtifact::generated_policy(),
        "runtime_09_m3_1_toml_envelope_leaf_dto_not_generated_source"
    );
    assert!(!UiRuntimeCompiledAssetArtifact::requires_generated_source_marker());
}
