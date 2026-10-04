use super::{
    TextDirection, TextFontRequest, TextLayoutError, TextRenderMode, TextShapeRequest,
    TextShapeResult,
};

/// 对外的后端中立排版接口；具体字体集合和 shaping backend 留在 text 域。
///
/// 调用方传入借用请求，取得可供布局和渲染使用的拥有型结果或类型化错误。
pub trait TextLayoutService: Send + Sync {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode;

    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection;

    /// 将一次请求投影到中立字形和度量；实现应在同一字体代际内完成结果投影。
    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError>;
}
