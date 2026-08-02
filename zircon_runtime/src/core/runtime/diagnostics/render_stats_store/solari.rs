use crate::core::framework::render::{RenderStats, SolariDegradationReason, SolariRuntimeStatus};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let report = &stats.last_solari_runtime_report;
    record_bool(
        store,
        "render.solari.requested",
        frame_index,
        report.requested,
        &["render", "solari", "requested"],
    );
    record_bool(
        store,
        "render.solari.enabled",
        frame_index,
        report.enabled(),
        &["render", "solari", "enabled"],
    );
    record_bool(
        store,
        "render.solari.provider_present",
        frame_index,
        report.provider_id.is_some(),
        &["render", "solari", "provider"],
    );
    record_bool(
        store,
        "render.solari.settings.experimental_enabled",
        frame_index,
        report.settings.experimental_enabled,
        &["render", "solari", "settings"],
    );
    record_status(store, frame_index, report.status);
    record_degradations(store, frame_index, stats);
}

fn record_status(store: &mut DiagnosticStore, frame_index: u64, status: SolariRuntimeStatus) {
    record_bool(
        store,
        "render.solari.status.not_requested",
        frame_index,
        status == SolariRuntimeStatus::NotRequested,
        &["render", "solari", "status"],
    );
    record_bool(
        store,
        "render.solari.status.ready",
        frame_index,
        status == SolariRuntimeStatus::Ready,
        &["render", "solari", "status", "ready"],
    );
    record_bool(
        store,
        "render.solari.status.capability_missing",
        frame_index,
        status == SolariRuntimeStatus::CapabilityMissing,
        &["render", "solari", "status", "capability"],
    );
    record_bool(
        store,
        "render.solari.status.provider_missing",
        frame_index,
        status == SolariRuntimeStatus::ProviderMissing,
        &["render", "solari", "status", "provider"],
    );
    record_bool(
        store,
        "render.solari.status.experimental_disabled",
        frame_index,
        status == SolariRuntimeStatus::ExperimentalDisabled,
        &["render", "solari", "status", "experimental"],
    );
    record_bool(
        store,
        "render.solari.status.unavailable",
        frame_index,
        status == SolariRuntimeStatus::Unavailable,
        &["render", "solari", "status", "unavailable"],
    );
}

fn record_degradations(store: &mut DiagnosticStore, frame_index: u64, stats: &RenderStats) {
    let degradations = &stats.last_solari_runtime_report.degradations;
    record_count(
        store,
        "render.solari.degradation_count",
        frame_index,
        degradations.len(),
        &["render", "solari", "degradation"],
    );
    record_count(
        store,
        "render.solari.backend_capability_missing_degradation_count",
        frame_index,
        degradations
            .iter()
            .filter(|degradation| {
                degradation.reason == SolariDegradationReason::BackendCapabilityMissing
            })
            .count(),
        &["render", "solari", "degradation", "capability"],
    );
    record_count(
        store,
        "render.solari.provider_missing_degradation_count",
        frame_index,
        degradations
            .iter()
            .filter(|degradation| degradation.reason == SolariDegradationReason::ProviderMissing)
            .count(),
        &["render", "solari", "degradation", "provider"],
    );
    record_count(
        store,
        "render.solari.experimental_disabled_degradation_count",
        frame_index,
        degradations
            .iter()
            .filter(|degradation| {
                degradation.reason == SolariDegradationReason::ExperimentalDisabled
            })
            .count(),
        &["render", "solari", "degradation", "experimental"],
    );
    record_count(
        store,
        "render.solari.provider_unavailable_degradation_count",
        frame_index,
        degradations
            .iter()
            .filter(|degradation| {
                degradation.reason == SolariDegradationReason::ProviderUnavailable
            })
            .count(),
        &["render", "solari", "degradation", "unavailable"],
    );
}
