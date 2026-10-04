use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use zircon_runtime_interface::ui::{ecs::UiEcsDirtyDomainKind, pipeline::UiPipelineStage};

const BENCHMARK_IMPACT_COUNT: usize = 4_096;
const BENCHMARK_SAMPLES: usize = 11;
const BENCHMARK_ITERATIONS: usize = 64;

#[test]
fn single_buffer_schedule_summary_preserves_bytes_and_filtering() {
    let impacts = vec![
        UiEcsProjectionScheduleImpact {
            stage: UiPipelineStage::InputCollect,
            required: true,
            dirty_reasons: vec![
                UiPipelineDirtyReason::Input,
                UiPipelineDirtyReason::Diagnostics,
            ],
            node_count: 0,
            ..UiEcsProjectionScheduleImpact::default()
        },
        UiEcsProjectionScheduleImpact {
            stage: UiPipelineStage::Layout,
            required: false,
            dirty_reasons: vec![UiPipelineDirtyReason::Layout],
            node_count: 7,
            ..UiEcsProjectionScheduleImpact::default()
        },
        UiEcsProjectionScheduleImpact::default(),
    ];

    assert_eq!(
        schedule_impact_summary(&impacts),
        retired_schedule_impact_summary(&impacts)
    );
    for reasons in [
        vec![],
        vec![UiPipelineDirtyReason::Render],
        vec![
            UiPipelineDirtyReason::Text,
            UiPipelineDirtyReason::LayoutMetrics,
        ],
    ] {
        assert_eq!(
            dirty_reason_summary(&reasons),
            retired_dirty_reason_summary(&reasons)
        );
    }
}

#[test]
fn single_buffer_schedule_summary_source_contract() {
    let source = include_str!("../schedule_sections.rs");
    let production = source
        .split_once("#[cfg(test)]")
        .expect("production module end")
        .0;
    let dirty_summary = production
        .split_once("fn dirty_reason_summary")
        .expect("dirty reason summary")
        .1
        .split_once("fn pipeline_counter_summary")
        .expect("dirty reason summary end")
        .0;
    let counter_summary = production
        .split_once("fn pipeline_counter_summary")
        .expect("pipeline counter summary")
        .1
        .split_once("fn schedule_mask_summary")
        .expect("pipeline counter summary end")
        .0;
    let schedule_summary = production
        .split_once("fn schedule_impact_summary")
        .expect("schedule impact summary")
        .1
        .split_once("fn dirty_domain_impact_summary")
        .expect("schedule impact summary end")
        .0;
    let dirty_domain_summary = production
        .split_once("fn dirty_domain_impact_summary")
        .expect("dirty-domain impact summary")
        .1;

    assert!(!dirty_summary.contains("collect::<Vec"));
    assert!(!counter_summary.contains("collect::<Vec"));
    assert!(!counter_summary.contains("format!("));
    assert!(!schedule_summary.contains("collect::<Vec"));
    assert!(!dirty_domain_summary.contains("collect::<Vec"));
    assert!(!dirty_domain_summary.contains("format!("));
    assert!(dirty_summary.contains("write!("));
    assert!(counter_summary.contains("write!("));
    assert!(schedule_summary.contains("append_dirty_reason_summary"));
    assert!(dirty_domain_summary.contains("write!("));
}

#[test]
fn single_buffer_dirty_domain_summary_preserves_bytes_and_filtering() {
    let impacts = vec![
        UiEcsDirtyDomainImpact {
            domain: UiEcsDirtyDomainKind::Layout,
            active: true,
            node_count: 0,
            ..UiEcsDirtyDomainImpact::default()
        },
        UiEcsDirtyDomainImpact {
            domain: UiEcsDirtyDomainKind::Text,
            active: false,
            node_count: 3,
            ..UiEcsDirtyDomainImpact::default()
        },
        UiEcsDirtyDomainImpact {
            domain: UiEcsDirtyDomainKind::Input,
            active: false,
            node_count: 0,
            ..UiEcsDirtyDomainImpact::default()
        },
        UiEcsDirtyDomainImpact {
            domain: UiEcsDirtyDomainKind::Render,
            active: true,
            node_count: 2,
            ..UiEcsDirtyDomainImpact::default()
        },
    ];

    assert_eq!(
        dirty_domain_impact_summary(&impacts),
        retired_dirty_domain_impact_summary(&impacts)
    );
    assert_eq!(
        dirty_domain_impact_summary(&impacts),
        "Layout=0,Text=3,Render=2"
    );
    assert!(dirty_domain_impact_summary(&[]).is_empty());
}

#[test]
fn single_buffer_pipeline_counter_summary_preserves_bytes_and_filtering() {
    let counters = UiPipelineStageCounters {
        input_event_count: 1,
        text_measure_count: 2,
        render_extract_command_count: 3,
        batch_count: 4,
        ..UiPipelineStageCounters::default()
    };

    assert_eq!(
        pipeline_counter_summary(counters),
        retired_pipeline_counter_summary(counters)
    );
    assert_eq!(
        pipeline_counter_summary(counters),
        "input=1,text=2,render=3,batch=4"
    );
    assert_eq!(
        pipeline_counter_summary(UiPipelineStageCounters::default()),
        "none"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn single_buffer_schedule_summary_release_benchmark() {
    let impacts = (0..BENCHMARK_IMPACT_COUNT)
        .map(|index| UiEcsProjectionScheduleImpact {
            stage: UiPipelineStage::RenderExtract,
            required: true,
            dirty_reasons: vec![
                UiPipelineDirtyReason::Render,
                UiPipelineDirtyReason::Diagnostics,
            ],
            node_count: index as u64 + 1,
            ..UiEcsProjectionScheduleImpact::default()
        })
        .collect::<Vec<_>>();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_summary(|| {
                retired_schedule_impact_summary(&impacts)
            }));
            optimized_samples.push(measure_summary(|| schedule_impact_summary(&impacts)));
        } else {
            optimized_samples.push(measure_summary(|| schedule_impact_summary(&impacts)));
            retired_samples.push(measure_summary(|| {
                retired_schedule_impact_summary(&impacts)
            }));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "EDITOR25_SINGLE_BUFFER_SCHEDULE_SUMMARY_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
impacts={BENCHMARK_IMPACT_COUNT} dirty_reasons_per_impact=2 \
retired_intermediate_strings_per_summary=16384 optimized_intermediate_strings_per_summary=0 \
retired_temporary_vec_buffers_per_summary=4097 optimized_temporary_vec_buffers_per_summary=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(60),
        "single-buffer schedule summary must reduce P95 by at least 40%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

#[test]
#[ignore = "release-only performance evidence"]
fn editor824_single_buffer_dirty_domain_summary_bench_v1() {
    let impacts = (0..BENCHMARK_IMPACT_COUNT)
        .map(|index| UiEcsDirtyDomainImpact {
            domain: UiEcsDirtyDomainKind::ORDER[index % UiEcsDirtyDomainKind::ORDER.len()],
            active: index % 2 == 0,
            node_count: if index % 2 == 0 { 0 } else { index as u64 + 1 },
            ..UiEcsDirtyDomainImpact::default()
        })
        .collect::<Vec<_>>();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_summary(|| {
                retired_dirty_domain_impact_summary(&impacts)
            }));
            optimized_samples.push(measure_summary(|| dirty_domain_impact_summary(&impacts)));
        } else {
            optimized_samples.push(measure_summary(|| dirty_domain_impact_summary(&impacts)));
            retired_samples.push(measure_summary(|| {
                retired_dirty_domain_impact_summary(&impacts)
            }));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        concat!(
            "EDITOR824_SINGLE_BUFFER_DIRTY_DOMAIN_SUMMARY_BENCH_V1 ",
            "samples={} iterations={} impacts={} ",
            "retired_intermediate_strings_per_summary={} ",
            "optimized_intermediate_strings_per_summary=0 ",
            "retired_temporary_vec_buffers_per_summary=1 ",
            "optimized_temporary_vec_buffers_per_summary=0 ",
            "retired_p95_ns={} optimized_p95_ns={} ",
            "reduction_basis_points={}"
        ),
        BENCHMARK_SAMPLES,
        BENCHMARK_ITERATIONS,
        BENCHMARK_IMPACT_COUNT,
        BENCHMARK_IMPACT_COUNT,
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
        reduction_basis_points,
    );
    assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "single-buffer dirty-domain summary must reduce P95 by at least 20%: retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
}

#[test]
#[ignore = "release-only performance evidence"]
fn editor825_single_buffer_pipeline_counter_summary_bench_v1() {
    let counters = UiPipelineStageCounters {
        input_event_count: 1,
        pointer_move_count: 2,
        focus_change_count: 3,
        widget_behavior_count: 4,
        text_measure_count: 5,
        layout_node_count: 6,
        picking_candidate_count: 7,
        accessibility_node_count: 8,
        render_extract_command_count: 9,
        batch_count: 10,
        ..UiPipelineStageCounters::default()
    };
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_summary(|| {
                retired_pipeline_counter_summary(counters)
            }));
            optimized_samples.push(measure_summary(|| pipeline_counter_summary(counters)));
        } else {
            optimized_samples.push(measure_summary(|| pipeline_counter_summary(counters)));
            retired_samples.push(measure_summary(|| {
                retired_pipeline_counter_summary(counters)
            }));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        concat!(
            "EDITOR825_SINGLE_BUFFER_PIPELINE_COUNTER_SUMMARY_BENCH_V1 ",
            "samples={} iterations={} active_counters=10 ",
            "retired_intermediate_strings_per_summary=10 ",
            "optimized_intermediate_strings_per_summary=0 ",
            "retired_temporary_vec_buffers_per_summary=1 ",
            "optimized_temporary_vec_buffers_per_summary=0 ",
            "retired_p95_ns={} optimized_p95_ns={} ",
            "reduction_basis_points={}"
        ),
        BENCHMARK_SAMPLES,
        BENCHMARK_ITERATIONS,
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
        reduction_basis_points,
    );
    assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(80),
            "single-buffer pipeline-counter summary must reduce P95 by at least 20%: retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
}

fn retired_pipeline_counter_summary(counters: UiPipelineStageCounters) -> String {
    let entries = [
        ("input", counters.input_event_count),
        ("pointer_move", counters.pointer_move_count),
        ("focus", counters.focus_change_count),
        ("widget", counters.widget_behavior_count),
        ("text", counters.text_measure_count),
        ("layout", counters.layout_node_count),
        ("picking", counters.picking_candidate_count),
        ("a11y", counters.accessibility_node_count),
        ("render", counters.render_extract_command_count),
        ("batch", counters.batch_count),
    ];
    let active = entries
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(name, count)| format!("{name}={count}"))
        .collect::<Vec<_>>();

    if active.is_empty() {
        "none".to_string()
    } else {
        active.join(",")
    }
}

fn retired_dirty_reason_summary(reasons: &[UiPipelineDirtyReason]) -> String {
    if reasons.is_empty() {
        return "none".to_string();
    }

    reasons
        .iter()
        .map(|reason| format!("{reason:?}"))
        .collect::<Vec<_>>()
        .join(",")
}

fn retired_schedule_impact_summary(impacts: &[UiEcsProjectionScheduleImpact]) -> String {
    impacts
        .iter()
        .filter(|impact| impact.required || impact.node_count > 0)
        .map(|impact| {
            format!(
                "{}={} nodes reasons={}",
                impact.stage.as_str(),
                impact.node_count,
                retired_dirty_reason_summary(&impact.dirty_reasons)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn retired_dirty_domain_impact_summary(impacts: &[UiEcsDirtyDomainImpact]) -> String {
    impacts
        .iter()
        .filter(|impact| impact.active || impact.node_count > 0)
        .map(|impact| format!("{:?}={}", impact.domain, impact.node_count))
        .collect::<Vec<_>>()
        .join(",")
}

fn measure_summary(mut summarize: impl FnMut() -> String) -> Duration {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        black_box(summarize());
    }
    started.elapsed()
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
