use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanePayloadKind {
    ConsoleV1,
    InspectorV1,
    HierarchyV1,
    AnimationSequenceV1,
    AnimationGraphV1,
    RuntimeDiagnosticsV1,
    ModulePluginsV1,
    UiComponentShowcaseV1,
}
