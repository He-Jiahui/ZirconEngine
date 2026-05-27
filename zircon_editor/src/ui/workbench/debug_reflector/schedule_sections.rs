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
    if reasons.is_empty() {
        return "none".to_string();
    }

    reasons
        .iter()
        .map(|reason| format!("{reason:?}"))
        .collect::<Vec<_>>()
        .join(",")
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
    impacts
        .iter()
        .filter(|impact| impact.required || impact.node_count > 0)
        .map(|impact| {
            format!(
                "{}={} nodes reasons={}",
                impact.stage.as_str(),
                impact.node_count,
                dirty_reason_summary(&impact.dirty_reasons)
            )
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn dirty_domain_impact_summary(impacts: &[UiEcsDirtyDomainImpact]) -> String {
    impacts
        .iter()
        .filter(|impact| impact.active || impact.node_count > 0)
        .map(|impact| format!("{:?}={}", impact.domain, impact.node_count))
        .collect::<Vec<_>>()
        .join(",")
}
