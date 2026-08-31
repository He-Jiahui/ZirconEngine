mod log;
mod submit;

use super::super::super::chrome_command_stream::build_chrome_command_stream;
use super::super::super::data::{
    FrameRect, HostPresentationGenerationCursor, HostWindowPresentationData,
};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::super::snapshot::paint_host_presentation_snapshot;
use super::backbuffer::{
    acquire_native_resize_snapshot, can_region_repaint, repaint_backbuffer,
    NativeResizeSnapshotAcquisition,
};
use super::diagnostics::{plan_present_for_diagnostics, record_chrome_command_stream_counters};
use super::lifecycle::resize_presenter;
use super::surface_io::{copy_scaled_rgba_to_softbuffer, current_window_size};
use super::SoftbufferHostPresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use log::write_verbose_present_log;
use submit::submit_presented_frame;

pub(in crate::ui::retained_host::host_contract) fn present(
    presenter: &mut SoftbufferHostPresenter,
    presentation: &HostWindowPresentationData,
    _presentation_cursor: HostPresentationGenerationCursor,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
) -> Result<HostRefreshDiagnostics, softbuffer::SoftBufferError> {
    zircon_runtime::profile_scope!("editor", "host_presenter", "present");
    record_current_ui_perf_counter(UiPerfCounter::SoftwareFallbackPresentCount, 1.0);
    let size = current_window_size(presenter.surface.window().as_ref());
    if presenter.size != size {
        resize_presenter(presenter, size)?;
    }
    presenter.native_resize_snapshot = None;

    let planned = plan_present_for_diagnostics(
        &presenter.diagnostics,
        can_region_repaint(presenter),
        presenter.last_debug_overlay_text.as_deref(),
        presentation,
        damage,
        invalidation,
        size,
    );
    let stream =
        build_chrome_command_stream(&planned.presentation, size, planned.damage.as_ref(), true);
    record_chrome_command_stream_counters(&stream);
    let outcome = {
        zircon_runtime::profile_scope!("editor", "host_presenter", "repaint_backbuffer");
        repaint_backbuffer(presenter, &stream, size)
    };
    record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, outcome.painted_pixels as f64);
    record_current_ui_perf_counter(
        UiPerfCounter::PresentedSurfacePixels,
        u64::from(size.0).saturating_mul(u64::from(size.1)) as f64,
    );
    if outcome.full_paint {
        record_current_ui_perf_counter(UiPerfCounter::FullPaintCount, 1.0);
    }
    if outcome.region_paint {
        record_current_ui_perf_counter(UiPerfCounter::RegionPaintCount, 1.0);
    }
    debug_assert_eq!(
        planned.diagnostics.painted_pixel_count,
        presenter
            .diagnostics
            .painted_pixel_count
            .saturating_add(outcome.painted_pixels)
    );
    presenter.last_debug_overlay_text = Some(planned.overlay_text);
    presenter.diagnostics = planned.diagnostics;
    write_verbose_present_log(presenter, &planned.presentation, &outcome, size);
    submit_presented_frame(presenter, outcome.damage.as_ref(), size)?;
    Ok(presenter
        .diagnostics_snapshot()
        .with_invalidation_diagnostics(invalidation))
}

pub(in crate::ui::retained_host::host_contract) fn present_during_native_resize(
    presenter: &mut SoftbufferHostPresenter,
    presentation: &HostWindowPresentationData,
    _presentation_cursor: HostPresentationGenerationCursor,
    invalidation: HostInvalidationDiagnostics,
) -> Result<HostRefreshDiagnostics, softbuffer::SoftBufferError> {
    zircon_runtime::profile_scope!("editor", "host_presenter", "present_during_native_resize");
    record_current_ui_perf_counter(UiPerfCounter::SoftwareFallbackPresentCount, 1.0);
    let size = current_window_size(presenter.surface.window().as_ref());
    if presenter.size != size {
        resize_presenter(presenter, size)?;
    }

    let acquisition = acquire_native_resize_snapshot(
        &mut presenter.native_resize_snapshot,
        &mut presenter.backbuffer,
        || paint_host_presentation_snapshot(size.0, size.1, presentation),
    );
    let acquisition_counter = match acquisition {
        NativeResizeSnapshotAcquisition::Reused => {
            "ui.window_resize.softbuffer_snapshot_reuse_count"
        }
        NativeResizeSnapshotAcquisition::CapturedBackbuffer => {
            "ui.window_resize.softbuffer_snapshot_capture_count"
        }
        NativeResizeSnapshotAcquisition::BuiltFallback => {
            "ui.window_resize.softbuffer_snapshot_build_count"
        }
    };
    zircon_runtime::profile_counter!("editor", acquisition_counter, 1_u8);
    if acquisition == NativeResizeSnapshotAcquisition::CapturedBackbuffer {
        zircon_runtime::profile_counter!(
            "editor",
            "ui.window_resize.softbuffer_same_size_snapshot_capture_count",
            1_u8
        );
    }

    let presented_pixels = u64::from(size.0).saturating_mul(u64::from(size.1));
    let window = presenter.surface.window().clone();
    let mut buffer = presenter.surface.buffer_mut()?;
    {
        zircon_runtime::profile_scope!("editor", "host_presenter", "scale_native_resize_snapshot");
        copy_scaled_rgba_to_softbuffer(
            presenter
                .native_resize_snapshot
                .as_ref()
                .expect("native resize snapshot is initialized above"),
            &mut *buffer,
            size,
        );
    }
    zircon_runtime::profile_counter!(
        "editor",
        "ui.window_resize.softbuffer_snapshot_scale_count",
        1_u8
    );
    record_current_ui_perf_counter(UiPerfCounter::PaintedPixels, presented_pixels as f64);
    record_current_ui_perf_counter(
        UiPerfCounter::PresentedSurfacePixels,
        presented_pixels as f64,
    );
    presenter
        .diagnostics
        .record_present(presented_pixels, false, false);

    window.pre_present_notify();
    zircon_runtime::profile_scope!("editor", "host_presenter", "softbuffer_present");
    buffer.present()?;
    Ok(presenter
        .diagnostics_snapshot()
        .with_invalidation_diagnostics(invalidation))
}
