use super::{TextDirection, TextLayoutMetrics, TextShapeRun};

/// 文本服务投影出的中立结果，包含各硬行字形、整体度量和最终段落方向。
///
/// # 使用与生命周期
/// 结果拥有行与字形集合，不借用请求，也不保存原文；源范围仍须结合对应请求文本解释。
/// 各行及字形沿用服务输出顺序，整体解析方向不能代表每个字形的双向方向。
/// 参见 [`TextLayoutService::shape`](super::TextLayoutService::shape) 与 [`TextShapeRun`]。
#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeResult {
    /// 服务投影出的各成形硬行；每个条目拥有自己的字形集合。
    pub runs: Vec<TextShapeRun>,
    /// 服务返回的整体排版度量，使用与字形位置相同的布局长度尺度。
    pub metrics: TextLayoutMetrics,
    /// 解析后的整体基础方向；各硬行的代表方向另存于 `runs`。
    pub resolved_direction: TextDirection,
}
