use super::super::overlay::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;
use super::model::HostRefreshDiagnostics;

pub(super) fn refresh_overlay_text(diagnostics: &HostRefreshDiagnostics) -> String {
    if diagnostics.present_count == 0 {
        return STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.to_string();
    }

    format!(
        "FPS {:.1} | present {} | full {} | region {} | pixels {} | slow {} | render {} | paint-only {}",
        diagnostics.fps().unwrap_or(0.0),
        diagnostics.present_count,
        diagnostics.full_paint_count,
        diagnostics.region_paint_count,
        diagnostics.painted_pixel_count,
        diagnostics.slow_path_rebuild_count,
        diagnostics.render_rebuild_count,
        diagnostics.paint_only_request_count,
    )
}
