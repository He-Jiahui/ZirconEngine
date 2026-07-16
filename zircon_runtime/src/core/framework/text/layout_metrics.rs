#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextLayoutMetrics {
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
    pub descent: f32,
    pub line_gap: f32,
    pub baseline: f32,
}
