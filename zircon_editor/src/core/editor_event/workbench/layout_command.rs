use serde::{Deserialize, Serialize};

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
