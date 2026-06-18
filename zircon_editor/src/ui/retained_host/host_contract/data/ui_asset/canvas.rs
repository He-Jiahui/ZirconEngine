use crate::ui::retained_host::primitives::{ModelRc, SharedString};

#[derive(Clone, Default)]
pub(crate) struct UiAssetCanvasNodeData {
    pub node_id: SharedString,
    pub label: SharedString,
    pub kind: SharedString,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub depth: i32,
    pub z_index: i32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetCanvasSlotTargetData {
    pub label: SharedString,
    pub detail: SharedString,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPaletteDragData {
    pub target_preview_index: i32,
    pub target_action: SharedString,
    pub target_label: SharedString,
    pub slot_target_items: ModelRc<UiAssetCanvasSlotTargetData>,
    pub candidate_items: ModelRc<SharedString>,
    pub candidate_selected_index: i32,
    pub target_chooser_active: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewCanvasData {
    pub width: f32,
    pub height: f32,
    pub items: ModelRc<UiAssetCanvasNodeData>,
}
