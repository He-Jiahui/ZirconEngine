use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 128;
const STAGES_PER_BUILD: usize = 4_096;

#[test]
fn optimization_batch_20260826fr_editor159_invalid_plan_stays_allocation_free() {
    let source = include_str!("../execution.rs");
    let pipeline = source
        .split("pub fn execute_export_wizard_pipeline(")
        .nth(1)
        .expect("pipeline execution implementation");
    let fatal_guard = pipeline.find("if fatal {").expect("fatal guard");
    let reserve = pipeline
        .find("stages.reserve(plan.stages.len());")
        .expect("stage capacity reservation");

    assert!(fatal_guard < reserve);
    assert!(pipeline[fatal_guard..reserve].contains("return ExportWizardPipelineExecution"));
}

#[test]
fn optimization_batch_20260826fr_editor159_pipeline_stages_reserve_plan_upper_bound() {
    let source = include_str!("../execution.rs");
    assert!(source.contains("stages.reserve(plan.stages.len());"));
    assert!(source.contains("let mut stages = Vec::new();"));
    assert_eq!(source.matches("stages.push(stage_execution);").count(), 1);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fr_editor159_export_pipeline_stage_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR159_EXPORT_PIPELINE_STAGE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} stages_per_build={STAGES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct StageExecutionFixture([usize; 5]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let mut stages = Vec::new();
        if reserve {
            stages.reserve(STAGES_PER_BUILD);
        }
        for stage in 0..STAGES_PER_BUILD {
            stages.push(StageExecutionFixture([black_box(build ^ stage); 5]));
        }
        checksum ^= black_box(stages.len() ^ stages.capacity() ^ stages[STAGES_PER_BUILD - 1].0[0]);
        black_box(&stages);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
