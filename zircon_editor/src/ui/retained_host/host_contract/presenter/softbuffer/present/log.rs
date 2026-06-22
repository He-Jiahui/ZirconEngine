use zircon_runtime::diagnostic_log::{
    diagnostic_log_allows, write_diagnostic_log, DiagnosticLogLevel,
};

use super::super::super::super::data::HostWindowPresentationData;
use super::super::backbuffer::RepaintOutcome;
use super::super::diagnostics::{frame_summary, presentation_summary};
use super::super::SoftbufferHostPresenter;

pub(super) fn write_verbose_present_log(
    presenter: &mut SoftbufferHostPresenter,
    presentation: &HostWindowPresentationData,
    outcome: &RepaintOutcome,
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
