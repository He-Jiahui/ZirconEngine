#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// 字形位图字节的通道布局；消费者按格式区分单通道覆盖率、子像素覆盖率和 RGBA 颜色。
pub enum TextGlyphBitmapFormat {
    AlphaMask,
    SubpixelMask,
    ColorRgba,
}
