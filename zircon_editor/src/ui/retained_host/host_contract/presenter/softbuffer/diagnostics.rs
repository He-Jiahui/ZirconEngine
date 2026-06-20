use super::super::super::chrome_command_stream::ChromeCommandStream;
use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::super::super::paint_diagnostics::{
    debug_refresh_overlay_frame, presentation_top_bar_frame, union_diagnostic_frames,
};
use super::backbuffer::RepaintOutcome;
use super::surface_io::{damage_pixel_count, pixel_bounds};

#[cfg(feature = "profiling")]
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::host_contract) struct PlannedPresent {
    pub(in crate::ui::retained_host::host_contract) presentation: HostWindowPresentationData,
    pub(in crate::ui::retained_host::host_contract) damage: Option<FrameRect>,
    pub(in crate::ui::retained_host::host_contract) diagnostics: HostRefreshDiagnostics,
    pub(in crate::ui::retained_host::host_contract) overlay_text: String,
}

pub(in crate::ui::retained_host::host_contract) fn plan_present_for_diagnostics(
    current: &HostRefreshDiagnostics,
    can_region_repaint: bool,
    last_debug_overlay_text: Option<&str>,
    presentation: &HostWindowPresentationData,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
    size: (u32, u32),
) -> PlannedPresent {
    let mut damage = if can_region_repaint
        && damage
            .as_ref()
            .is_some_and(|damage| pixel_bounds(damage, size).is_some())
    {
        damage
    } else {
        None
    };

    // The overlay text includes the painted pixel total, and text width can expand
    // the region damage. Iterate until the text and expanded damage describe the
    // same present so the same-frame overlay matches the recorded diagnostics.
    for _ in 0..8 {
        let outcome = repaint_outcome_for_damage(damage.clone(), size);
        let mut diagnostics = current.clone();
        diagnostics.record_present(
            outcome.painted_pixels,
            outcome.full_paint,
            outcome.region_paint,
        );
        let overlay_text = diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation)
            .overlay_text();
        let expanded_damage = if outcome.region_paint {
            damage_with_debug_overlay(
                damage.clone(),
                last_debug_overlay_text,
                &overlay_text,
                size,
                presentation,
            )
        } else {
            None
        };
        if expanded_damage == damage {
            let mut presentation = presentation.clone();
            presentation.host_shell.debug_refresh_rate = overlay_text.clone().into();
            return PlannedPresent {
                presentation,
                damage: expanded_damage,
                diagnostics,
                overlay_text,
            };
        }
        damage = expanded_damage;
    }

    let outcome = repaint_outcome_for_damage(damage.clone(), size);
    let mut diagnostics = current.clone();
    diagnostics.record_present(
        outcome.painted_pixels,
        outcome.full_paint,
        outcome.region_paint,
    );
    let overlay_text = diagnostics
        .clone()
        .with_invalidation_diagnostics(invalidation)
        .overlay_text();
    let mut presentation = presentation.clone();
    presentation.host_shell.debug_refresh_rate = overlay_text.clone().into();
    PlannedPresent {
        presentation,
        damage,
        diagnostics,
        overlay_text,
    }
}

pub(in crate::ui::retained_host::host_contract) fn record_chrome_command_stream_counters(
    stream: &ChromeCommandStream,
) {
    #[cfg(feature = "profiling")]
    {
        let counter = if stream.is_full_rebuild() {
            UiPerfCounter::ChromeCommandFullRebuildCount
        } else {
            UiPerfCounter::ChromeCommandPatchCount
        };
        record_current_ui_perf_counter(counter, 1.0);
    }
    #[cfg(not(feature = "profiling"))]
    {
        let _ = stream;
    }
}

fn repaint_outcome_for_damage(damage: Option<FrameRect>, size: (u32, u32)) -> RepaintOutcome {
    if let Some(damage) = damage {
        return RepaintOutcome {
            painted_pixels: damage_pixel_count(&damage, size),
            damage: Some(damage),
            full_paint: false,
            region_paint: true,
        };
    }

    RepaintOutcome {
        damage: None,
        painted_pixels: (size.0 as u64) * (size.1 as u64),
        full_paint: true,
        region_paint: false,
    }
}

pub(in crate::ui::retained_host::host_contract) fn damage_with_debug_overlay(
    damage: Option<FrameRect>,
    last_debug_overlay_text: Option<&str>,
    debug_overlay_text: &str,
    size: (u32, u32),
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let damage = damage?;
    if last_debug_overlay_text == Some(debug_overlay_text) {
        return Some(damage);
    }
    let overlay = debug_refresh_overlay_frame(
        &presentation_top_bar_frame(size.0, size.1, presentation),
        debug_overlay_text,
    )?;
    Some(union_diagnostic_frames(&damage, &overlay))
}

pub(in crate::ui::retained_host::host_contract) fn presentation_summary(
    presentation: &HostWindowPresentationData,
) -> String {
    let layout = &presentation.host_layout;
    let scene = &presentation.host_scene_data;
    format!(
        "project_path={} viewport_label={} status={} center={} status_bar={} document={} viewport={} left={} right={} bottom={} page_tabs={} document_tabs={} left_tabs={} right_tabs={} bottom_tabs={} floating_windows={} document_pane_kind={} left_pane_kind={} right_pane_kind={} bottom_pane_kind={}",
        presentation.host_shell.project_path,
        presentation.host_shell.viewport_label,
        presentation.host_shell.status_secondary,
        frame_summary(&layout.center_band_frame),
        frame_summary(&layout.status_bar_frame),
        frame_summary(&layout.document_region_frame),
        frame_summary(&layout.viewport_content_frame),
        frame_summary(&layout.left_region_frame),
        frame_summary(&layout.right_region_frame),
        frame_summary(&layout.bottom_region_frame),
        scene.page_chrome.tabs.row_count(),
        scene.document_dock.tabs.row_count(),
        scene.left_dock.tabs.row_count(),
        scene.right_dock.tabs.row_count(),
        scene.bottom_dock.tabs.row_count(),
        scene.floating_layer.floating_windows.row_count(),
        scene.document_dock.pane.kind,
        scene.left_dock.pane.kind,
        scene.right_dock.pane.kind,
        scene.bottom_dock.pane.kind,
    )
}

pub(in crate::ui::retained_host::host_contract) fn frame_summary(frame: &FrameRect) -> String {
    format!(
        "{:.1},{:.1},{:.1},{:.1}",
        frame.x, frame.y, frame.width, frame.height
    )
}
