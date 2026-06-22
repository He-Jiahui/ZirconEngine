use super::super::super::invalidation::HostInvalidationDiagnostics;
use super::super::overlay_text::refresh_overlay_text;
use super::super::timing::{record_present_timing, refresh_fps};
use super::HostRefreshDiagnostics;

impl HostRefreshDiagnostics {
    pub(crate) fn record_present(
        &mut self,
        painted_pixels: u64,
        full_paint: bool,
        region_paint: bool,
    ) {
        record_present_timing(self);
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
        refresh_fps(
            self.first_present_at,
            self.last_present_at,
            self.present_count,
        )
    }

    pub(crate) fn overlay_text(&self) -> String {
        refresh_overlay_text(self)
    }
}
