use super::PaneConstraints;

#[derive(Clone, Copy, Debug)]
/// 本轮logical区域需求；可见占位与展开交互分开表达，不回写持久化抽屉状态。
pub(super) struct RegionState {
    pub(super) visible: bool,
    pub(super) expanded: bool,
    pub(super) constraints: PaneConstraints,
}
