// TODO: [CR-RUNTIME-SCENE-EXTRACT-0002] 确认页依赖是否需约束驻留调度；当前只进入调试快照，规划器未读取。
/// cooked 页间关系的提取表示，调试快照可用于解释页结构。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderVirtualGeometryPageDependency {
    pub page_id: u32,
    pub parent_page_id: Option<u32>,
    /// Stable child list from cooked VG data; runtime may derive its parent map from either side.
    pub child_page_ids: Vec<u32>,
}
