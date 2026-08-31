use std::collections::HashSet;

use zircon_runtime::graphics::{RealtimeIblCpuTimingReport, RealtimeIblGpuTimingReport};

use super::REALTIME_GENERATION_TICKET_FRAME_COUNT;

pub(super) fn assert_realtime_gpu_timings(
    reports: &[RealtimeIblGpuTimingReport],
    ticket_count: usize,
) {
    assert_eq!(
        reports.len(),
        REALTIME_GENERATION_TICKET_FRAME_COUNT * ticket_count,
        "each cold and warm ticket frame must produce one GPU timestamp"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.elapsed_gpu_nanoseconds > 0.0),
        "every realtime IBL ticket operation must consume measurable GPU time: {reports:?}"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.pass_count == 1 && report.dispatch_count == 1),
        "a ticket frame must record exactly one graph pass and dispatch: {reports:?}"
    );
    assert!(
        reports.iter().all(|report| report.scheduled_workgroups > 0),
        "every ticket operation must report its dispatched workgroups: {reports:?}"
    );
    assert!(
        reports
            .iter()
            .all(|report| report.completed_workgroups == report.scheduled_workgroups),
        "GPU timestamp readback must report every scheduled workgroup as completed: {reports:?}"
    );
    assert!(
        reports
            .iter()
            .all(|report| !report.recipe_fingerprint.is_empty()),
        "every ticket operation must identify its complete bake recipe: {reports:?}"
    );
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.operation_label == "diffuse_sh9")
            .count(),
        ticket_count,
        "each complete ticket must timestamp its terminal SH9 operation"
    );
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.terminal_reason == "published_after_sh9")
            .count(),
        ticket_count,
        "only each terminal SH9 operation may publish a ticket"
    );
    let generations = reports
        .iter()
        .map(|report| report.generation)
        .collect::<HashSet<_>>();
    assert!(
        generations.len() >= ticket_count,
        "initial, updated, and warm environments must publish distinct generation timings: {reports:?}"
    );
}

pub(super) fn assert_realtime_binding_cache_metrics(reports: &[RealtimeIblGpuTimingReport]) {
    assert!(
        reports.iter().all(|report| {
            report.params_buffer_creations == report.binding_cache_misses
                && report.bind_group_creations == report.binding_cache_misses
                && report.binding_cache_hits + report.binding_cache_misses <= 1
                && report.binding_cache_resets <= 1
        }),
        "each product realtime IBL operation must report coherent binding-cache counters: {reports:?}"
    );
    assert!(
        reports.iter().any(|report| report.binding_cache_misses > 0),
        "the product run must include cold binding-template construction: {reports:?}"
    );
    let warm_generation = reports
        .iter()
        .map(|report| report.generation)
        .max()
        .expect("the product run must report a warm-cache generation");
    let warm_reports = reports
        .iter()
        .filter(|report| report.generation == warm_generation)
        .collect::<Vec<_>>();
    assert_eq!(
        warm_reports.len(),
        REALTIME_GENERATION_TICKET_FRAME_COUNT,
        "the third generation must contain one complete realtime IBL ticket: {warm_reports:?}"
    );
    assert!(
        warm_reports
            .iter()
            .all(|report| report.binding_cache_resets == 0),
        "a same-layout warm generation must not reset binding templates: {warm_reports:?}"
    );
    let warm_binding_reports = warm_reports
        .iter()
        .filter(|report| report.binding_cache_hits + report.binding_cache_misses == 1)
        .collect::<Vec<_>>();
    assert_eq!(
        warm_binding_reports.len(),
        11,
        "a default ticket must issue ten PMREM and one SH9 binding command: {warm_binding_reports:?}"
    );
    assert!(
        warm_binding_reports.iter().all(|report| {
            report.binding_cache_hits == 1
                && report.binding_cache_misses == 0
                && report.params_buffer_creations == 0
                && report.bind_group_creations == 0
        }),
        "the third same-layout ticket must reuse its B-slot binding templates: {warm_binding_reports:?}"
    );
}

pub(super) fn assert_realtime_capture_and_source_mip_binding_metrics(
    reports: &[RealtimeIblGpuTimingReport],
    ticket_count: usize,
) {
    let capture_reports = reports
        .iter()
        .filter(|report| report.operation_label == "capture_sky")
        .collect::<Vec<_>>();
    assert_eq!(
        capture_reports.len(),
        ticket_count * 3,
        "each default ticket must expose its three dynamic CaptureSky binding creations: {capture_reports:?}"
    );
    assert!(
        capture_reports.iter().all(|report| {
            report.capture_params_buffer_creations == 1
                && report.capture_bind_group_creations == 1
                && report.source_mip_params_buffer_creations == 0
                && report.source_mip_bind_group_creations == 0
        }),
        "each CaptureSky operation must report only its own dynamic binding creation: {capture_reports:?}"
    );

    let source_mip_reports = reports
        .iter()
        .filter(|report| report.operation_label == "source_mip")
        .collect::<Vec<_>>();
    assert_eq!(
        source_mip_reports.len(),
        ticket_count * 7,
        "each default ticket must expose its seven source-mip binding creations: {source_mip_reports:?}"
    );
    assert!(
        source_mip_reports.iter().all(|report| {
            report.capture_params_buffer_creations == 0
                && report.capture_bind_group_creations == 0
                && report.source_mip_params_buffer_creations == 1
                && report.source_mip_bind_group_creations == 1
        }),
        "each source-mip operation must report only its own binding creation: {source_mip_reports:?}"
    );
}

pub(super) fn assert_realtime_cpu_timings(
    reports: &[RealtimeIblCpuTimingReport],
    ticket_count: usize,
) {
    assert_eq!(
        reports.len(),
        REALTIME_GENERATION_TICKET_FRAME_COUNT * ticket_count,
        "each accepted realtime IBL ticket operation must yield one CPU recording report"
    );
    assert!(
        reports.iter().all(|report| {
            report.profile_capture_epoch > 0
                && report.generation_start_frame_number > 0
                && report.generation_elapsed_frame_count > 0
                && report.generation_elapsed_frame_count
                    == report
                        .frame_number
                        .wrapping_sub(report.generation_start_frame_number)
                        .wrapping_add(1)
                && report.pass_count == 1
                && report.dispatch_count == 1
                && report.scheduled_workgroups > 0
                && !report.recipe_fingerprint.is_empty()
                && report.overwritten_report_count == 0
                && report.execution_resource_cache_hits + report.execution_resource_cache_misses
                    == 1
                && report.execution_resource_cache_topology_capacity > 0
                && report.execution_resource_cache_entry_count
                    <= report.execution_resource_cache_topology_capacity
        }),
        "CPU timing reports must identify accepted, bounded realtime IBL work: {reports:?}"
    );
    let epochs = reports
        .iter()
        .map(|report| report.profile_capture_epoch)
        .collect::<HashSet<_>>();
    assert_eq!(
        epochs.len(),
        1,
        "a product profile must not mix asynchronous capture epochs: {reports:?}"
    );
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.operation_label == "diffuse_sh9")
            .count(),
        ticket_count,
        "each complete ticket must report its terminal SH9 CPU recording window"
    );
    assert_eq!(
        reports
            .iter()
            .filter(|report| report.terminal_reason == "published_after_sh9")
            .count(),
        ticket_count,
        "only terminal SH9 reports may publish a CPU-profiled ticket"
    );
}

pub(super) fn gpu_timing_report(reports: &[RealtimeIblGpuTimingReport]) -> String {
    let average_millis = reports
        .iter()
        .map(|report| report.elapsed_gpu_nanoseconds / 1_000_000.0)
        .sum::<f64>()
        / reports.len().max(1) as f64;
    let maximum_millis = reports
        .iter()
        .map(|report| report.elapsed_gpu_nanoseconds / 1_000_000.0)
        .fold(0.0_f64, f64::max);
    let samples = reports
        .iter()
        .map(|report| {
            format!(
                "frame_{:02}_gpu_ms={:.6} generation={} recipe={} state={} work_slot={} passes={} dispatches={} binding_cache_hits={} binding_cache_misses={} params_buffer_creations={} bind_group_creations={} binding_cache_resets={} capture_params_buffer_creations={} capture_bind_group_creations={} source_mip_params_buffer_creations={} source_mip_bind_group_creations={} scheduled_workgroups={} completed_workgroups={} terminal_reason={} operation={}",
                report.frame_number,
                report.elapsed_gpu_nanoseconds / 1_000_000.0,
                report.generation,
                report.recipe_fingerprint,
                report.logical_state,
                report.work_slot,
                report.pass_count,
                report.dispatch_count,
                report.binding_cache_hits,
                report.binding_cache_misses,
                report.params_buffer_creations,
                report.bind_group_creations,
                report.binding_cache_resets,
                report.capture_params_buffer_creations,
                report.capture_bind_group_creations,
                report.source_mip_params_buffer_creations,
                report.source_mip_bind_group_creations,
                report.scheduled_workgroups,
                report.completed_workgroups,
                report.terminal_reason,
                report.operation_label,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "timestamp_query_supported=true\nticket_operation_sample_count={}\nticket_operation_average_gpu_ms={average_millis:.6}\nticket_operation_max_gpu_ms={maximum_millis:.6}\n{samples}\n",
        reports.len()
    )
}

pub(super) fn cpu_timing_report(reports: &[RealtimeIblCpuTimingReport]) -> String {
    let command_plan_creation_total_micros = reports
        .iter()
        .map(|report| report.command_plan_creation_micros)
        .sum::<u64>();
    let pipeline_ensure_total_micros = reports
        .iter()
        .map(|report| report.pipeline_ensure_micros)
        .sum::<u64>();
    let binding_creation_total_micros = reports
        .iter()
        .map(|report| report.binding_creation_micros)
        .sum::<u64>();
    let resource_binding_total_micros = reports
        .iter()
        .map(|report| report.execution_resource_binding_micros)
        .sum::<u64>();
    let validation_total_micros = reports
        .iter()
        .map(|report| report.validation_micros)
        .sum::<u64>();
    let execution_resource_cache_hits = reports
        .iter()
        .map(|report| report.execution_resource_cache_hits)
        .sum::<u64>();
    let execution_resource_cache_misses = reports
        .iter()
        .map(|report| report.execution_resource_cache_misses)
        .sum::<u64>();
    let execution_resource_cache_entry_peak = reports
        .iter()
        .map(|report| report.execution_resource_cache_entry_count)
        .max()
        .unwrap_or_default();
    let execution_resource_cache_topology_capacity = reports
        .iter()
        .map(|report| report.execution_resource_cache_topology_capacity)
        .max()
        .unwrap_or_default();
    let samples = reports
        .iter()
        .map(|report| {
            format!(
                "frame_{:02}_cpu_recording generation={} generation_start_frame={} generation_elapsed_frames={} coalesced_source_changes={} queued_generation_pending={} recipe={} state={} work_slot={} passes={} dispatches={} binding_cache_hits={} binding_cache_misses={} params_buffer_creations={} bind_group_creations={} binding_cache_resets={} command_plan_creation_micros={} pipeline_ensure_micros={} binding_creation_micros={} capture_params_buffer_creations={} capture_bind_group_creations={} capture_binding_creation_micros={} source_mip_params_buffer_creations={} source_mip_bind_group_creations={} source_mip_binding_creation_micros={} execution_resource_binding_micros={} validation_micros={} execution_resource_cache_hits={} execution_resource_cache_misses={} execution_resource_cache_entry_count={} execution_resource_cache_topology_capacity={} texture_view_binding_count={} buffer_binding_count={} total_bound_resource_count={} scheduled_workgroups={} terminal_reason={} operation={} overwrite_count={}",
                report.frame_number,
                report.generation,
                report.generation_start_frame_number,
                report.generation_elapsed_frame_count,
                report.coalesced_source_change_count,
                report.queued_generation_pending,
                report.recipe_fingerprint,
                report.logical_state,
                report.work_slot,
                report.pass_count,
                report.dispatch_count,
                report.binding_cache_hits,
                report.binding_cache_misses,
                report.params_buffer_creations,
                report.bind_group_creations,
                report.binding_cache_resets,
                report.command_plan_creation_micros,
                report.pipeline_ensure_micros,
                report.binding_creation_micros,
                report.capture_params_buffer_creations,
                report.capture_bind_group_creations,
                report.capture_binding_creation_micros,
                report.source_mip_params_buffer_creations,
                report.source_mip_bind_group_creations,
                report.source_mip_binding_creation_micros,
                report.execution_resource_binding_micros,
                report.validation_micros,
                report.execution_resource_cache_hits,
                report.execution_resource_cache_misses,
                report.execution_resource_cache_entry_count,
                report.execution_resource_cache_topology_capacity,
                report.texture_view_binding_count,
                report.buffer_binding_count,
                report.total_bound_resource_count,
                report.scheduled_workgroups,
                report.terminal_reason,
                report.operation_label,
                report.overwritten_report_count,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let capture_epoch = reports
        .first()
        .map(|report| report.profile_capture_epoch)
        .unwrap_or_default();
    format!(
        "clock_domain=cpu_command_recording_only\nprofile_capture_epoch={capture_epoch}\nticket_operation_sample_count={}\ncommand_plan_creation_total_micros={command_plan_creation_total_micros}\npipeline_ensure_total_micros={pipeline_ensure_total_micros}\nbinding_creation_total_micros={binding_creation_total_micros}\nexecution_resource_binding_total_micros={resource_binding_total_micros}\nvalidation_total_micros={validation_total_micros}\nexecution_resource_cache_hits={execution_resource_cache_hits}\nexecution_resource_cache_misses={execution_resource_cache_misses}\nexecution_resource_cache_entry_peak={execution_resource_cache_entry_peak}\nexecution_resource_cache_topology_capacity={execution_resource_cache_topology_capacity}\n{samples}\n",
        reports.len()
    )
}
