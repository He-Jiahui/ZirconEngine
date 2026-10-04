use serde::{Deserialize, Serialize};

use crate::core::editor_message::DocumentId;

use super::{
    ActivityDrawerMode, ActivityDrawerSlot, MainPageId, SplitAxis, SplitPlacement,
    TabInsertionAnchor, ViewHost, ViewInstanceId, WorkspaceTarget,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutCommand {
    OpenView {
        instance_id: ViewInstanceId,
        target: ViewHost,
    },
    CloseView {
        instance_id: ViewInstanceId,
    },
    CloseViews {
        window_id: MainPageId,
        instance_ids: Vec<ViewInstanceId>,
    },
    FocusView {
        instance_id: ViewInstanceId,
    },
    MoveView {
        instance_id: ViewInstanceId,
        target: ViewHost,
    },
    AttachView {
        instance_id: ViewInstanceId,
        target: ViewHost,
        anchor: Option<TabInsertionAnchor>,
    },
    DetachViewToWindow {
        instance_id: ViewInstanceId,
        new_window: MainPageId,
    },
    CreateSplit {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
        new_instance: ViewInstanceId,
    },
    ResizeSplit {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        ratio: f32,
    },
    SetDrawerMode {
        slot: ActivityDrawerSlot,
        mode: ActivityDrawerMode,
    },
    SetDrawerExtent {
        slot: ActivityDrawerSlot,
        extent: f32,
    },
    SetDrawerRegionExtent {
        slot: ActivityDrawerSlot,
        extent: f32,
    },
    ActivateDrawerTab {
        slot: ActivityDrawerSlot,
        instance_id: ViewInstanceId,
    },
    ActivateMainPage {
        page_id: MainPageId,
    },
    SavePreset {
        name: String,
    },
    LoadPreset {
        name: String,
    },
    ResetToDefault,
}

/// Every mutable authoring authority participating in a document close decision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCloseRevision {
    pub external_generation: u64,
    pub edit_generation: u64,
    pub history_generation: u64,
    pub source_revision: Option<u64>,
}
