use super::super::super::UiTextRange;

/// 一个连续的视觉字节区间；双向文本中的同一逻辑源范围可对应多个区间。
///
/// # 坐标与约束
/// 沿用所属行 `visual_range` 的视觉 UTF-8 字节坐标，起点包含、终点不包含。
/// 直接组装此值不检查区间顺序或字符边界；它只保存偏移，不持有行或文档。
///
/// # 参考
/// [`UiTextLineSourceMap::visual_spans_for_source_range`](super::UiTextLineSourceMap::visual_spans_for_source_range)
/// 生成这些范围，消费者再通过同一映射将边界换算为排版前进距离。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiTextVisualSpan {
    /// 视觉文本的半开字节范围，使用所属行的视觉坐标系。
    pub visual_range: UiTextRange,
}
