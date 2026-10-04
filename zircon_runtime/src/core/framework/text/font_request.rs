use super::TextRenderMode;

/// 排版调用方借用的字体选择条件；由文本服务映射到实现层样式和字体集合。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextFontRequest<'a> {
    pub families: &'a [&'a str],
    pub asset: Option<&'a str>,
    pub size: f32,
    pub weight: u16,
    pub stretch: u16,
    pub italic: bool,
    pub render_mode: TextRenderMode,
}

impl Default for TextFontRequest<'_> {
    fn default() -> Self {
        Self {
            families: &[],
            asset: None,
            size: 16.0,
            weight: 400,
            stretch: 100,
            italic: false,
            render_mode: TextRenderMode::Auto,
        }
    }
}
