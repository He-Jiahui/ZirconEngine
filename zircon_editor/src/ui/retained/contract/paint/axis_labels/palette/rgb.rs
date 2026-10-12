//! 比例颜色仅缩放 RGB 而保留透明度，供轴文字和链接资源的主题投影共享。

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
