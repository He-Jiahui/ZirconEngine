use std::fmt::Write as _;

use zircon_runtime_interface::ui::{
    ecs::{UiEcsDirtyDomainImpact, UiEcsProjectionScheduleImpact, UiEcsProjectionScheduleMask},
    pipeline::{UiPipelineDirtyReason, UiPipelineStageCounters},
    surface::UiSurfaceDebugSnapshot,
};

use super::model::{EditorUiDebugReflectorModel, EditorUiDebugReflectorSection};

impl EditorUiDebugReflectorModel {
    pub(crate) fn with_schedule_sections(mut self, snapshot: &UiSurfaceDebugSnapshot) -> Self {
        self.sections.splice(
            0..0,
            [pipeline_section(snapshot), ecs_projection_section(snapshot)],
        );
        self
    }
}

fn pipeline_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    let report = &snapshot.pipeline_report;
    let missing = report
        .missing_required_stages()
        .iter()
        .map(|stage| stage.as_str())
        .collect::<Vec<_>>();
    let mut lines = vec![
        format!("frame: {}", report.frame_index),
        format!(
            "stages: completed={} total={} ordered={} missing={}",
            report.completed_stage_count(),
            report.stages.len(),
            report.is_complete_ordered(),
            if missing.is_empty() {
                "none".to_string()
            } else {
                missing.join(", ")
            }
        ),
        format!("elapsed micros: {}", report.total_elapsed_micros),
        format!("totals: {}", pipeline_counter_summary(report.totals)),
    ];

    if report.stages.is_empty() {
        lines.push("stage rows: none".to_string());
    } else {
        for stage in &report.stages {
            lines.push(format!(
                "stage={} skipped={} elapsed={} dirty={} counters={}",
                stage.stage.as_str(),
                stage.skipped,
                stage.elapsed_micros,
                dirty_reason_summary(&stage.dirty_reasons),
                pipeline_counter_summary(stage.counters),
            ));
        }
    }

    EditorUiDebugReflectorSection {
        title: "Pipeline".to_string(),
        lines,
    }
}

fn ecs_projection_section(snapshot: &UiSurfaceDebugSnapshot) -> EditorUiDebugReflectorSection {
    let projection = &snapshot.ecs_projection;
    let totals = projection.totals;
    let mut lines = vec![
        format!("tree: {}", projection.tree_id.0),
        format!(
            "nodes: total={} dirty={} roots={}",
            totals.node_count,
            totals.dirty_node_count,
            projection.roots.len()
        ),
        format!(
            "dirty domains: layout={} text={} input={} picking={} a11y={} render={}",
            totals.layout_dirty_count,
            totals.text_dirty_count,
            totals.input_dirty_count,
            totals.picking_dirty_count,
            totals.accessibility_dirty_count,
            totals.render_dirty_count
        ),
        format!(
            "interaction: focused={} hovered={} pressed={} disabled={}",
            totals.focused_count, totals.hovered_count, totals.pressed_count, totals.disabled_count
        ),
        format!(
            "surface facts: render_commands={} hit_entries={}",
            totals.render_command_count, totals.hit_entry_count
        ),
        format!(
            "schedule mask: {}",
            schedule_mask_summary(projection.schedule_mask)
        ),
    ];

    let schedule_impacts = schedule_impact_summary(&projection.schedule_impacts);
    if schedule_impacts.is_empty() {
        lines.push("schedule impacts: none".to_string());
    } else {
        lines.push(format!("schedule impacts: {schedule_impacts}"));
    }

    let dirty_domain_impacts = dirty_domain_impact_summary(&projection.dirty_domain_impacts);
    if dirty_domain_impacts.is_empty() {
        lines.push("dirty-domain impacts: none".to_string());
    } else {
        lines.push(format!("dirty-domain impacts: {dirty_domain_impacts}"));
    }

    EditorUiDebugReflectorSection {
        title: "ECS Projection".to_string(),
        lines,
    }
}

fn dirty_reason_summary(reasons: &[UiPipelineDirtyReason]) -> String {
    let mut summary = String::new();
    append_dirty_reason_summary(&mut summary, reasons);
    summary
}

fn append_dirty_reason_summary(summary: &mut String, reasons: &[UiPipelineDirtyReason]) {
    if reasons.is_empty() {
        summary.push_str("none");
        return;
    }

    for (index, reason) in reasons.iter().enumerate() {
        if index != 0 {
            summary.push(',');
        }
        write!(&mut *summary, "{reason:?}").expect("writing to String cannot fail");
    }
}

fn pipeline_counter_summary(counters: UiPipelineStageCounters) -> String {
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

fn schedule_mask_summary(mask: UiEcsProjectionScheduleMask) -> String {
    let stages = mask
        .pipeline_stages()
        .iter()
        .map(|stage| stage.as_str())
        .collect::<Vec<_>>();
    if stages.is_empty() {
        "none".to_string()
    } else {
        stages.join(",")
    }
}

fn schedule_impact_summary(impacts: &[UiEcsProjectionScheduleImpact]) -> String {
    let mut summary = String::new();
    for impact in impacts
        .iter()
        .filter(|impact| impact.required || impact.node_count > 0)
    {
        if !summary.is_empty() {
            summary.push_str(" | ");
        }
        write!(
            &mut summary,
            "{}={} nodes reasons=",
            impact.stage.as_str(),
            impact.node_count
        )
        .expect("writing to String cannot fail");
        append_dirty_reason_summary(&mut summary, &impact.dirty_reasons);
    }
    summary
}

fn dirty_domain_impact_summary(impacts: &[UiEcsDirtyDomainImpact]) -> String {
    impacts
        .iter()
        .filter(|impact| impact.active || impact.node_count > 0)
        .map(|impact| format!("{:?}={}", impact.domain, impact.node_count))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use zircon_runtime_interface::ui::pipeline::UiPipelineStage;

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
        let source = include_str!("schedule_sections.rs");
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
        let schedule_summary = production
            .split_once("fn schedule_impact_summary")
            .expect("schedule impact summary")
            .1
            .split_once("fn dirty_domain_impact_summary")
            .expect("schedule impact summary end")
            .0;

        assert!(!dirty_summary.contains("collect::<Vec"));
        assert!(!schedule_summary.contains("collect::<Vec"));
        assert!(dirty_summary.contains("write!("));
        assert!(schedule_summary.contains("append_dirty_reason_summary"));
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
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(60),
            "single-buffer schedule summary must reduce P95 by at least 40%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
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
}
