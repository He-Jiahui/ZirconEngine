use std::{hint::black_box, time::Instant};

use super::*;

const SOURCE_EXTENSIONS: [&str; 10] = [
    "WGSL", "wgsl", "GLSL", "Vert", "FRAG", "comp", "VS", "fs", "Cs", "SPV",
];
const STAGE_NAMES: [&str; 9] = [
    "Vertex", "VERT", "vs", "Fragment", "FRAG", "fs", "Compute", "COMP", "cs",
];

#[test]
fn raw_wgsl_imports_as_generic_shader_module() {
    let context = AssetImportContext::new(
        "module.wgsl".into(),
        crate::asset::AssetUri::parse("res://shaders/module.wgsl").unwrap(),
        b"fn helper() {}".to_vec(),
        Default::default(),
    );

    let outcome = import_wgsl(&context).unwrap();
    let ImportedAsset::Shader(shader) = &outcome.root_entry().unwrap().asset else {
        panic!("expected WGSL shader asset");
    };

    assert_eq!(shader.kind, ShaderAssetKind::Module);
    assert!(shader.entry_points.is_empty());
    assert!(shader.readiness_report().is_ready());
}

fn legacy_shader_source_kind(extension: &str) -> Option<ShaderSourceKind> {
    match extension.to_ascii_lowercase().as_str() {
        "wgsl" => Some(ShaderSourceKind::Wgsl),
        "glsl" | "vert" | "frag" | "comp" | "vs" | "fs" | "cs" => Some(ShaderSourceKind::Glsl),
        "spv" => Some(ShaderSourceKind::SpirV),
        _ => None,
    }
}

fn legacy_shader_stage_hint(stage: &str) -> Option<naga::ShaderStage> {
    match stage.trim().to_ascii_lowercase().as_str() {
        "vertex" | "vert" | "vs" => Some(naga::ShaderStage::Vertex),
        "fragment" | "frag" | "fs" => Some(naga::ShaderStage::Fragment),
        "compute" | "comp" | "cs" => Some(naga::ShaderStage::Compute),
        _ => None,
    }
}

#[test]
fn plugins07_builtin_shader_hotpath_extension_dispatch_preserves_ascii_case_matching() {
    for extension in SOURCE_EXTENSIONS {
        assert_eq!(
            ShaderSourceKind::from_extension(extension),
            legacy_shader_source_kind(extension),
        );
    }
    assert_eq!(ShaderSourceKind::from_extension("metal"), None);
}

#[test]
fn plugins07_builtin_shader_hotpath_stage_parse_preserves_aliases_and_diagnostics() {
    for stage in STAGE_NAMES {
        assert_eq!(shader_stage_hint(stage), legacy_shader_stage_hint(stage));
    }
    assert_eq!(shader_stage_hint("geometry"), None);
    let error = parse_shader_stage(" Geometry ")
        .expect_err("unknown shader stage must fail closed")
        .to_string();
    assert!(
        error.contains("unsupported shader stage `geometry`"),
        "unexpected error: {error}"
    );
}

#[test]
#[ignore = "release-only borrowed shader extension dispatch benchmark"]
fn plugins07_builtin_shader_hotpath_release_borrowed_extension_dispatch_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 100_000;
    let (legacy_samples, optimized_samples) = alternating_samples(
        SAMPLE_PAIRS,
        || {
            measure(CHECKS_PER_SAMPLE, &SOURCE_EXTENSIONS, |extension| {
                legacy_shader_source_kind(extension)
            })
        },
        || {
            measure(CHECKS_PER_SAMPLE, &SOURCE_EXTENSIONS, |extension| {
                ShaderSourceKind::from_extension(extension)
            })
        },
    );
    report_and_assert(
        "plugins07_builtin_shader_extension_dispatch",
        SAMPLE_PAIRS,
        CHECKS_PER_SAMPLE,
        SOURCE_EXTENSIONS.len(),
        &legacy_samples,
        &optimized_samples,
    );
}

#[test]
#[ignore = "release-only borrowed shader stage parse benchmark"]
fn plugins07_builtin_shader_hotpath_release_borrowed_stage_parse_p95_gate() {
    const SAMPLE_PAIRS: usize = 21;
    const CHECKS_PER_SAMPLE: usize = 100_000;
    let (legacy_samples, optimized_samples) = alternating_samples(
        SAMPLE_PAIRS,
        || {
            measure(CHECKS_PER_SAMPLE, &STAGE_NAMES, |stage| {
                legacy_shader_stage_hint(stage)
            })
        },
        || measure(CHECKS_PER_SAMPLE, &STAGE_NAMES, shader_stage_hint),
    );
    report_and_assert(
        "plugins07_builtin_shader_stage_parse",
        SAMPLE_PAIRS,
        CHECKS_PER_SAMPLE,
        STAGE_NAMES.len(),
        &legacy_samples,
        &optimized_samples,
    );
}

fn measure<T: Copy>(
    checks_per_sample: usize,
    values: &[&str],
    mut classify: impl FnMut(&str) -> Option<T>,
) -> u128 {
    let started = Instant::now();
    for check in 0..checks_per_sample {
        let value = black_box(values[check % values.len()]);
        black_box(classify(value));
    }
    started.elapsed().as_nanos().max(1)
}

fn alternating_samples(
    sample_pairs: usize,
    mut legacy: impl FnMut() -> u128,
    mut optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(legacy());
        black_box(optimized());
    }
    let mut legacy_samples = Vec::with_capacity(sample_pairs);
    let mut optimized_samples = Vec::with_capacity(sample_pairs);
    for pair in 0..sample_pairs {
        if pair % 2 == 0 {
            legacy_samples.push(legacy());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            legacy_samples.push(legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn report_and_assert(
    name: &str,
    sample_pairs: usize,
    checks_per_sample: usize,
    variants: usize,
    legacy_samples: &[u128],
    optimized_samples: &[u128],
) {
    let legacy_p95_ns = percentile(legacy_samples, 95);
    let optimized_p95_ns = percentile(optimized_samples, 95);
    let improvement_percent = improvement_percent(legacy_p95_ns, optimized_p95_ns);
    println!(
        "PERF_RESULT {name} sample_pairs={sample_pairs} \
checks_per_sample={checks_per_sample} variants={variants} \
order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_owned_strings_per_sample={checks_per_sample} optimized_owned_strings_per_sample=0 \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
improvement_percent={improvement_percent} threshold_percent=50 \
legacy_ns={} optimized_ns={}",
        raw(legacy_samples),
        raw(optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "borrowed shader classification must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn improvement_percent(legacy: u128, optimized: u128) -> u128 {
    if optimized >= legacy {
        0
    } else {
        legacy.saturating_sub(optimized).saturating_mul(100) / legacy.max(1)
    }
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
