use super::super::super::paint_theme::HostTextSmoothing;

const FALLBACK_TEXT_ORIGIN_PX: f32 = 0.0;

pub(super) fn retained_text_origin_for_smoothing(value: f32, smoothing: HostTextSmoothing) -> f32 {
    let value = finite_text_origin(value);
    match smoothing {
        HostTextSmoothing::Grayscale => value.round(),
        HostTextSmoothing::Subpixel => value,
    }
}

fn finite_text_origin(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        FALLBACK_TEXT_ORIGIN_PX
    }
}

#[cfg(test)]
#[path = "tests/placement.rs"]
mod tests;
