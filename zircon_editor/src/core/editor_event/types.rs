use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use zircon_runtime::core::framework::animation::AnimationTrackPath;

use super::{InspectorFieldChange, LayoutCommand, MenuAction, SelectionHostEvent};

macro_rules! define_id {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
        }
    };
}

define_id!(EditorEventId);
define_id!(EditorEventSequence);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventSource {
    Slint,
    Headless,
    Cli,
    Mcp,
    Replay,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetSurface {
    Activity,
    Browser,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetViewMode {
    List,
    Thumbnail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetUtilityTab {
    Preview,
    References,
    Metadata,
    Plugins,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorAssetEvent {
    OpenAsset {
        asset_path: String,
    },
    SelectFolder {
        folder_id: String,
    },
    SelectItem {
        asset_uuid: String,
    },
    ActivateReference {
        asset_uuid: String,
    },
    SetSearchQuery {
        query: String,
    },
    SetKindFilter {
        kind: Option<String>,
    },
    SetViewMode {
        surface: EditorAssetSurface,
        view_mode: EditorAssetViewMode,
    },
    SetUtilityTab {
        surface: EditorAssetSurface,
        tab: EditorAssetUtilityTab,
    },
    OpenAssetBrowser,
    LocateSelectedAsset,
    ImportModel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorInspectorEvent {
    pub subject_path: String,
    pub changes: Vec<InspectorFieldChange>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorDraftEvent {
    SetInspectorField {
        subject_path: String,
        field_id: String,
        value: String,
    },
    SetMeshImportPath {
        value: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorOperationEvent {
    ControlFailure { operation_id: String, error: String },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorAnimationEvent {
    AddKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    RemoveKey {
        track_path: AnimationTrackPath,
        frame: u32,
    },
    CreateTrack {
        track_path: AnimationTrackPath,
    },
    RemoveTrack {
        track_path: AnimationTrackPath,
    },
    RebindTrack {
        from_track_path: AnimationTrackPath,
        to_track_path: AnimationTrackPath,
    },
    ScrubTimeline {
        frame: u32,
    },
    SetTimelineRange {
        start_frame: u32,
        end_frame: u32,
    },
    SelectTimelineSpan {
        track_path: AnimationTrackPath,
        start_frame: u32,
        end_frame: u32,
    },
    SetPlayback {
        playing: bool,
        looping: bool,
        speed: f32,
    },
    AddGraphNode {
        graph_path: String,
        node_id: String,
        node_kind: String,
    },
    RemoveGraphNode {
        graph_path: String,
        node_id: String,
    },
    ConnectGraphNodes {
        graph_path: String,
        from_node_id: String,
        to_node_id: String,
    },
    DisconnectGraphNodes {
        graph_path: String,
        from_node_id: String,
        to_node_id: String,
    },
    SetGraphParameter {
        graph_path: String,
        parameter_name: String,
        value_literal: String,
    },
    CreateState {
        state_machine_path: String,
        state_name: String,
        graph_path: String,
    },
    RemoveState {
        state_machine_path: String,
        state_name: String,
    },
    SetEntryState {
        state_machine_path: String,
        state_name: String,
    },
    CreateTransition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
        duration_frames: u32,
    },
    RemoveTransition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
    },
    SetTransitionCondition {
        state_machine_path: String,
        from_state: String,
        to_state: String,
        parameter_name: String,
        operator: String,
        value_literal: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorViewportEvent {
    PointerMoved { x: f32, y: f32 },
    LeftPressed { x: f32, y: f32 },
    LeftReleased,
    RightPressed { x: f32, y: f32 },
    RightReleased,
    MiddlePressed { x: f32, y: f32 },
    MiddleReleased,
    Scrolled { delta: f32 },
    Resized { width: u32, height: u32 },
    SetTool { tool: SceneViewportTool },
    SetTransformSpace { space: TransformSpace },
    SetProjectionMode { mode: ProjectionMode },
    AlignView { orientation: ViewOrientation },
    SetDisplayMode { mode: DisplayMode },
    SetGridMode { mode: GridMode },
    SetTranslateSnap { step: f32 },
    SetRotateSnapDegrees { step: f32 },
    SetScaleSnap { step: f32 },
    SetPreviewLighting { enabled: bool },
    SetPreviewSkybox { enabled: bool },
    SetGizmosEnabled { enabled: bool },
    FrameSelection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventTransient {
    HoverNode { node_path: String, hovered: bool },
    FocusNode { node_path: String },
    PressNode { node_path: String, pressed: bool },
    SetDrawerResizing { drawer_id: String, resizing: bool },
    BeginViewDrag { instance_id: String },
    EndViewDrag,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorEvent {
    WorkbenchMenu(MenuAction),
    Layout(LayoutCommand),
    Selection(SelectionHostEvent),
    Asset(EditorAssetEvent),
    Draft(EditorDraftEvent),
    Animation(EditorAnimationEvent),
    Inspector(EditorInspectorEvent),
    Viewport(EditorViewportEvent),
    Operation(EditorOperationEvent),
    Transient(EditorEventTransient),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorEventEnvelope {
    pub source: EditorEventSource,
    pub event: EditorEvent,
}

impl EditorEventEnvelope {
    pub fn new(source: EditorEventSource, event: EditorEvent) -> Self {
        Self { source, event }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventEffect {
    PresentationChanged,
    LayoutChanged,
    RenderChanged,
    ReflectionChanged,
    PresentWelcomeRequested,
    ProjectOpenRequested,
    ProjectSaveRequested,
    AssetDetailsRefreshRequested,
    AssetPreviewRefreshRequested,
    ImportModelRequested,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorEventResult {
    pub value: Option<Value>,
    pub error: Option<String>,
}

impl EditorEventResult {
    pub fn success(value: Value) -> Self {
        Self {
            value: Some(value),
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            value: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventUndoPolicy {
    NonUndoable,
    DelegatedToEditorHistory,
    FutureInverseEvent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorEventRecord {
    pub event_id: EditorEventId,
    pub sequence: EditorEventSequence,
    pub source: EditorEventSource,
    pub event: EditorEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_display_name: Option<String>,
    pub effects: Vec<EditorEventEffect>,
    pub undo_policy: EditorEventUndoPolicy,
    pub before_revision: u64,
    pub after_revision: u64,
    pub result: EditorEventResult,
}
