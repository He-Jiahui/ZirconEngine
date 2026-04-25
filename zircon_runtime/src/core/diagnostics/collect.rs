use crate::core::manager::{
    resolve_animation_manager, resolve_physics_manager, resolve_render_framework,
};
use crate::core::CoreHandle;

use super::{
    RuntimeAnimationDiagnostics, RuntimeDiagnosticsSnapshot, RuntimePhysicsDiagnostics,
    RuntimeRenderDiagnostics,
};

pub fn collect_runtime_diagnostics(core: &CoreHandle) -> RuntimeDiagnosticsSnapshot {
    RuntimeDiagnosticsSnapshot {
        render: collect_render_diagnostics(core),
        physics: collect_physics_diagnostics(core),
        animation: collect_animation_diagnostics(core),
    }
}

fn collect_render_diagnostics(core: &CoreHandle) -> RuntimeRenderDiagnostics {
    let render_framework = match resolve_render_framework(core) {
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
    let physics = match resolve_physics_manager(core) {
        Ok(physics) => physics,
        Err(error) => return RuntimePhysicsDiagnostics::unavailable(error.to_string()),
    };
    let settings = physics.settings();

    RuntimePhysicsDiagnostics {
        available: true,
        backend_name: Some(physics.backend_name()),
        backend_status: Some(physics.backend_status()),
        fixed_hz: Some(settings.fixed_hz),
        error: None,
    }
}

fn collect_animation_diagnostics(core: &CoreHandle) -> RuntimeAnimationDiagnostics {
    let animation = match resolve_animation_manager(core) {
        Ok(animation) => animation,
        Err(error) => return RuntimeAnimationDiagnostics::unavailable(error.to_string()),
    };

    RuntimeAnimationDiagnostics {
        available: true,
        playback_settings: Some(animation.playback_settings()),
        error: None,
    }
}
