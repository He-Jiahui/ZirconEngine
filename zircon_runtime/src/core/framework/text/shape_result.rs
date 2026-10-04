use super::{TextDirection, TextLayoutMetrics, TextShapeRun};

/// 文本服务投影出的中立结果，包含各硬行字形、整体度量和最终段落方向。
#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeResult {
    pub runs: Vec<TextShapeRun>,
    pub metrics: TextLayoutMetrics,
    pub resolved_direction: TextDirection,
}
