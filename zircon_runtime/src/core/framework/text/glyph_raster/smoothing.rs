#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextGlyphRasterSmoothing {
    None,
    #[default]
    Grayscale,
    Subpixel,
}
