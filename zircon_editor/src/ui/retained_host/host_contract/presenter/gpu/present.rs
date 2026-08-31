use zircon_runtime::rhi::{UiSurfaceDrawList, UiSurfacePresentStats, UiSurfacePresenter};

use super::super::super::chrome_command_stream::{
    build_chrome_command_stream_with_residency,
    ui_surface_draw_list_from_owned_stream_with_generation_and_residency,
    ui_surface_draw_list_from_stream_with_residency, ChromeCommandStream,
};
use super::super::super::data::{
    FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData,
};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::geometry::{damage_pixel_count, full_surface_pixels};
use super::stats::record_present_stats;
use super::GpuChromePresenter;
use crate::ui::retained_host::host_contract::presenter::error::{
    HostPresenterError, HostPresenterResult,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

impl<P: UiSurfacePresenter> GpuChromePresenter<P> {
    pub(in crate::ui::retained_host::host_contract) fn present(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        damage: Option<FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let _ = presentation_cursor;
        self.native_resize_draw_list = None;
        self.native_resize_projection_size = self.size;
        let stream_damage = damage.as_ref().filter(|_| self.surface_cache_initialized);
        let stream = build_chrome_command_stream_with_residency(
            presentation,
            self.size,
            stream_damage,
            true,
            |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            },
        );
        let submitted_damage = stream.damage().cloned();
        let region_present = submitted_damage.is_some();
        let surface_size = stream.surface_size();
        // A damage stream contains only commands for the damaged region. It is not a complete
        // projection and must never share the full-projection cache key across presents.
        let draw_list = ui_surface_draw_list_from_owned_stream_with_residency(
            stream,
            |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            },
        );
        self.present_draw_list_with_damage_diagnostics(
            draw_list,
            surface_size,
            region_present,
            submitted_damage.as_ref(),
            invalidation,
        )
    }

    pub(in crate::ui::retained_host::host_contract) fn present_during_native_resize(
        &mut self,
        presentation: &HostWindowPresentationData,
        presentation_cursor: HostPresentationGenerationCursor,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let _ = presentation_cursor;
        let reused_snapshot = self.native_resize_draw_list.is_some();
        if !reused_snapshot {
            self.native_resize_generation = self.native_resize_generation.saturating_add(1);
            let stream = build_chrome_command_stream_with_residency(
                presentation,
                self.native_resize_projection_size,
                None,
                true,
                |resource_key, generation| {
                    self.surface
                        .is_image_resource_resident(resource_key, generation)
                },
            );
            let mut draw_list =
                ui_surface_draw_list_from_owned_stream_with_generation_and_residency(
                    stream,
                    self.native_resize_generation,
                    |resource_key, generation| {
                        self.surface
                            .is_image_resource_resident(resource_key, generation)
                    },
                );
            draw_list.retarget_surface_size_preserving_projection(self.size);
            self.native_resize_draw_list = Some(draw_list);
            #[cfg(test)]
            {
                self.native_resize_snapshot_build_count =
                    self.native_resize_snapshot_build_count.saturating_add(1);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.command_snapshot_build_count",
                1_u8
            );
        } else {
            #[cfg(test)]
            {
                self.native_resize_snapshot_reuse_count =
                    self.native_resize_snapshot_reuse_count.saturating_add(1);
            }
            zircon_runtime::profile_counter!(
                "editor",
                "ui.window_resize.command_snapshot_reuse_count",
                1_u8
            );
        }

        let mut draw_list = self
            .native_resize_draw_list
            .take()
            .expect("native resize draw list is initialized above");
        draw_list.retarget_surface_size_preserving_projection(self.size);
        draw_list.damage = None;
        let stats = self.surface.present(&draw_list);
        self.native_resize_draw_list = Some(draw_list);
        let stats = require_submitted_present(stats?)?;
        let diagnostics = self.finish_present(stats, self.size, false, None, invalidation)?;
        // The resize projection is a frozen transaction snapshot, not the ordinary retained
        // surface baseline required by a later region-only command stream.
        self.surface_cache_initialized = false;
        Ok(diagnostics)
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
        let draw_list =
            ui_surface_draw_list_from_stream_with_residency(stream, |resource_key, generation| {
                self.surface
                    .is_image_resource_resident(resource_key, generation)
            });
        let region_present = diagnostic_damage.is_some() || !stream.is_full_rebuild();
        self.present_draw_list_with_damage_diagnostics(
            draw_list,
            stream.surface_size(),
            region_present,
            diagnostic_damage,
            invalidation,
        )
    }

    fn present_draw_list_with_damage_diagnostics(
        &mut self,
        draw_list: UiSurfaceDrawList,
        surface_size: (u32, u32),
        region_present: bool,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        let stats = require_submitted_present(self.surface.present_owned(draw_list)?)?;
        self.finish_present(
            stats,
            surface_size,
            region_present,
            diagnostic_damage,
            invalidation,
        )
    }

    fn finish_present(
        &mut self,
        stats: UiSurfacePresentStats,
        surface_size: (u32, u32),
        region_present: bool,
        diagnostic_damage: Option<&FrameRect>,
        invalidation: HostInvalidationDiagnostics,
    ) -> HostPresenterResult<HostRefreshDiagnostics> {
        record_present_stats(self, &stats, region_present);
        self.surface_cache_initialized = true;

        let painted_pixels = diagnostic_damage
            .map(|damage| damage_pixel_count(damage, surface_size))
            .unwrap_or_else(|| full_surface_pixels(surface_size));
        record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, painted_pixels as f64);
        record_current_ui_perf_counter(
            UiPerfCounter::PresentedSurfacePixels,
            full_surface_pixels(surface_size) as f64,
        );
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

fn require_submitted_present(
    stats: UiSurfacePresentStats,
) -> HostPresenterResult<UiSurfacePresentStats> {
    if stats.outcome.is_submitted() {
        Ok(stats)
    } else {
        Err(HostPresenterError::RetryableSurfacePresent)
    }
}
