use zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot;

use crate::ui::workbench::debug_reflector::{
    EditorUiDebugReflectorModel, EditorUiDebugReflectorOverlayState,
};

use super::super::pane_payload::{PanePayload, RuntimeDiagnosticsPanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let default_diagnostics;
    let diagnostics = match context.runtime_diagnostics {
        Some(diagnostics) => diagnostics,
        None => {
            default_diagnostics = RuntimeDiagnosticsSnapshot::default();
            &default_diagnostics
        }
    };
    let active_ui_debug_snapshot = context.active_ui_debug_snapshot;
    let (reflector, overlay_primitives) = active_ui_debug_snapshot
        .map(|snapshot| {
            (
                EditorUiDebugReflectorModel::from_snapshot(snapshot)
                    .with_schedule_sections(snapshot),
                EditorUiDebugReflectorOverlayState::default().primitives_from_snapshot(snapshot),
            )
        })
        .unwrap_or_else(|| (EditorUiDebugReflectorModel::no_active_surface(), Vec::new()));
    let reflector_sections = reflector.section_display_lines();
    let reflector_nodes = reflector.nodes.into_iter().map(|node| node.label).collect();

    PanePayload::RuntimeDiagnosticsV1(RuntimeDiagnosticsPanePayload {
        summary: summary(diagnostics),
        render_status: render_status(diagnostics),
        physics_status: physics_status(diagnostics),
        animation_status: animation_status(diagnostics),
        detail_items: detail_items(diagnostics),
        ui_debug_reflector_summary: reflector.summary.title,
        ui_debug_reflector_nodes: reflector_nodes,
        ui_debug_reflector_details: reflector.details,
        ui_debug_reflector_sections: reflector_sections,
        ui_debug_reflector_export_status: reflector.summary.export_status,
        ui_debug_reflector_overlay_primitives: overlay_primitives,
        ui_debug_reflector_has_active_snapshot: active_ui_debug_snapshot.is_some(),
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
        .map(|status| physics_state_display(Some(&status.state)))
        .unwrap_or_else(|| physics_state_display(None));
    match diagnostics.physics.fixed_hz {
        Some(fixed_hz) => format!("Physics: {backend} ({state}, {fixed_hz} Hz)"),
        None => format!("Physics: {backend} ({state})"),
    }
}

fn physics_state_display(state: Option<&str>) -> String {
    let state = state.map(str::trim).filter(|state| !state.is_empty());
    let Some(state) = state else {
        return "Unknown".to_string();
    };
    let mut characters = state.chars();
    let first = characters
        .next()
        .expect("a non-empty trimmed physics state has a first character");
    format!("{}{}", first.to_uppercase(), characters.as_str())
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
    let mut items = Vec::with_capacity(detail_item_capacity(diagnostics));
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
        if let Some(settings) = stats.last_hybrid_gi_resolved_settings {
            items.push(format!(
                "Hybrid GI effective: profile={}, mode={}, quality={}",
                settings.profile.label(),
                settings.mode.label(),
                settings.quality.label()
            ));
            items.push(format!(
                "Hybrid GI budgets: trace={}, cards={}, voxels={}",
                settings.trace_budget, settings.card_budget, settings.voxel_budget
            ));
            items.push(format!(
                "Hybrid GI fallback: {}",
                settings
                    .fallback_reason
                    .map(|reason| reason.label())
                    .unwrap_or("none")
            ));
        } else {
            items.push("Hybrid GI effective: unavailable".to_string());
        }
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
    if diagnostics.profile.feature_enabled {
        let state = if diagnostics.profile.active {
            "active"
        } else {
            "inactive"
        };
        items.push(format!(
            "Profiling: {state} ({} frames, {} spans, {} counters)",
            diagnostics.profile.frames.len(),
            diagnostics.profile.spans.len(),
            diagnostics.profile.counters.len()
        ));
        let over_budget = diagnostics
            .profile
            .frames
            .iter()
            .filter(|frame| frame.over_budget)
            .count();
        items.push(format!("Profiling over-budget frames: {over_budget}"));
    }
    items
}

/// Detail projection emits one base line, a bounded render-stat block, up to one line per
/// subsystem error, and two profiling lines. Count those predicates before formatting so the
/// retained pane vector does not grow geometrically on a dense diagnostic snapshot.
fn detail_item_capacity(diagnostics: &RuntimeDiagnosticsSnapshot) -> usize {
    let mut capacity = 1usize;
    if let Some(stats) = diagnostics.render.stats.as_ref() {
        capacity = capacity.saturating_add(2);
        capacity = capacity.saturating_add(if stats.last_hybrid_gi_resolved_settings.is_some() {
            3
        } else {
            1
        });
    }
    capacity = capacity
        .saturating_add(usize::from(diagnostics.render.error.is_some()))
        .saturating_add(usize::from(diagnostics.physics.error.is_some()))
        .saturating_add(usize::from(diagnostics.animation.error.is_some()));
    if diagnostics.profile.feature_enabled {
        capacity = capacity.saturating_add(2);
    }
    capacity
}

#[cfg(test)]
#[path = "tests/runtime_diagnostics.rs"]
mod tests;
