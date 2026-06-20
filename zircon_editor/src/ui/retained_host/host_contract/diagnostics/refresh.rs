use std::time::Instant;

use super::invalidation::HostInvalidationDiagnostics;
use super::overlay::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HostRefreshDiagnostics {
    pub present_count: u64,
    pub full_paint_count: u64,
    pub region_paint_count: u64,
    pub painted_pixel_count: u64,
    pub slow_path_rebuild_count: u64,
    pub render_rebuild_count: u64,
    pub paint_only_request_count: u64,
    first_present_at: Option<Instant>,
    last_present_at: Option<Instant>,
}

impl Default for HostRefreshDiagnostics {
    fn default() -> Self {
        Self {
            present_count: 0,
            full_paint_count: 0,
            region_paint_count: 0,
            painted_pixel_count: 0,
            slow_path_rebuild_count: 0,
            render_rebuild_count: 0,
            paint_only_request_count: 0,
            first_present_at: None,
            last_present_at: None,
        }
    }
}

impl HostRefreshDiagnostics {
    pub(crate) fn record_present(
        &mut self,
        painted_pixels: u64,
        full_paint: bool,
        region_paint: bool,
    ) {
        let now = Instant::now();
        if self.first_present_at.is_none() {
            self.first_present_at = Some(now);
        }
        self.last_present_at = Some(now);
        self.present_count = self.present_count.saturating_add(1);
        if full_paint {
            self.full_paint_count = self.full_paint_count.saturating_add(1);
        }
        if region_paint {
            self.region_paint_count = self.region_paint_count.saturating_add(1);
        }
        self.painted_pixel_count = self.painted_pixel_count.saturating_add(painted_pixels);
    }

    pub(crate) fn with_invalidation_counts(
        mut self,
        slow_path_rebuild_count: u64,
        render_rebuild_count: u64,
        paint_only_request_count: u64,
    ) -> Self {
        self.slow_path_rebuild_count = slow_path_rebuild_count;
        self.render_rebuild_count = render_rebuild_count;
        self.paint_only_request_count = paint_only_request_count;
        self
    }

    pub(crate) fn with_invalidation_diagnostics(
        self,
        invalidation: HostInvalidationDiagnostics,
    ) -> Self {
        self.with_invalidation_counts(
            invalidation.slow_path_rebuild_count,
            invalidation.render_rebuild_count,
            invalidation.paint_only_request_count,
        )
    }

    pub(crate) fn fps(&self) -> Option<f32> {
        let start = self.first_present_at?;
        let end = self.last_present_at?;
        let seconds = end.duration_since(start).as_secs_f32();
        (seconds > 0.0).then_some(self.present_count as f32 / seconds)
    }

    pub(crate) fn overlay_text(&self) -> String {
        if self.present_count == 0 {
            return STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.to_string();
        }

        format!(
            "FPS {:.1} | present {} | full {} | region {} | pixels {} | slow {} | render {} | paint-only {}",
            self.fps().unwrap_or(0.0),
            self.present_count,
            self.full_paint_count,
            self.region_paint_count,
            self.painted_pixel_count,
            self.slow_path_rebuild_count,
            self.render_rebuild_count,
            self.paint_only_request_count,
        )
    }
}
