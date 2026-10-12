/// 说明本帧能否沿用上帧空间索引；缺失或不可信历史要求完整重建。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VisibilityBvhUpdateStrategy {
    #[default]
    FullRebuild,
    Incremental,
}
