use crate::core::manager::{
    animation_manager_handle, render_framework_handle, resolve_manager_service,
};
use crate::core::CoreHandle;

use super::render_stats_store::record_render_stats_diagnostics;
use super::{
    RuntimeAnimationDiagnostics, RuntimeDiagnosticsSnapshot, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
};

pub fn collect_runtime_diagnostics(core: &CoreHandle) -> RuntimeDiagnosticsSnapshot {
    let render = collect_render_diagnostics(core);
    let physics = collect_physics_diagnostics(core);
    let animation = collect_animation_diagnostics(core);
    let store = collect_diagnostic_store_snapshot(core, &render, &physics, &animation);
    let profile = super::profiling::snapshot();

    RuntimeDiagnosticsSnapshot {
        render,
        physics,
        animation,
        store,
        profile,
    }
}

fn collect_render_diagnostics(core: &CoreHandle) -> RuntimeRenderDiagnostics {
    let render_framework = match render_framework_handle(core)
        .and_then(|handle| resolve_manager_service(core, handle))
    {
        Ok(render_framework) => render_framework,
        Err(error) => return RuntimeRenderDiagnostics::unavailable(error.to_string()),
    };

    let (stats, stats_error) = match render_framework.query_stats() {
        Ok(stats) => (Some(stats), None),
        Err(error) => (None, Some(error.to_string())),
    };
    let (virtual_geometry_debug_available, debug_error) =
        match render_framework.query_virtual_geometry_debug_snapshot() {
            Ok(snapshot) => (snapshot.is_some(), None),
            Err(error) => (false, Some(error.to_string())),
        };

    RuntimeRenderDiagnostics {
        available: true,
        stats,
        virtual_geometry_debug_available,
        error: stats_error.or(debug_error),
    }
}

fn collect_physics_diagnostics(core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    super::physics_collection::collect(core)
}

fn collect_animation_diagnostics(core: &CoreHandle) -> RuntimeAnimationDiagnostics {
    let animation = match animation_manager_handle(core)
        .and_then(|handle| resolve_manager_service(core, handle))
    {
        Ok(animation) => animation,
        Err(error) => return RuntimeAnimationDiagnostics::unavailable(error.to_string()),
    };

    RuntimeAnimationDiagnostics {
        available: true,
        playback_settings: Some(animation.playback_settings()),
        error: None,
    }
}

fn collect_diagnostic_store_snapshot(
    core: &CoreHandle,
    render: &RuntimeRenderDiagnostics,
    physics: &RuntimePhysicsDiagnostics,
    animation: &RuntimeAnimationDiagnostics,
) -> super::DiagnosticStoreSnapshot {
    let mut store = core.diagnostic_store();
    if let Some(stats) = &render.stats {
        store.record(
            "render.submitted_frames",
            stats.submitted_frames,
            stats.submitted_frames as f64,
            Some("frame"),
            ["render"],
        );
        store.record(
            "render.active_viewports",
            stats.submitted_frames,
            stats.active_viewports as f64,
            Some("count"),
            ["render"],
        );
        record_render_stats_diagnostics(&mut store, stats);
    }
    if let Some(fixed_hz) = physics.fixed_hz {
        store.record(
            "physics.fixed_hz",
            0,
            fixed_hz as f64,
            Some("hz"),
            ["physics"],
        );
    }
    if let Some(playback_settings) = &animation.playback_settings {
        store.record(
            "animation.enabled",
            0,
            u8::from(playback_settings.enabled) as f64,
            Some("bool"),
            ["animation"],
        );
        store.record(
            "animation.graphs_enabled",
            0,
            u8::from(playback_settings.graphs) as f64,
            Some("bool"),
            ["animation", "graph"],
        );
    }
    store.snapshot()
}
