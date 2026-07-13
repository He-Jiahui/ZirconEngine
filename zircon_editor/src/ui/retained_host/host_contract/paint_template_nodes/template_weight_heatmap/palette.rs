const STOPS: [(f32, [u8; 4]); 6] = [
    (0.0, [14, 25, 99, 255]),
    (0.24, [0, 91, 214, 255]),
    (0.44, [0, 196, 198, 255]),
    (0.64, [82, 211, 79, 255]),
    (0.82, [244, 213, 42, 255]),
    (1.0, [235, 51, 36, 255]),
];

pub(super) const OUTER_SURFACE: [u8; 4] = [17, 20, 24, 255];
pub(super) const OUTER_BORDER: [u8; 4] = [49, 55, 62, 255];
pub(super) const SOURCE_MARKER: [u8; 4] = [232, 237, 240, 255];
pub(super) const SELECTED_SOURCE: [u8; 4] = [24, 187, 214, 255];
pub(super) const LEGEND_TEXT: [u8; 4] = [174, 181, 187, 255];

pub(super) fn heat_color(value: f32) -> [u8; 4] {
    let value = value.clamp(0.0, 1.0);
    for pair in STOPS.windows(2) {
        let (start_value, start_color) = pair[0];
        let (end_value, end_color) = pair[1];
        if value <= end_value {
            let factor = ((value - start_value) / (end_value - start_value)).clamp(0.0, 1.0);
            return interpolate(start_color, end_color, factor);
        }
    }
    STOPS[STOPS.len() - 1].1
}

fn interpolate(start: [u8; 4], end: [u8; 4], factor: f32) -> [u8; 4] {
    let channel = |index: usize| {
        (start[index] as f32 + (end[index] as f32 - start[index] as f32) * factor).round() as u8
    };
    [channel(0), channel(1), channel(2), channel(3)]
}
