#[derive(Clone, Debug, PartialEq)]
pub enum PanePayload {
    ConsoleV1(ConsolePanePayload),
    InspectorV1(InspectorPanePayload),
    HierarchyV1(HierarchyPanePayload),
    AnimationSequenceV1(AnimationSequencePanePayload),
    AnimationGraphV1(AnimationGraphPanePayload),
    RuntimeDiagnosticsV1(RuntimeDiagnosticsPanePayload),
    UiComponentShowcaseV1(UiComponentShowcasePanePayload),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsolePanePayload {
    pub status_text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InspectorPanePayload {
    pub node_id: u64,
    pub name: String,
    pub parent: String,
    pub translation: [String; 3],
    pub delete_enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyPanePayload {
    pub nodes: Vec<HierarchyPaneNodePayload>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HierarchyPaneNodePayload {
    pub node_id: u64,
    pub name: String,
    pub depth: u32,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationSequencePanePayload {
    pub mode: String,
    pub asset_path: String,
    pub status: String,
    pub selection: String,
    pub current_frame: u32,
    pub timeline_start_frame: u32,
    pub timeline_end_frame: u32,
    pub playback_label: String,
    pub track_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationGraphPanePayload {
    pub mode: String,
    pub asset_path: String,
    pub status: String,
    pub selection: String,
    pub parameter_items: Vec<String>,
    pub node_items: Vec<String>,
    pub state_items: Vec<String>,
    pub transition_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeDiagnosticsPanePayload {
    pub summary: String,
    pub render_status: String,
    pub physics_status: String,
    pub animation_status: String,
    pub detail_items: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiComponentShowcasePanePayload {
    pub state_summary: String,
}
