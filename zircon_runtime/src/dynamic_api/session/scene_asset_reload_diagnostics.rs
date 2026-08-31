use crate::core::runtime::diagnostics::DiagnosticStore;
use crate::core::CoreRuntime;
use crate::scene::{DynamicSceneAssetReloadFrameApplyReport, DynamicSceneAssetReloadSkipReason};

const SCENE_ASSET_RELOAD_EVENTS_DRAINED_DIAGNOSTIC: &str = "scene.asset_reload.events_drained";
const SCENE_ASSET_RELOAD_SCHEDULED_DIAGNOSTIC: &str = "scene.asset_reload.scheduled";
const SCENE_ASSET_RELOAD_SKIPPED_DIAGNOSTIC: &str = "scene.asset_reload.skipped";
const SCENE_ASSET_RELOAD_SKIPPED_REMOVED_DIAGNOSTIC: &str = "scene.asset_reload.skipped_removed";
const SCENE_ASSET_RELOAD_SKIPPED_RELOAD_FAILED_DIAGNOSTIC: &str =
    "scene.asset_reload.skipped_reload_failed";
const SCENE_ASSET_RELOAD_SKIPPED_MISSING_LOCATOR_DIAGNOSTIC: &str =
    "scene.asset_reload.skipped_missing_locator";
const SCENE_ASSET_RELOAD_SKIPPED_STALE_REVISION_DIAGNOSTIC: &str =
    "scene.asset_reload.skipped_stale_revision";
const SCENE_ASSET_RELOAD_SUPERSEDED_PENDING_DIAGNOSTIC: &str =
    "scene.asset_reload.superseded_pending";
const SCENE_ASSET_RELOAD_APPLIED_DIAGNOSTIC: &str = "scene.asset_reload.applied";
const SCENE_ASSET_RELOAD_FAILED_DIAGNOSTIC: &str = "scene.asset_reload.failed";
const SCENE_ASSET_RELOAD_STALE_DIAGNOSTIC: &str = "scene.asset_reload.stale";
const SCENE_ASSET_RELOAD_PENDING_DIAGNOSTIC: &str = "scene.asset_reload.pending";
const SCENE_ASSET_RELOAD_RECEIVER_DISCONNECTED_DIAGNOSTIC: &str =
    "scene.asset_reload.receiver_disconnected";
const SCENE_ASSET_RELOAD_DIAGNOSTIC_TAGS: [&str; 2] = ["scene", "asset_reload"];

pub(super) fn record_scene_asset_reload_frame_report(
    runtime: &CoreRuntime,
    report: &DynamicSceneAssetReloadFrameApplyReport,
) {
    let frame_index = runtime.real_time().frame_index();
    runtime.handle().update_diagnostic_store(|store| {
        record_count(
            store,
            SCENE_ASSET_RELOAD_EVENTS_DRAINED_DIAGNOSTIC,
            frame_index,
            report.events_drained(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SCHEDULED_DIAGNOSTIC,
            frame_index,
            report.scheduled_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SKIPPED_DIAGNOSTIC,
            frame_index,
            report.skipped_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SKIPPED_REMOVED_DIAGNOSTIC,
            frame_index,
            report.skipped_count_for(DynamicSceneAssetReloadSkipReason::Removed),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SKIPPED_RELOAD_FAILED_DIAGNOSTIC,
            frame_index,
            report.skipped_count_for(DynamicSceneAssetReloadSkipReason::ReloadFailed),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SKIPPED_MISSING_LOCATOR_DIAGNOSTIC,
            frame_index,
            report.skipped_count_for(DynamicSceneAssetReloadSkipReason::MissingLocator),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SKIPPED_STALE_REVISION_DIAGNOSTIC,
            frame_index,
            report.skipped_count_for(DynamicSceneAssetReloadSkipReason::StaleRevision),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_SUPERSEDED_PENDING_DIAGNOSTIC,
            frame_index,
            report.superseded_pending_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_APPLIED_DIAGNOSTIC,
            frame_index,
            report.applied_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_FAILED_DIAGNOSTIC,
            frame_index,
            report.failed_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_STALE_DIAGNOSTIC,
            frame_index,
            report.stale_count(),
        );
        record_count(
            store,
            SCENE_ASSET_RELOAD_PENDING_DIAGNOSTIC,
            frame_index,
            report.pending_count(),
        );
        record_bool(
            store,
            SCENE_ASSET_RELOAD_RECEIVER_DISCONNECTED_DIAGNOSTIC,
            frame_index,
            report.receiver_disconnected(),
        );
    });
}

fn record_count(store: &mut DiagnosticStore, path: &'static str, frame_index: u64, value: usize) {
    store.record_static(
        path,
        frame_index,
        value as f64,
        Some("count"),
        &SCENE_ASSET_RELOAD_DIAGNOSTIC_TAGS,
    );
}

fn record_bool(store: &mut DiagnosticStore, path: &'static str, frame_index: u64, value: bool) {
    store.record_static(
        path,
        frame_index,
        u8::from(value) as f64,
        Some("bool"),
        &SCENE_ASSET_RELOAD_DIAGNOSTIC_TAGS,
    );
}
