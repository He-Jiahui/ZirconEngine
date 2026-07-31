use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PanePayloadKind {
    TemplateV2,
    ConsoleV1,
    InspectorV1,
    HierarchyV1,
    AnimationSequenceV1,
    AnimationGraphV1,
    RuntimeDiagnosticsV1,
    PerformanceTimelineV1,
    ModulePluginsV1,
    BuildExportV1,
    GeneratedBottomV1,
    UiComponentShowcaseV1,
}
