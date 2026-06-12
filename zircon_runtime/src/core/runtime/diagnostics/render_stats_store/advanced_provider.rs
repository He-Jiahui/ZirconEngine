use crate::core::framework::render::{
    AdvancedProviderReport, AdvancedProviderStatus, AdvancedRenderDegradationReason,
    AdvancedRenderFeature, RenderStats,
};

use super::{record_bool, record_count, DiagnosticStore};

pub(super) fn record(store: &mut DiagnosticStore, stats: &RenderStats) {
    record_availability(store, stats);
    record_reports(store, stats);
    record_feature(
        store,
        stats,
        AdvancedRenderFeature::VirtualGeometry,
        "virtual_geometry",
        AdvancedProviderFeaturePaths {
            provider_present: "render.advanced_provider.virtual_geometry.report_provider_present",
            requested: "render.advanced_provider.virtual_geometry.requested",
            ready: "render.advanced_provider.virtual_geometry.ready",
            degraded: "render.advanced_provider.virtual_geometry.degraded",
            enabled: "render.advanced_provider.virtual_geometry.enabled",
            degradation_count: "render.advanced_provider.virtual_geometry.degradation_count",
            missing_capability_degradation_count:
                "render.advanced_provider.virtual_geometry.missing_capability_degradation_count",
            missing_provider_degradation_count:
                "render.advanced_provider.virtual_geometry.missing_provider_degradation_count",
        },
    );
    record_feature(
        store,
        stats,
        AdvancedRenderFeature::HybridGlobalIllumination,
        "hybrid_gi",
        AdvancedProviderFeaturePaths {
            provider_present: "render.advanced_provider.hybrid_gi.report_provider_present",
            requested: "render.advanced_provider.hybrid_gi.requested",
            ready: "render.advanced_provider.hybrid_gi.ready",
            degraded: "render.advanced_provider.hybrid_gi.degraded",
            enabled: "render.advanced_provider.hybrid_gi.enabled",
            degradation_count: "render.advanced_provider.hybrid_gi.degradation_count",
            missing_capability_degradation_count:
                "render.advanced_provider.hybrid_gi.missing_capability_degradation_count",
            missing_provider_degradation_count:
                "render.advanced_provider.hybrid_gi.missing_provider_degradation_count",
        },
    );
}

fn record_availability(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    record_bool(
        store,
        "render.advanced_provider.availability.virtual_geometry_provider_present",
        frame_index,
        stats
            .advanced_provider_availability
            .virtual_geometry_provider_id
            .is_some(),
        &[
            "render",
            "advanced_provider",
            "availability",
            "virtual_geometry",
        ],
    );
    record_bool(
        store,
        "render.advanced_provider.availability.hybrid_gi_provider_present",
        frame_index,
        stats
            .advanced_provider_availability
            .hybrid_gi_provider_id
            .is_some(),
        &["render", "advanced_provider", "availability", "hybrid_gi"],
    );
}

fn record_reports(store: &mut DiagnosticStore, stats: &RenderStats) {
    let frame_index = stats.submitted_frames;
    let reports = &stats.last_advanced_provider_reports;
    record_count(
        store,
        "render.advanced_provider.report_count",
        frame_index,
        reports.len(),
        &["render", "advanced_provider"],
    );
    record_count(
        store,
        "render.advanced_provider.requested_count",
        frame_index,
        reports.iter().filter(|report| report.requested).count(),
        &["render", "advanced_provider", "requested"],
    );
    record_count(
        store,
        "render.advanced_provider.ready_count",
        frame_index,
        reports
            .iter()
            .filter(|report| report.status == AdvancedProviderStatus::Ready)
            .count(),
        &["render", "advanced_provider", "ready"],
    );
    record_count(
        store,
        "render.advanced_provider.degraded_count",
        frame_index,
        reports
            .iter()
            .filter(|report| report.status == AdvancedProviderStatus::Degraded)
            .count(),
        &["render", "advanced_provider", "degraded"],
    );
    record_count(
        store,
        "render.advanced_provider.enabled_count",
        frame_index,
        reports.iter().filter(|report| report.enabled()).count(),
        &["render", "advanced_provider", "enabled"],
    );
    record_count(
        store,
        "render.advanced_provider.degradation_count",
        frame_index,
        degradation_count(reports),
        &["render", "advanced_provider", "degradation"],
    );
    record_count(
        store,
        "render.advanced_provider.missing_capability_degradation_count",
        frame_index,
        degradation_reason_count(
            reports,
            AdvancedRenderDegradationReason::BackendCapabilityMissing,
        ),
        &["render", "advanced_provider", "degradation", "capability"],
    );
    record_count(
        store,
        "render.advanced_provider.missing_provider_degradation_count",
        frame_index,
        degradation_reason_count(reports, AdvancedRenderDegradationReason::ProviderMissing),
        &["render", "advanced_provider", "degradation", "provider"],
    );
}

fn record_feature(
    store: &mut DiagnosticStore,
    stats: &RenderStats,
    feature: AdvancedRenderFeature,
    feature_tag: &'static str,
    paths: AdvancedProviderFeaturePaths,
) {
    let frame_index = stats.submitted_frames;
    let report = stats
        .last_advanced_provider_reports
        .iter()
        .find(|report| report.feature == feature);
    record_bool(
        store,
        paths.provider_present,
        frame_index,
        report.is_some_and(|report| report.provider_id.is_some()),
        &["render", "advanced_provider", feature_tag, "provider"],
    );
    record_bool(
        store,
        paths.requested,
        frame_index,
        report.is_some_and(|report| report.requested),
        &["render", "advanced_provider", feature_tag, "requested"],
    );
    record_bool(
        store,
        paths.ready,
        frame_index,
        report.is_some_and(|report| report.status == AdvancedProviderStatus::Ready),
        &["render", "advanced_provider", feature_tag, "ready"],
    );
    record_bool(
        store,
        paths.degraded,
        frame_index,
        report.is_some_and(|report| report.status == AdvancedProviderStatus::Degraded),
        &["render", "advanced_provider", feature_tag, "degraded"],
    );
    record_bool(
        store,
        paths.enabled,
        frame_index,
        report.is_some_and(AdvancedProviderReport::enabled),
        &["render", "advanced_provider", feature_tag, "enabled"],
    );
    record_count(
        store,
        paths.degradation_count,
        frame_index,
        report.map_or(0, |report| report.degradations.len()),
        &["render", "advanced_provider", feature_tag, "degradation"],
    );
    record_count(
        store,
        paths.missing_capability_degradation_count,
        frame_index,
        report.map_or(0, |report| {
            report
                .degradations
                .iter()
                .filter(|degradation| {
                    degradation.reason == AdvancedRenderDegradationReason::BackendCapabilityMissing
                })
                .count()
        }),
        &[
            "render",
            "advanced_provider",
            feature_tag,
            "degradation",
            "capability",
        ],
    );
    record_count(
        store,
        paths.missing_provider_degradation_count,
        frame_index,
        report.map_or(0, |report| {
            report
                .degradations
                .iter()
                .filter(|degradation| {
                    degradation.reason == AdvancedRenderDegradationReason::ProviderMissing
                })
                .count()
        }),
        &[
            "render",
            "advanced_provider",
            feature_tag,
            "degradation",
            "provider",
        ],
    );
}

fn degradation_count(reports: &[AdvancedProviderReport]) -> usize {
    reports.iter().map(|report| report.degradations.len()).sum()
}

fn degradation_reason_count(
    reports: &[AdvancedProviderReport],
    reason: AdvancedRenderDegradationReason,
) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.degradations)
        .filter(|degradation| degradation.reason == reason)
        .count()
}

struct AdvancedProviderFeaturePaths {
    provider_present: &'static str,
    requested: &'static str,
    ready: &'static str,
    degraded: &'static str,
    enabled: &'static str,
    degradation_count: &'static str,
    missing_capability_degradation_count: &'static str,
    missing_provider_degradation_count: &'static str,
}
