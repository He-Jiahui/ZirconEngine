from __future__ import annotations


RUNTIME_DIAGNOSTICS_ANCHORS = (
    (
        "zircon_runtime_interface/src/profiling.rs",
        "pub struct RuntimeDiagnosticsSnapshot",
    ),
    (
        "zircon_runtime_interface/src/profiling.rs",
        "pub runtime_diagnostics: Option<RuntimeDiagnosticsSnapshot>",
    ),
    (
        "zircon_runtime_interface/src/profiling.rs",
        "pub struct RuntimeDiagnosticSeriesSnapshot",
    ),
    (
        "zircon_runtime_interface/src/profiling.rs",
        "pub struct RuntimeSceneAssetReloadDiagnostics",
    ),
    (
        "zircon_runtime_interface/src/lib.rs",
        "RuntimeSceneAssetReloadDiagnostics",
    ),
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "ProfileControlCommand::RuntimeDiagnosticsSnapshot",
    ),
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "runtime_diagnostics_response(session)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/diagnostics.rs",
        "pub(super) fn runtime_diagnostics_response",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/diagnostics.rs",
        "collect_runtime_diagnostics(&session.runtime.handle())",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/diagnostics.rs",
        "session.scene_asset_reload_queue.is_some()",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/diagnostics.rs",
        "session.last_scene_asset_reload_report.as_ref()",
    ),
    (
        "zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs",
        "runtime diagnostics snapshot requires dynamic session",
    ),
    (
        "zircon_runtime/src/dynamic_api/tests/profile_control.rs",
        "runtime_diagnostics_snapshot_returns_store_and_scene_reload_report",
    ),
    (
        "zircon_runtime_interface/src/tests/contracts.rs",
        "runtime_diagnostics_snapshot",
    ),
    (
        "docs/zircon_runtime/dynamic_api/session.md",
        "Runtime Diagnostics Profile-Control Snapshot",
    ),
)


SCENE_ASSET_RELOAD_DIAGNOSTIC_PATH_ANCHORS = (
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "queue.tick_into_level(self.runtime.handle().scheduler(), &self.level)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "record_scene_asset_reload_frame_report(&self.runtime, &report)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session.rs",
        "self.last_scene_asset_reload_report = Some(report)",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "pub(super) fn record_scene_asset_reload_frame_report",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.events_drained",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.scheduled",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.skipped",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.skipped_removed",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.skipped_reload_failed",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.skipped_missing_locator",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.skipped_stale_revision",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.superseded_pending",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.applied",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.failed",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.stale",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.pending",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "scene.asset_reload.receiver_disconnected",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        "DynamicSceneAssetReloadSkipReason::MissingLocator",
    ),
    (
        "zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs",
        '["scene", "asset_reload"]',
    ),
    (
        "docs/zircon_runtime/dynamic_api/session.md",
        "scene.asset_reload.*",
    ),
    (
        "docs/zircon_runtime/scene/dynamic_scene.md",
        "Dynamic runtime session scene-asset reload diagnostics",
    ),
)
