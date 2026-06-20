use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::super::super::chrome_command_stream::build_chrome_command_stream;
use super::super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::super::diagnostics::{HostInvalidationDiagnostics, HostRefreshDiagnostics};
use super::backbuffer::{can_region_repaint, repaint_backbuffer};
use super::diagnostics::{
    frame_summary, plan_present_for_diagnostics, presentation_summary,
    record_chrome_command_stream_counters,
};
use super::lifecycle::resize_presenter;
use super::surface_io::{copy_rgba_to_softbuffer, current_window_size, softbuffer_damage_rect};
use super::SoftbufferHostPresenter;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

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

    let frame = presenter
        .backbuffer
        .as_ref()
        .expect("presenter repaint path always creates a backbuffer");
    let window = presenter.surface.window().clone();
    let mut buffer = presenter.surface.buffer_mut()?;
    {
        zircon_runtime::profile_scope!("editor", "host_presenter", "copy_rgba_to_softbuffer");
        copy_rgba_to_softbuffer(frame, &mut *buffer, outcome.damage.as_ref(), size);
    }

    window.pre_present_notify();
    zircon_runtime::profile_scope!("editor", "host_presenter", "softbuffer_present");
    let result = if let Some(damage) = softbuffer_damage_rect(outcome.damage.as_ref(), size) {
        buffer.present_with_damage(&[damage])
    } else {
        buffer.present()
    };
    result?;
    Ok(presenter
        .diagnostics_snapshot()
        .with_invalidation_diagnostics(invalidation))
}

fn write_verbose_present_log(
    presenter: &mut SoftbufferHostPresenter,
    presentation: &HostWindowPresentationData,
    outcome: &super::backbuffer::RepaintOutcome,
    size: (u32, u32),
) {
    if !diagnostic_log_allows(DiagnosticLogLevel::Verbose) {
        return;
    }
    let summary = presentation_summary(presentation);
    if presenter.diagnostics.present_count > 8
        && presenter.last_logged_size == Some(size)
        && presenter.last_logged_presentation.as_deref() == Some(summary.as_str())
    {
        return;
    }
    write_diagnostic_log(
        "editor_host_presenter",
        format!(
            "present frame={} frame_size={}x{} damage={} painted_pixels={} full_paints={} region_paints={} total_painted_pixels={} {}",
            presenter.diagnostics.present_count,
            size.0,
            size.1,
            outcome
                .damage
                .as_ref()
                .map(frame_summary)
                .unwrap_or_else(|| "full".to_string()),
            outcome.painted_pixels,
            presenter.diagnostics.full_paint_count,
            presenter.diagnostics.region_paint_count,
            presenter.diagnostics.painted_pixel_count,
            summary
        ),
    );
    presenter.last_logged_size = Some(size);
    presenter.last_logged_presentation = Some(summary);
}
