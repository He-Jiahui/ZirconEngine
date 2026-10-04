use super::{
    TextGlyphRasterHinting, TextGlyphRasterMode, TextGlyphRasterSmoothing, TextGlyphSyntheticStyle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// 描述送往字体后端的一次字形栅格请求，包括设备像素尺寸、相位、提示和合成样式。
pub struct TextGlyphRasterRequest {
    pub glyph_id: u32,
    pub physical_ppem: u32,
    pub horizontal_phase: u8,
    pub vertical_phase: u8,
    pub mode: TextGlyphRasterMode,
    pub hinting: TextGlyphRasterHinting,
    pub smoothing: TextGlyphRasterSmoothing,
    pub synthetic: TextGlyphSyntheticStyle,
}

impl TextGlyphRasterRequest {
    pub const HORIZONTAL_PHASE_COUNT: u8 = 3;
    pub const VERTICAL_PHASE_COUNT: u8 = 4;

    pub const fn new(glyph_id: u32, physical_ppem: u32, mode: TextGlyphRasterMode) -> Self {
        Self {
            glyph_id,
            physical_ppem,
            horizontal_phase: 0,
            vertical_phase: 0,
            mode,
            hinting: TextGlyphRasterHinting::Vertical,
            smoothing: TextGlyphRasterSmoothing::Grayscale,
            synthetic: TextGlyphSyntheticStyle { oblique: false },
        }
    }

    /// 直接写入水平三档、垂直四档相位；此 builder 不裁剪越界值，后端会将其作为无效请求拒绝。
    pub const fn with_subpixel_phase(mut self, horizontal: u8, vertical: u8) -> Self {
        self.horizontal_phase = horizontal;
        self.vertical_phase = vertical;
        self
    }

    /// 将像素位置的小数部分量化到共享相位网格；负坐标按周期归一，非有限值归到零相位。
    pub fn with_subpixel_position(mut self, horizontal: f32, vertical: f32) -> Self {
        self.horizontal_phase = Self::horizontal_phase_for_position(horizontal);
        self.vertical_phase = Self::vertical_phase_for_position(vertical);
        self
    }

    pub fn horizontal_phase_for_position(position: f32) -> u8 {
        quantized_phase(position, Self::HORIZONTAL_PHASE_COUNT)
    }

    pub fn vertical_phase_for_position(position: f32) -> u8 {
        quantized_phase(position, Self::VERTICAL_PHASE_COUNT)
    }

    pub const fn with_hinting(mut self, hinting: TextGlyphRasterHinting) -> Self {
        self.hinting = hinting;
        self
    }

    pub const fn with_smoothing(mut self, smoothing: TextGlyphRasterSmoothing) -> Self {
        self.smoothing = smoothing;
        self
    }

    pub const fn with_synthetic_style(mut self, synthetic: TextGlyphSyntheticStyle) -> Self {
        self.synthetic = synthetic;
        self
    }
}

fn quantized_phase(position: f32, phase_count: u8) -> u8 {
    if !position.is_finite() || phase_count == 0 {
        return 0;
    }
    let fraction = position.rem_euclid(1.0);
    ((fraction * f32::from(phase_count)).floor() as u8).min(phase_count - 1)
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
