use crate::core::diagnostics::{
    profiling, record_render_stats_diagnostics, DiagnosticStore, DiagnosticStoreCurrentSnapshot,
    DiagnosticStoreSnapshot, RuntimeAnimationDiagnostics, RuntimeDiagnosticsSnapshot,
    RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
};
use crate::core::manager::{
    animation_manager_handle, render_framework_handle, resolve_manager_service,
};
use crate::core::CoreHandle;

pub fn collect_runtime_diagnostics(core: &CoreHandle) -> RuntimeDiagnosticsSnapshot {
    let (render, physics, animation) = collect_runtime_diagnostic_domains(core);
    let store = collect_diagnostic_store_snapshot(core, &render, &physics, &animation);
    let profile = profiling::snapshot();

    RuntimeDiagnosticsSnapshot {
        render,
        physics,
        animation,
        store,
        profile,
    }
}

pub(crate) fn collect_runtime_diagnostic_store(core: &CoreHandle) -> DiagnosticStoreSnapshot {
    let (render, physics, animation) = collect_runtime_diagnostic_domains(core);
    collect_diagnostic_store_snapshot(core, &render, &physics, &animation)
}

pub(crate) fn collect_runtime_diagnostic_current_store(
    core: &CoreHandle,
) -> DiagnosticStoreCurrentSnapshot {
    let (render, physics, animation) = collect_runtime_diagnostic_domains(core);
    core.update_diagnostic_store(|store| {
        record_diagnostic_domains(store, &render, &physics, &animation);
        store.current_snapshot()
    })
}

fn collect_runtime_diagnostic_domains(
    core: &CoreHandle,
) -> (
    RuntimeRenderDiagnostics,
    RuntimePhysicsDiagnostics,
    RuntimeAnimationDiagnostics,
) {
    (
        collect_render_diagnostics(core),
        collect_physics_diagnostics(core),
        collect_animation_diagnostics(core),
    )
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
        match render_framework.query_virtual_geometry_debug_snapshot_available() {
            Ok(available) => (available, None),
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
) -> DiagnosticStoreSnapshot {
    core.update_diagnostic_store(|store| {
        record_diagnostic_domains(store, render, physics, animation);
        store.snapshot()
    })
}

fn record_diagnostic_domains(
    store: &mut DiagnosticStore,
    render: &RuntimeRenderDiagnostics,
    physics: &RuntimePhysicsDiagnostics,
    animation: &RuntimeAnimationDiagnostics,
) {
    if let Some(stats) = &render.stats {
        store.record_static(
            "render.submitted_frames",
            stats.submitted_frames,
            stats.submitted_frames as f64,
            Some("frame"),
            &["render"],
        );
        store.record_static(
            "render.active_viewports",
            stats.submitted_frames,
            stats.active_viewports as f64,
            Some("count"),
            &["render"],
        );
        record_render_stats_diagnostics(store, stats);
    }
    if let Some(fixed_hz) = physics.fixed_hz {
        store.record_static(
            "physics.fixed_hz",
            0,
            fixed_hz as f64,
            Some("hz"),
            &["physics"],
        );
    }
    if let Some(playback_settings) = &animation.playback_settings {
        store.record_static(
            "animation.enabled",
            0,
            u8::from(playback_settings.enabled) as f64,
            Some("bool"),
            &["animation"],
        );
        store.record_static(
            "animation.graphs_enabled",
            0,
            u8::from(playback_settings.graphs) as f64,
            Some("bool"),
            &["animation", "graph"],
        );
    }
}

#[cfg(test)]
mod performance_tests {
    use super::{collect_diagnostic_store_snapshot, collect_runtime_diagnostic_store};
    use crate::core::diagnostics::{
        RuntimeAnimationDiagnostics, RuntimePhysicsDiagnostics, RuntimeRenderDiagnostics,
    };
    use crate::core::CoreRuntime;

    #[test]
    fn runtime_diagnostic_collection_uses_static_metadata_recording() {
        let source = include_str!("collect.rs");
        let end = source
            .find("mod performance_tests {")
            .expect("performance test module");

        assert!(!source[..end].contains("store.record("));
    }

    #[test]
    fn store_only_collection_does_not_clone_the_profile_timeline() {
        let source = include_str!("collect.rs");
        let start = source
            .find("pub(crate) fn collect_runtime_diagnostic_store")
            .expect("store-only collector");
        let end = source[start..]
            .find("fn collect_runtime_diagnostic_domains")
            .map(|offset| start + offset)
            .expect("collector helper after store-only collector");

        assert!(!source[start..end].contains("profiling::snapshot"));

        let runtime = CoreRuntime::new();
        let snapshot = collect_runtime_diagnostic_store(&runtime.handle());
        assert_eq!(snapshot, runtime.diagnostic_store_snapshot());
    }

    #[test]
    fn periodic_log_collection_uses_current_values_without_history() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        for frame_index in 0..64 {
            core.record_diagnostic(
                "runtime.retained_history",
                frame_index,
                frame_index as f64,
                Some("count"),
                ["runtime"],
            );
        }

        let full = collect_runtime_diagnostic_store(&core);
        let current = super::collect_runtime_diagnostic_current_store(&core);

        assert_eq!(full.series[0].history.len(), 64);
        assert_eq!(current.series.len(), full.series.len());
        assert_eq!(current.series[0].current, full.series[0].current.unwrap());
    }

    #[test]
    fn runtime_diagnostic_collection_preserves_authoritative_series_history() {
        let runtime = CoreRuntime::new();
        let core = runtime.handle();
        let render = RuntimeRenderDiagnostics::default();
        let animation = RuntimeAnimationDiagnostics::default();

        let first = RuntimePhysicsDiagnostics {
            available: true,
            fixed_hz: Some(60),
            ..RuntimePhysicsDiagnostics::default()
        };
        collect_diagnostic_store_snapshot(&core, &render, &first, &animation);

        let second = RuntimePhysicsDiagnostics {
            available: true,
            fixed_hz: Some(120),
            ..RuntimePhysicsDiagnostics::default()
        };
        let collected = collect_diagnostic_store_snapshot(&core, &render, &second, &animation);
        let authoritative = core.diagnostic_store_snapshot();
        assert_eq!(collected, authoritative);

        let series = collected
            .series
            .iter()
            .find(|series| series.path.as_str() == "physics.fixed_hz")
            .expect("physics fixed-rate diagnostics should be collected");
        assert_eq!(
            series
                .history
                .iter()
                .map(|measurement| measurement.value)
                .collect::<Vec<_>>(),
            [60.0, 120.0]
        );
        let smoothed = series.smoothed.expect("EMA should be available");
        assert!((smoothed - 66.0).abs() <= 1.0e-9);
        assert_eq!(series.min, Some(60.0));
        assert_eq!(series.max, Some(120.0));
    }
}
