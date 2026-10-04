#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextGlyphRasterHinting {
    None,
    #[default]
    Vertical,
    Full,
}
