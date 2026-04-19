use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};

use crate::ui::binding_dispatch::SelectionHostEvent;
use crate::ui::workbench::model::MenuAction;
use crate::{InspectorFieldChange, LayoutCommand};

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
    Inspector(EditorInspectorEvent),
    Viewport(EditorViewportEvent),
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
    pub effects: Vec<EditorEventEffect>,
    pub undo_policy: EditorEventUndoPolicy,
    pub before_revision: u64,
    pub after_revision: u64,
    pub result: EditorEventResult,
}
