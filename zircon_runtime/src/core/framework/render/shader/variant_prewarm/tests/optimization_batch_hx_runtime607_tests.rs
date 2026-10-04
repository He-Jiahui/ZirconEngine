use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::{
    ShaderFeatureBits, ShaderPassType, ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH,
    SHADING_MODEL_ID_STANDARD_PBR,
};
use crate::core::resource::ResourceId;

fn source(label: &str, body: &str) -> ShaderVariantPrewarmSource {
    ShaderVariantPrewarmSource::new(label, body, Vec::new(), "template-r1", "naga-r1", "wgpu-r1")
}

fn request(source: &ShaderVariantPrewarmSource, revision: u64) -> ShaderVariantPrewarmRequest {
    ShaderVariantPrewarmRequest {
        key: ShaderVariantKey {
            material_shader: ResourceId::from_stable_label(source.source_label.as_str()),
            material_revision: revision,
            material_layout_hash: 0,
            material_option_bits: 0,
            geometry_source: GEOMETRY_SOURCE_ID_STATIC_MESH,
            shading_model: SHADING_MODEL_ID_STANDARD_PBR,
            pass_type: ShaderPassType::Forward,
            features: ShaderFeatureBits::new(0),
            quality: ShaderQualityTier::Medium,
            platform_token: "runtime607".to_string(),
        },
        pipeline_state: None,
        source_id: source.id.clone(),
    }
}

#[test]
fn optimization_batch_hx_runtime607_replacement_prunes_only_unreferenced_sources() {
    let first = source("res://first.wgsl", "fn first() {}");
    let second = source("res://second.wgsl", "fn second() {}");
    let replacement = source("res://replacement.wgsl", "fn replacement() {}");
    let mut manifest = ShaderVariantPrewarmManifest::new(
        vec![first.clone(), second.clone()],
        vec![request(&first, 1), request(&second, 2)],
    );

    assert!(manifest.replace_variant_source(0, replacement.clone()));

    assert_eq!(manifest.variants[0].source_id, replacement.id);
    assert_eq!(manifest.variants[1].source_id, second.id);
    assert_eq!(manifest.sources.len(), 2);
    assert!(manifest.sources.iter().any(|source| source.id == second.id));
    assert!(manifest
        .sources
        .iter()
        .any(|source| source.id == replacement.id));
    assert!(manifest.validate_integrity().is_ok());
}

#[test]
fn optimization_batch_hx_runtime607_source_id_membership_is_borrowed() {
    let source = include_str!("../../variant_prewarm.rs");
    let validation = source
        .split("pub fn validate_integrity")
        .nth(1)
        .expect("integrity validation")
        .split("pub fn replace_variant_source")
        .next()
        .expect("bounded integrity validation");
    let replacement = source
        .split("pub fn replace_variant_source")
        .nth(1)
        .expect("variant replacement")
        .split("#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]")
        .next()
        .expect("bounded variant replacement");

    assert!(validation.contains("source_ids.insert(&source.id)"));
    assert!(!validation.contains("source.id.clone()"));
    assert!(replacement.contains("map(|request| &request.source_id)"));
    assert!(!replacement.contains("map(|request| request.source_id.clone())"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_hx_runtime607_borrowed_source_id_membership_performance_evidence() {
    const SOURCE_COUNT: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    let sources = (0..SOURCE_COUNT)
        .map(|index| {
            source(
                &format!("res://performance/source_{index:05}.wgsl"),
                &format!("fn source_{index:05}() {{}}"),
            )
        })
        .collect::<Vec<_>>();
    let measure_owned = || {
        let started = Instant::now();
        black_box(
            black_box(&sources)
                .iter()
                .map(|source| source.id.clone())
                .collect::<HashSet<_>>(),
        );
        started.elapsed().as_nanos().max(1)
    };
    let measure_borrowed = || {
        let started = Instant::now();
        black_box(
            black_box(&sources)
                .iter()
                .map(|source| &source.id)
                .collect::<HashSet<_>>(),
        );
        started.elapsed().as_nanos().max(1)
    };
    for _ in 0..3 {
        black_box(measure_owned());
        black_box(measure_borrowed());
    }

    let mut owned_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            owned_samples.push(measure_owned());
            borrowed_samples.push(measure_borrowed());
        } else {
            borrowed_samples.push(measure_borrowed());
            owned_samples.push(measure_owned());
        }
    }
    owned_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let owned_p50 = owned_samples[8];
    let owned_p95 = owned_samples[16];
    let borrowed_p50 = borrowed_samples[8];
    let borrowed_p95 = borrowed_samples[16];
    println!(
        "RUNTIME607_BORROWED_SOURCE_ID_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} pair_order=alternating_owned_even owned_first_pairs=9 borrowed_first_pairs=8 sources={SOURCE_COUNT} source_id_bytes=64 owned_p50_ns={owned_p50} owned_p95_ns={owned_p95} borrowed_p50_ns={borrowed_p50} borrowed_p95_ns={borrowed_p95} deep_id_clones=16384->0 target_ratio_bp=8500"
    );
    assert!(
        borrowed_p95.saturating_mul(10_000) <= owned_p95.saturating_mul(8_500),
        "borrowed source ID membership P95 {borrowed_p95} ns exceeded 85% of owned {owned_p95} ns"
    );
}
