use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

use super::super::pane_payload::{PanePayload, RuntimeDiagnosticsPanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let diagnostics = context
        .runtime_diagnostics
        .cloned()
        .unwrap_or_else(RuntimeDiagnosticsSnapshot::default);

    PanePayload::RuntimeDiagnosticsV1(RuntimeDiagnosticsPanePayload {
        summary: summary(&diagnostics),
        render_status: render_status(&diagnostics),
        physics_status: physics_status(&diagnostics),
        animation_status: animation_status(&diagnostics),
        detail_items: detail_items(&diagnostics),
    })
}

fn summary(diagnostics: &RuntimeDiagnosticsSnapshot) -> String {
    let available = [
        diagnostics.render.available,
        diagnostics.physics.available,
        diagnostics.animation.available,
    ]
    .into_iter()
    .filter(|available| *available)
    .count();
    format!("{available} runtime systems available")
}

fn render_status(diagnostics: &RuntimeDiagnosticsSnapshot) -> String {
    if !diagnostics.render.available {
        return format!(
            "Render: unavailable ({})",
            diagnostics
                .render
                .error
                .as_deref()
                .unwrap_or("render framework not resolved")
        );
    }

    let Some(stats) = diagnostics.render.stats.as_ref() else {
        return "Render: available (stats unavailable)".to_string();
    };
    let backend = if stats.capabilities.backend_name.is_empty() {
        "unknown"
    } else {
        stats.capabilities.backend_name.as_str()
    };
    format!(
        "Render: {backend} ({} viewports, {} frames)",
        stats.active_viewports, stats.submitted_frames
    )
}

fn physics_status(diagnostics: &RuntimeDiagnosticsSnapshot) -> String {
    if !diagnostics.physics.available {
        return format!(
            "Physics: unavailable ({})",
            diagnostics
                .physics
                .error
                .as_deref()
                .unwrap_or("physics manager not resolved")
        );
    }

    let backend = diagnostics
        .physics
        .backend_status
        .as_ref()
        .and_then(|status| status.active_backend.as_deref())
        .or(diagnostics.physics.backend_name.as_deref())
        .unwrap_or("unknown");
    let state = diagnostics
        .physics
        .backend_status
        .as_ref()
        .map(|status| format!("{:?}", status.state))
        .unwrap_or_else(|| "Unknown".to_string());
    match diagnostics.physics.fixed_hz {
        Some(fixed_hz) => format!("Physics: {backend} ({state}, {fixed_hz} Hz)"),
        None => format!("Physics: {backend} ({state})"),
    }
}

fn animation_status(diagnostics: &RuntimeDiagnosticsSnapshot) -> String {
    if !diagnostics.animation.available {
        return format!(
            "Animation: unavailable ({})",
            diagnostics
                .animation
                .error
                .as_deref()
                .unwrap_or("animation manager not resolved")
        );
    }

    let Some(settings) = diagnostics.animation.playback_settings.as_ref() else {
        return "Animation: available (settings unavailable)".to_string();
    };
    let enabled = if settings.enabled {
        "enabled"
    } else {
        "disabled"
    };
    let graphs = if settings.graphs {
        "graphs on"
    } else {
        "graphs off"
    };
    let state_machines = if settings.state_machines {
        "state machines on"
    } else {
        "state machines off"
    };
    format!("Animation: {enabled} ({graphs}, {state_machines})")
}

fn detail_items(diagnostics: &RuntimeDiagnosticsSnapshot) -> Vec<String> {
    let mut items = Vec::new();
    items.push(format!(
        "Virtual Geometry Debug: {}",
        if diagnostics.render.virtual_geometry_debug_available {
            "available"
        } else {
            "unavailable"
        }
    ));
    if let Some(stats) = diagnostics.render.stats.as_ref() {
        items.push(format!(
            "Hybrid GI active probes: {}",
            stats.last_hybrid_gi_active_probe_count
        ));
        items.push(format!(
            "Virtual Geometry visible clusters: {}",
            stats.last_virtual_geometry_visible_cluster_count
        ));
    }
    if let Some(error) = diagnostics.render.error.as_ref() {
        items.push(format!("Render error: {error}"));
    }
    if let Some(error) = diagnostics.physics.error.as_ref() {
        items.push(format!("Physics error: {error}"));
    }
    if let Some(error) = diagnostics.animation.error.as_ref() {
        items.push(format!("Animation error: {error}"));
    }
    items
}
