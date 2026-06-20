use zircon_runtime::rhi::{UiSurfacePresentStats, UiSurfacePresenter};

use super::super::chrome_command_stream::{
    build_chrome_command_stream, ui_surface_draw_list_from_stream, ChromeCommandStream,
};
use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::error::HostPresenterResult;
use super::host_chrome_presenter::HostChromePresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::host_contract) struct GpuChromePresenter<P: UiSurfacePresenter> {
    surface: P,
    size: (u32, u32),
    diagnostics: HostRefreshDiagnostics,
    last_upload_bytes: u64,
    last_draw_calls: u64,
    surface_cache_initialized: bool,
}

impl<P: UiSurfacePresenter> HostChromePresenter for GpuChromePresenter<P> {
    fn resize(&mut self, size: (u32, u32)) -> HostPresenterResult<()> {
        GpuChromePresenter::resize(self, size)
    }

    fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let stream_damage = damage.as_ref().filter(|_| self.surface_cache_initialized);
        let stream = build_chrome_command_stream(presentation, self.size, stream_damage, true);
        self.present_stream_with_damage_diagnostics(&stream, damage.as_ref(), invalidation)
    }

    fn diagnostics_snapshot(&self) -> HostRefreshDiagnostics {
        GpuChromePresenter::diagnostics_snapshot(self)
    }
}

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(in crate::ui::retained_host::host_contract) fn new(surface: P, size: (u32, u32)) -> Self {
        Self {
            surface,
            size: clamp_size(size),
            diagnostics: HostRefreshDiagnostics::default(),
            last_upload_bytes: 0,
            last_draw_calls: 0,
            surface_cache_initialized: false,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn resize(
        &mut self,
        size: (u32, u32),
    ) -> HostPresenterResult<()> {
        let size = clamp_size(size);
        self.surface.resize(size.0, size.1)?;
        self.size = size;
        self.surface_cache_initialized = false;
        Ok(())
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
        self.record_present_stats(&stats, region_present);
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

    pub(in crate::ui::retained_host::host_contract) fn diagnostics_snapshot(
        &self,
    ) -> HostRefreshDiagnostics {
        self.diagnostics.clone()
    }

    pub(in crate::ui::retained_host::host_contract) fn last_upload_bytes(&self) -> u64 {
        self.last_upload_bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn last_draw_calls(&self) -> u64 {
        self.last_draw_calls
    }

    fn record_present_stats(&mut self, stats: &UiSurfacePresentStats, region_present: bool) {
        self.last_upload_bytes = stats.image_upload_bytes;
        self.last_draw_calls = stats.draw_calls;
        record_current_ui_perf_counter(
            UiPerfCounter::GpuUploadBytes,
            stats.image_upload_bytes as f64,
        );
        record_current_ui_perf_counter(UiPerfCounter::GpuDrawCalls, stats.draw_calls as f64);
        record_current_ui_perf_counter(
            UiPerfCounter::GpuVisibleCommands,
            stats.visible_command_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuVisibleDrawItems,
            stats.visible_draw_item_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuBatchLayers,
            stats.batch_layer_count as f64,
        );
        record_current_ui_perf_counter(
            UiPerfCounter::GpuBatchDependencies,
            stats.batch_dependency_count as f64,
        );
        if region_present {
            record_current_ui_perf_counter(UiPerfCounter::ChromeCommandPatchCount, 1.0);
        } else {
            record_current_ui_perf_counter(UiPerfCounter::ChromeCommandFullRebuildCount, 1.0);
        }
    }
}

fn full_surface_pixels(size: (u32, u32)) -> u64 {
    u64::from(size.0.max(1)) * u64::from(size.1.max(1))
}

fn damage_pixel_count(frame: &super::super::data::FrameRect, size: (u32, u32)) -> u64 {
    let x0 = frame.x.floor().max(0.0).min(size.0.max(1) as f32) as u32;
    let y0 = frame.y.floor().max(0.0).min(size.1.max(1) as f32) as u32;
    let x1 = (frame.x + frame.width)
        .ceil()
        .max(0.0)
        .min(size.0.max(1) as f32) as u32;
    let y1 = (frame.y + frame.height)
        .ceil()
        .max(0.0)
        .min(size.1.max(1) as f32) as u32;
    u64::from(x1.saturating_sub(x0)) * u64::from(y1.saturating_sub(y0))
}

fn clamp_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

#[cfg(test)]
mod tests;
