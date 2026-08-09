mod log;
mod submit;

use super::super::super::chrome_command_stream::build_chrome_command_stream;
use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::backbuffer::{can_region_repaint, repaint_backbuffer};
use super::diagnostics::{plan_present_for_diagnostics, record_chrome_command_stream_counters};
use super::lifecycle::resize_presenter;
use super::surface_io::current_window_size;
use super::SoftbufferHostPresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};
use log::write_verbose_present_log;
use submit::submit_presented_frame;

pub(in crate::ui::retained_host::host_contract) fn present(
    presenter: &mut SoftbufferHostPresenter,
    presentation: &HostWindowPresentationData,
    damage: Option<FrameRect>,
    invalidation: HostInvalidationDiagnostics,
) -> Result<HostRefreshDiagnostics, softbuffer::SoftBufferError> {
    zircon_runtime::profile_scope!("editor", "host_presenter", "present");
    record_current_ui_perf_counter(UiPerfCounter::SoftwareFallbackPresentCount, 1.0);
    let size = current_window_size(presenter.surface.window().as_ref());
    if presenter.size != size {
        resize_presenter(presenter, size)?;
    }

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
