use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorRegionRole {
    PlacementTools,
    ProjectTree,
    HierarchyStructure,
    DetailInspector,
    ConsoleDiagnosticsTimeline,
    CenterDocument,
}
