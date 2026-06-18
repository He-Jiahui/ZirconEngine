use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::common::UiAssetStringSelectionData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorSemanticData {
    pub title: SharedString,
    pub collection: UiAssetStringSelectionData,
    pub path: SharedString,
    pub value: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorSlotData {
    pub padding: SharedString,
    pub width_preferred: SharedString,
    pub height_preferred: SharedString,
    pub semantic: UiAssetInspectorSemanticData,
    pub kind: SharedString,
    pub linear_main_weight: SharedString,
    pub linear_main_stretch: SharedString,
    pub linear_cross_weight: SharedString,
    pub linear_cross_stretch: SharedString,
    pub overlay_anchor_x: SharedString,
    pub overlay_anchor_y: SharedString,
    pub overlay_pivot_x: SharedString,
    pub overlay_pivot_y: SharedString,
    pub overlay_position_x: SharedString,
    pub overlay_position_y: SharedString,
    pub overlay_z_index: SharedString,
    pub grid_row: SharedString,
    pub grid_column: SharedString,
    pub grid_row_span: SharedString,
    pub grid_column_span: SharedString,
    pub flow_break_before: SharedString,
    pub flow_alignment: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorLayoutData {
    pub width_preferred: SharedString,
    pub height_preferred: SharedString,
    pub semantic: UiAssetInspectorSemanticData,
    pub kind: SharedString,
    pub box_gap: SharedString,
    pub scroll_axis: SharedString,
    pub scroll_gap: SharedString,
    pub scrollbar_visibility: SharedString,
    pub virtualization_item_extent: SharedString,
    pub virtualization_overscan: SharedString,
    pub clip: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorBindingData {
    pub collection: UiAssetStringSelectionData,
    pub binding_id: SharedString,
    pub binding_event: SharedString,
    pub event_collection: UiAssetStringSelectionData,
    pub binding_route: SharedString,
    pub binding_route_target: SharedString,
    pub binding_action_target: SharedString,
    pub route_suggestion_collection: UiAssetStringSelectionData,
    pub action_suggestion_collection: UiAssetStringSelectionData,
    pub action_kind_collection: UiAssetStringSelectionData,
    pub payload_collection: UiAssetStringSelectionData,
    pub payload_suggestion_collection: UiAssetStringSelectionData,
    pub payload_key: SharedString,
    pub payload_value: SharedString,
    pub schema_items: ModelRc<SharedString>,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorWidgetPropStateData {
    pub kind: SharedString,
    pub path: SharedString,
    pub value: SharedString,
    pub display: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorWidgetData {
    pub selected_node_id: SharedString,
    pub parent_node_id: SharedString,
    pub mount: SharedString,
    pub widget_kind: SharedString,
    pub widget_label: SharedString,
    pub control_id: SharedString,
    pub text_prop: SharedString,
    pub component_root_class_policy: SharedString,
    pub can_edit_control_id: bool,
    pub can_edit_text_prop: bool,
    pub can_edit_component_root_class_policy: bool,
    pub promote_asset_id: SharedString,
    pub promote_component_name: SharedString,
    pub promote_document_id: SharedString,
    pub can_edit_promote_draft: bool,
    pub prop_state_rows: ModelRc<UiAssetInspectorWidgetPropStateData>,
    pub prop_state_items: ModelRc<SharedString>,
    pub items: ModelRc<SharedString>,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorPanelData {
    pub widget: UiAssetInspectorWidgetData,
    pub slot: UiAssetInspectorSlotData,
    pub layout: UiAssetInspectorLayoutData,
    pub binding: UiAssetInspectorBindingData,
}
