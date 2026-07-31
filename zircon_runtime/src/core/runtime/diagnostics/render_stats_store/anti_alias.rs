use crate::core::framework::render::{AntiAliasFallbackReason, AntiAliasMode, RenderStats};

use super::{DiagnosticStore, record_bool, record_count};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = stats.last_anti_alias_fallback;
    let fallback_reason_count = usize::from(report.reason.is_some());

    record_bool(
        store,
        "render.anti_alias.requested_post_process",
        frame_index,
        report.requested_mode.is_post_process(),
        &["render", "anti_alias", "requested"],
    );
    record_bool(
        store,
        "render.anti_alias.requested_requires_history",
        frame_index,
        report.requested_mode.requires_history(),
        &["render", "anti_alias", "requested", "history"],
    );
    record_count(
        store,
        "render.anti_alias.requested_msaa_sample_count",
        frame_index,
        msaa_sample_count(report.requested_mode),
        &["render", "anti_alias", "requested", "msaa"],
    );
    record_bool(
        store,
        "render.anti_alias.effective_post_process",
        frame_index,
        report.effective_mode.is_post_process(),
        &["render", "anti_alias", "effective"],
    );
    record_bool(
        store,
        "render.anti_alias.effective_requires_history",
        frame_index,
        report.effective_mode.requires_history(),
        &["render", "anti_alias", "effective", "history"],
    );
    record_count(
        store,
        "render.anti_alias.effective_msaa_sample_count",
        frame_index,
        msaa_sample_count(report.effective_mode),
        &["render", "anti_alias", "effective", "msaa"],
    );
    record_count(
        store,
        "render.anti_alias.graph_requested_msaa_sample_count",
        frame_index,
        stats.last_graph_requested_msaa_sample_count as usize,
        &["render", "anti_alias", "graph", "requested", "msaa"],
    );
    record_count(
        store,
        "render.anti_alias.graph_effective_msaa_sample_count",
        frame_index,
        stats.last_graph_effective_msaa_sample_count as usize,
        &["render", "anti_alias", "graph", "effective", "msaa"],
    );
    record_bool(
        store,
        "render.anti_alias.fallback.active",
        frame_index,
        report.reason.is_some(),
        &["render", "anti_alias", "fallback"],
    );
    record_count(
        store,
        "render.anti_alias.fallback.reason_count",
        frame_index,
        fallback_reason_count,
        &["render", "anti_alias", "fallback"],
    );
    record_bool(
        store,
        "render.anti_alias.fallback.missing_history",
        frame_index,
        matches!(report.reason, Some(AntiAliasFallbackReason::MissingHistory)),
        &["render", "anti_alias", "fallback", "history"],
    );
    record_bool(
        store,
        "render.anti_alias.normalization.active",
        frame_index,
        report.normalization_count() > 0,
        &["render", "anti_alias", "normalization"],
    );
    record_count(
        store,
        "render.anti_alias.normalization.count",
        frame_index,
        report.normalization_count(),
        &["render", "anti_alias", "normalization"],
    );
    record_bool(
        store,
        "render.anti_alias.normalization.graph_sample_count",
        frame_index,
        report.graph_sample_count_normalized,
        &["render", "anti_alias", "normalization", "graph", "msaa"],
    );
    record_bool(
        store,
        "render.anti_alias.normalization.taa_msaa_conflict",
        frame_index,
        report.taa_msaa_conflict_normalized(),
        &["render", "anti_alias", "normalization", "taa", "msaa"],
    );
    record_bool(
        store,
        "render.anti_alias.normalization.terminal_slot",
        frame_index,
        report.terminal_slot_normalized,
        &["render", "anti_alias", "normalization", "terminal"],
    );
}

const fn msaa_sample_count(mode: AntiAliasMode) -> usize {
    match mode {
        AntiAliasMode::Msaa { samples } => samples as usize,
        _ => 0,
    }
}
