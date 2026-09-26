/// 命中视觉簇边界时选择哪一侧，决定回映的源字节位置与光标亲和性。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextVisualBoundaryBias {
    LeadingCurrent,
    TrailingPrevious,
}
