use std::time::Instant;

use super::model::HostRefreshDiagnostics;

pub(super) fn record_present_timing(diagnostics: &mut HostRefreshDiagnostics) {
    let now = Instant::now();
    if diagnostics.first_present_at.is_none() {
        diagnostics.first_present_at = Some(now);
    }
    diagnostics.last_present_at = Some(now);
}

pub(super) fn refresh_fps(
    first_present_at: Option<Instant>,
    last_present_at: Option<Instant>,
    present_count: u64,
) -> Option<f32> {
    let start = first_present_at?;
    let end = last_present_at?;
    let seconds = end.duration_since(start).as_secs_f32();
    (seconds > 0.0).then_some(present_count as f32 / seconds)
}
