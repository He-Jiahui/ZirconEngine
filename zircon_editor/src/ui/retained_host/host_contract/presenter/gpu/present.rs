use zircon_runtime::rhi::UiSurfacePresenter;

use super::super::super::chrome_command_stream::{
    build_chrome_command_stream, ui_surface_draw_list_from_stream, ChromeCommandStream,
};
use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::geometry::{damage_pixel_count, full_surface_pixels};
use super::stats::record_present_stats;
use super::GpuChromePresenter;
use crate::ui::retained_host::host_contract::presenter::error::HostPresenterResult;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(in crate::ui::retained_host::host_contract) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let stream_damage = damage.as_ref().filter(|_| self.surface_cache_initialized);
        let stream = build_chrome_command_stream(presentation, self.size, stream_damage, true);
        self.present_stream_with_damage_diagnostics(&stream, damage.as_ref(), invalidation)
    }

    pub(in crate::ui::retained_host::host_contract) fn present_stream(
        &mut self,
        stream: &ChromeCommandStream,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        self.present_stream_with_damage_diagnostics(stream, stream.damage(), invalidation)
    }

    fn present_stream_with_damage_diagnostics(
        &mut self,
        stream: &ChromeCommandStream,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let draw_list = ui_surface_draw_list_from_stream(stream);
        let stats = self.surface.present(&draw_list)?;
        let region_present = diagnostic_damage.is_some() || !stream.is_full_rebuild();
        record_present_stats(self, &stats, region_present);
        self.surface_cache_initialized = true;

        let painted_pixels = diagnostic_damage
            .map(|damage| damage_pixel_count(damage, stream.surface_size()))
            .unwrap_or_else(|| full_surface_pixels(stream.surface_size()));
        record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, painted_pixels as f64);
        if region_present {
            record_current_ui_perf_counter(UiPerfCounter::RegionPaintCount, 1.0);
        } else {
            record_current_ui_perf_counter(UiPerfCounter::FullPaintCount, 1.0);
        }
        self.diagnostics
            .record_present(painted_pixels, !region_present, region_present);
        Ok(self
            .diagnostics
            .clone()
            .with_invalidation_diagnostics(invalidation))
    }
}
