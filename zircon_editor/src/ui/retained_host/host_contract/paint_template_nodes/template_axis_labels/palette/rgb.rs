const COLOR_CHANNEL_MIN: f32 = 0.0;
const COLOR_CHANNEL_MAX: f32 = 255.0;

pub(super) fn scaled_rgb(color: [u8; 4], scale: [f32; 3]) -> [u8; 4] {
    [
        scaled_channel(color[0], scale[0]),
        scaled_channel(color[1], scale[1]),
        scaled_channel(color[2], scale[2]),
        color[3],
    ]
}

fn scaled_channel(value: u8, scale: f32) -> u8 {
    (f32::from(value) * scale)
        .round()
        .clamp(COLOR_CHANNEL_MIN, COLOR_CHANNEL_MAX) as u8
}
