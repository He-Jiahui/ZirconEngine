use std::hint::black_box;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use zircon_runtime::asset::{AssetImportContext, AssetUri};

use crate::manifest_source::resolve_manifest_source;

#[test]
fn manifest_source_hotpath_single_resolution_matches_legacy_path_and_uri() {
    let context = manifest_context();
    let source = "characters/hero/materials/albedo.png";

    let optimized = resolve_manifest_source(&context, source).expect("valid manifest source");
    let legacy = legacy_resolve_manifest_source(&context, source);

    assert_eq!(optimized.path, legacy.0);
    assert_eq!(optimized.uri, legacy.1);
    assert_eq!(
        optimized.uri.to_string(),
        "res://textures/characters/hero/materials/albedo.png"
    );
}

#[test]
fn manifest_source_hotpath_single_resolution_preserves_admission_rejections() {
    let context = manifest_context();

    for source in ["../outside.png", "res://textures/albedo.png", "   "] {
        let error = resolve_manifest_source(&context, source)
            .expect_err("unsafe manifest source must fail closed")
            .to_string();
        assert!(
            error.contains("project-relative path without parent traversal"),
            "unexpected rejection for {source:?}: {error}"
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn manifest_source_hotpath_release_gate_walks_components_once_and_uses_typed_uri() {
    const SAMPLE_PAIRS: usize = 21;
    const RESOLUTIONS_PER_SAMPLE: usize = 4_096;
    const PATH_COMPONENTS: usize = 129;
    const REQUIRED_IMPROVEMENT_PERCENT: u128 = 20;

    let context = manifest_context();
    let source = (0..PATH_COMPONENTS - 1)
        .map(|index| format!("component_{index:03}"))
        .chain(std::iter::once("albedo.png".to_string()))
        .collect::<Vec<_>>()
        .join("/");

    for _ in 0..2 {
        black_box(measure_manifest_resolution(
            &context,
            &source,
            RESOLUTIONS_PER_SAMPLE,
            legacy_resolve_manifest_source,
        ));
        black_box(measure_manifest_resolution(
            &context,
            &source,
            RESOLUTIONS_PER_SAMPLE,
            optimized_resolve_manifest_source,
        ));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_manifest_resolution(
                &context,
                &source,
                RESOLUTIONS_PER_SAMPLE,
                legacy_resolve_manifest_source,
            ));
            optimized_samples.push(measure_manifest_resolution(
                &context,
                &source,
                RESOLUTIONS_PER_SAMPLE,
                optimized_resolve_manifest_source,
            ));
        } else {
            optimized_samples.push(measure_manifest_resolution(
                &context,
                &source,
                RESOLUTIONS_PER_SAMPLE,
                optimized_resolve_manifest_source,
            ));
            legacy_samples.push(measure_manifest_resolution(
                &context,
                &source,
                RESOLUTIONS_PER_SAMPLE,
                legacy_resolve_manifest_source,
            ));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples).as_nanos();
    let optimized_p95 = nearest_rank_p95(&optimized_samples).as_nanos();
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_manifest_source_resolution sample_pairs={SAMPLE_PAIRS} order=alternating_legacy_first_even resolutions_per_sample={RESOLUTIONS_PER_SAMPLE} path_components={PATH_COMPONENTS} legacy_component_walks_per_resolution=2 optimized_component_walks_per_resolution=1 legacy_uri_string_roundtrips_per_resolution=1 optimized_uri_string_roundtrips_per_resolution=0 legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent={REQUIRED_IMPROVEMENT_PERCENT}",
        durations_csv(&legacy_samples),
        durations_csv(&optimized_samples),
    );
    assert!(
        improvement_percent >= REQUIRED_IMPROVEMENT_PERCENT,
        "single-pass manifest source resolution must improve P95 by at least {REQUIRED_IMPROVEMENT_PERCENT}% (legacy={legacy_p95}ns optimized={optimized_p95}ns improvement={improvement_percent}%)"
    );
}

fn optimized_resolve_manifest_source(
    context: &AssetImportContext,
    source: &str,
) -> (PathBuf, AssetUri) {
    let resolved = resolve_manifest_source(context, source).expect("valid manifest source");
    (resolved.path, resolved.uri)
}

fn legacy_resolve_manifest_source(
    context: &AssetImportContext,
    source: &str,
) -> (PathBuf, AssetUri) {
    let relative_path = legacy_validated_relative_source(source);
    let filesystem_path = context
        .source_path
        .parent()
        .expect("manifest source directory")
        .join(relative_path);

    let relative_uri = legacy_validated_relative_source(source);
    let manifest_uri = context.uri.to_string();
    let (scheme, _) = manifest_uri.split_once("://").expect("manifest URI scheme");
    let parent = Path::new(context.uri.path())
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let path = parent
        .join(relative_uri)
        .to_string_lossy()
        .replace('\\', "/");
    let uri = AssetUri::parse(&format!("{scheme}://{path}")).expect("valid source URI");
    (filesystem_path, uri)
}

fn legacy_validated_relative_source(source: &str) -> &Path {
    let path = Path::new(source);
    assert!(
        !source.trim().is_empty()
            && !source.contains("://")
            && path
                .components()
                .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    );
    path
}

fn measure_manifest_resolution(
    context: &AssetImportContext,
    source: &str,
    resolutions_per_sample: usize,
    resolver: fn(&AssetImportContext, &str) -> (PathBuf, AssetUri),
) -> Duration {
    let started = Instant::now();
    for _ in 0..resolutions_per_sample {
        black_box(resolver(black_box(context), black_box(source)));
    }
    started.elapsed()
}

fn nearest_rank_p95(samples: &[Duration]) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * 95).div_ceil(100).saturating_sub(1)]
}

fn durations_csv(samples: &[Duration]) -> String {
    samples
        .iter()
        .map(|sample| sample.as_nanos().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn manifest_context() -> AssetImportContext {
    AssetImportContext::new(
        PathBuf::from(r"C:\project\assets\textures\environment.zcube"),
        AssetUri::parse("res://textures/environment.zcube").expect("valid manifest URI"),
        Vec::new(),
        Default::default(),
    )
}
