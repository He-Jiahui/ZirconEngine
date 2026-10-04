use std::fmt::Write as _;

use super::super::invalidation::HostInvalidationDiagnostics;
use super::super::overlay::STARTUP_REFRESH_DIAGNOSTICS_OVERLAY;
use super::model::HostRefreshDiagnostics;

pub(super) fn refresh_overlay_text(diagnostics: &HostRefreshDiagnostics) -> String {
    if diagnostics.present_count == 0 {
        return STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.to_string();
    }

    refresh_overlay_text_with_fps(diagnostics, diagnostics.fps().unwrap_or(0.0))
}

fn refresh_overlay_text_with_fps(diagnostics: &HostRefreshDiagnostics, fps: f32) -> String {
    refresh_overlay_text_with_counts(
        diagnostics,
        fps,
        diagnostics.slow_path_rebuild_count,
        diagnostics.render_rebuild_count,
        diagnostics.paint_only_request_count,
    )
}

pub(super) fn refresh_overlay_text_with_invalidation(
    diagnostics: &HostRefreshDiagnostics,
    invalidation: HostInvalidationDiagnostics,
) -> String {
    if diagnostics.present_count == 0 {
        return STARTUP_REFRESH_DIAGNOSTICS_OVERLAY.to_string();
    }

    refresh_overlay_text_with_counts(
        diagnostics,
        diagnostics.fps().unwrap_or(0.0),
        invalidation.slow_path_rebuild_count,
        invalidation.render_rebuild_count,
        invalidation.paint_only_request_count,
    )
}

fn refresh_overlay_text_with_counts(
    diagnostics: &HostRefreshDiagnostics,
    fps: f32,
    slow_path_rebuild_count: u64,
    render_rebuild_count: u64,
    paint_only_request_count: u64,
) -> String {
    let mut output = String::with_capacity(192);
    write!(&mut output, "FPS {fps:.1}").expect("writing to a String cannot fail");
    for (prefix, value) in [
        (" | present ", diagnostics.present_count),
        (" | full ", diagnostics.full_paint_count),
        (" | region ", diagnostics.region_paint_count),
        (" | pixels ", diagnostics.painted_pixel_count),
        (" | slow ", slow_path_rebuild_count),
        (" | render ", render_rebuild_count),
        (" | paint-only ", paint_only_request_count),
    ] {
        output.push_str(prefix);
        push_u64_decimal(&mut output, value);
    }
    output
}

fn push_u64_decimal(output: &mut String, mut value: u64) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    for digit in &digits[start..] {
        output.push(char::from(*digit));
    }
}

#[cfg(test)]
#[path = "tests/overlay_text_optimization_batch_fj_tests.rs"]
mod optimization_batch_fj_tests;
