use super::super::super::UiTextRange;

/// 一个连续的视觉字节区间；双向文本中的同一逻辑源范围可对应多个区间。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiTextVisualSpan {
    pub visual_range: UiTextRange,
}
