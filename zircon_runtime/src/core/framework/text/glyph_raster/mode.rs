#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TextGlyphRasterMode {
    #[default]
    Outline,
    ColorPreferred,
}
