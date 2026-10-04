use std::{hint::black_box, time::Instant};

use super::UiPipelineFrameReport;
use crate::ui::pipeline::{UiPipelineStage, UiPipelineStageReport};

const LOOKUP_COUNT: usize = 100_000;
const SAMPLE_COUNT: usize = 11;

fn report_with(stages: impl IntoIterator<Item = UiPipelineStage>) -> UiPipelineFrameReport {
    UiPipelineFrameReport::from_stage_reports(
        1,
        stages
            .into_iter()
            .map(|stage| UiPipelineStageReport::skipped(stage, Vec::new()))
            .collect(),
    )
}

#[test]
fn runtime_interface03_batch18_missing_stage_index_preserves_archived_semantics() {
    let report = report_with([
        UiPipelineStage::InputCollect,
        UiPipelineStage::Layout,
        UiPipelineStage::Diagnostics,
    ]);
    assert_eq!(
        report.missing_required_stages(),
        report.missing_required_stages_linear(),
    );
    assert!(!report
        .missing_required_stages()
        .contains(&UiPipelineStage::Diagnostics));
}

#[test]
fn runtime_interface03_batch19_stage_report_index_preserves_sparse_fallback() {
    let canonical = report_with(UiPipelineStage::ordered().iter().copied());
    let sparse = report_with([
        UiPipelineStage::Layout,
        UiPipelineStage::InputCollect,
        UiPipelineStage::Diagnostics,
    ]);
    for report in [&canonical, &sparse] {
        for stage in [
            UiPipelineStage::InputCollect,
            UiPipelineStage::Layout,
            UiPipelineStage::BatchPrepare,
            UiPipelineStage::Diagnostics,
        ] {
            assert_eq!(
                report.stage_report(stage),
                report.stage_report_linear(stage)
            );
        }
    }
}

#[test]
#[ignore = "release-only indexed pipeline stage-report lookup benchmark"]
fn runtime_interface03_batch19_stage_report_index_release_benchmark() {
    let report = report_with(UiPipelineStage::ordered().iter().copied());
    let stage = UiPipelineStage::BatchPrepare;
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(report.stage_report_linear(black_box(stage)));
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(report.stage_report(black_box(stage)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_STAGE_REPORT_INDEX_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(2) <= linear_samples[p95],
        "indexed stage-report lookup must improve P95 by at least 50%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}

#[test]
#[ignore = "release-only missing pipeline-stage index benchmark"]
fn runtime_interface03_batch18_missing_stage_index_release_benchmark() {
    let report = report_with(
        UiPipelineStage::ordered()
            .iter()
            .copied()
            .chain([UiPipelineStage::Diagnostics]),
    );
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(report.missing_required_stages_linear());
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(report.missing_required_stages());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    indexed_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_MISSING_STAGE_INDEX_BENCH_V1 lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} indexed_p95_ns={}",
        linear_samples[p95],
        indexed_samples[p95],
    );
    assert!(
        indexed_samples[p95].saturating_mul(2) <= linear_samples[p95],
        "indexed missing-stage projection must improve P95 by at least 50%: linear={}ns indexed={}ns",
        linear_samples[p95],
        indexed_samples[p95],
    );
}
