use std::fmt::{self, Write as _};
use std::hint::black_box;
use std::time::Instant;

use super::RenderMaterialLightingModel;

const PERF_MARKER: &str = "RUNTIME134_LIGHTING_MODEL_ZERO_ALLOCATION_FORMAT_BENCH_V1";

#[test]
fn optimization_batch_20260826cq_runtime_lighting_model_format_preserves_tokens() {
    let cases = [
        (RenderMaterialLightingModel::Pbr, "pbr"),
        (RenderMaterialLightingModel::BlinnPhong, "blinn_phong"),
        (RenderMaterialLightingModel::Unlit, "unlit"),
        (
            RenderMaterialLightingModel::Custom {
                name: "skin".to_owned(),
            },
            "custom:skin",
        ),
    ];

    for (model, expected) in cases {
        assert_eq!(model.to_string(), expected);
        assert_eq!(model.as_token(), expected);
    }
}

#[test]
fn optimization_batch_20260826cq_runtime_lighting_model_format_source_contract() {
    let source = include_str!("../lighting_model.rs");

    assert!(source.contains("const fn builtin_token"));
    assert!(source.contains("f.write_str(token)"));
    assert!(source.contains("serializer.serialize_str(token)"));
    assert!(!source.contains("f.write_str(&self.as_token())"));
    assert_eq!(
        PERF_MARKER,
        "RUNTIME134_LIGHTING_MODEL_ZERO_ALLOCATION_FORMAT_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826cq_runtime_lighting_model_format_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const FORMATS_PER_SAMPLE: usize = 90_000;
    let models = [
        RenderMaterialLightingModel::Pbr,
        RenderMaterialLightingModel::BlinnPhong,
        RenderMaterialLightingModel::Unlit,
    ];

    black_box(measure_legacy(&models, FORMATS_PER_SAMPLE / 10));
    black_box(measure_optimized(&models, FORMATS_PER_SAMPLE / 10));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&models, FORMATS_PER_SAMPLE));
            optimized_ns.push(measure_optimized(&models, FORMATS_PER_SAMPLE));
        } else {
            optimized_ns.push(measure_optimized(&models, FORMATS_PER_SAMPLE));
            legacy_ns.push(measure_legacy(&models, FORMATS_PER_SAMPLE));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} formats_per_sample={FORMATS_PER_SAMPLE} builtin_models=3 order=alternating_legacy_first_even legacy_allocations_per_sample={FORMATS_PER_SAMPLE} optimized_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "zero-allocation built-in formatting must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

#[derive(Default)]
struct CountingWriter(usize);

impl fmt::Write for CountingWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0 = self.0.wrapping_add(value.len());
        Ok(())
    }
}

fn measure_legacy(models: &[RenderMaterialLightingModel], formats: usize) -> u128 {
    let mut output = CountingWriter::default();
    let started = Instant::now();
    for index in 0..formats {
        let model = black_box(&models[index % models.len()]);
        output.write_str(&model.as_token()).unwrap();
    }
    black_box(output.0);
    started.elapsed().as_nanos()
}

fn measure_optimized(models: &[RenderMaterialLightingModel], formats: usize) -> u128 {
    let mut output = CountingWriter::default();
    let started = Instant::now();
    for index in 0..formats {
        let model = black_box(&models[index % models.len()]);
        write!(&mut output, "{model}").unwrap();
    }
    black_box(output.0);
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
