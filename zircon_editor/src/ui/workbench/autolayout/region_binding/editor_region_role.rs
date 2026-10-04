use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 区域内容职责的资产协议；绑定入口验证它与EditorRegion一致，不从面板路径猜测。
pub enum EditorRegionRole {
    PlacementTools,
    ProjectTree,
    HierarchyStructure,
    DetailInspector,
    ConsoleDiagnosticsTimeline,
    CenterDocument,
}
