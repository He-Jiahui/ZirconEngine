/// 命中视觉簇边界时选择哪一侧，决定回映的源字节位置与光标亲和性。
///
/// # 使用
/// 与 [`UiTextLineSourceMap::caret_for_visual_boundary`](super::UiTextLineSourceMap::caret_for_visual_boundary)
/// 配合使用；传入的边界是视觉簇索引，源位置的字节偏移由映射返回。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextVisualBoundaryBias {
    /// 选择边界索引对应的当前簇，生成下游亲和性的源位置；无当前簇时使用调用者的回退偏移。
    LeadingCurrent,
    /// 选择边界前一簇，生成上游亲和性的源位置；无前一簇时使用调用者的回退偏移。
    TrailingPrevious,
}
