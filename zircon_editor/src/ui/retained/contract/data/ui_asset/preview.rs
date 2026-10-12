use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::canvas::UiAssetPreviewCanvasData;
use super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewMockData {
    pub subject_collection: UiAssetStringSelectionData,
    pub subject_node_id: SharedString,
    pub collection: UiAssetStringSelectionData,
    pub property: SharedString,
    pub kind: SharedString,
    pub value: SharedString,
    pub expression_result: SharedString,
    pub nested_collection: UiAssetStringSelectionData,
    pub nested_key: SharedString,
    pub nested_kind: SharedString,
    pub nested_value: SharedString,
    pub suggestion_collection: UiAssetStringSelectionData,
    pub schema_items: ModelRc<SharedString>,
    pub state_graph_items: ModelRc<SharedString>,
    pub can_edit: bool,
    pub can_clear: bool,
    pub nested_can_edit: bool,
    pub nested_can_add: bool,
    pub nested_can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewPanelData {
    pub preset: SharedString,
    pub summary: SharedString,
    pub available: bool,
    pub canvas: UiAssetPreviewCanvasData,
    pub mock: UiAssetPreviewMockData,
}
