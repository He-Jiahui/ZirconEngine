use slint::{ModelRc, SharedString};

use super::TemplatePaneNodeData;

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
pub(crate) struct UiAssetStringSelectionData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetSourceDetailData {
    pub block_label: SharedString,
    pub selected_line: i32,
    pub cursor_byte_offset: i32,
    pub selected_excerpt: SharedString,
    pub roundtrip_status: SharedString,
    pub outline: UiAssetStringSelectionData,
}

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
pub(crate) struct UiAssetThemeSourceData {
    pub collection: UiAssetStringSelectionData,
    pub selected_source_reference: SharedString,
    pub selected_source_kind: SharedString,
    pub selected_source_token_count: i32,
    pub selected_source_rule_count: i32,
    pub selected_source_available: bool,
    pub can_promote_local: bool,
    pub selected_source_token_items: ModelRc<SharedString>,
    pub selected_source_rule_items: ModelRc<SharedString>,
    pub cascade_layer_items: ModelRc<SharedString>,
    pub cascade_token_items: ModelRc<SharedString>,
    pub cascade_rule_items: ModelRc<SharedString>,
    pub compare_items: ModelRc<SharedString>,
    pub merge_preview_items: ModelRc<SharedString>,
    pub rule_helper_items: ModelRc<SharedString>,
    pub refactor_items: ModelRc<SharedString>,
    pub promote_asset_id: SharedString,
    pub promote_document_id: SharedString,
    pub promote_display_name: SharedString,
    pub can_edit_promote_draft: bool,
    pub can_prune_duplicate_local_overrides: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleRuleData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_selector: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetMatchedStyleRuleData {
    pub collection: UiAssetStringSelectionData,
    pub selected_origin: SharedString,
    pub selected_selector: SharedString,
    pub selected_specificity: i32,
    pub selected_source_order: i32,
    pub selected_declaration_items: ModelRc<SharedString>,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleRuleDeclarationData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_path: SharedString,
    pub selected_value: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleTokenData {
    pub items: ModelRc<SharedString>,
    pub selected_index: i32,
    pub selected_name: SharedString,
    pub selected_value: SharedString,
    pub can_edit: bool,
    pub can_delete: bool,
}

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
pub(crate) struct UiAssetInspectorWidgetData {
    pub selected_node_id: SharedString,
    pub parent_node_id: SharedString,
    pub mount: SharedString,
    pub widget_kind: SharedString,
    pub widget_label: SharedString,
    pub control_id: SharedString,
    pub text_prop: SharedString,
    pub can_edit_control_id: bool,
    pub can_edit_text_prop: bool,
    pub promote_asset_id: SharedString,
    pub promote_component_name: SharedString,
    pub promote_document_id: SharedString,
    pub can_edit_promote_draft: bool,
    pub items: ModelRc<SharedString>,
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
pub(crate) struct UiAssetPaneHeaderData {
    pub asset_id: SharedString,
    pub mode: SharedString,
    pub status: SharedString,
    pub selection: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetActionStateData {
    pub can_save: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub can_insert_child: bool,
    pub can_insert_after: bool,
    pub can_move_up: bool,
    pub can_move_down: bool,
    pub can_reparent_into_previous: bool,
    pub can_reparent_into_next: bool,
    pub can_reparent_outdent: bool,
    pub can_open_reference: bool,
    pub can_convert_to_reference: bool,
    pub can_extract_component: bool,
    pub can_promote_to_external_widget: bool,
    pub can_wrap_in_vertical_box: bool,
    pub can_unwrap: bool,
    pub can_create_rule: bool,
    pub can_extract_rule: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetCollectionPanelData {
    pub palette: UiAssetStringSelectionData,
    pub hierarchy: UiAssetStringSelectionData,
    pub preview: UiAssetStringSelectionData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetSourcePanelData {
    pub text: SharedString,
    pub detail: UiAssetSourceDetailData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewCanvasData {
    pub width: f32,
    pub height: f32,
    pub items: ModelRc<UiAssetCanvasNodeData>,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetPreviewPanelData {
    pub preset: SharedString,
    pub summary: SharedString,
    pub available: bool,
    pub canvas: UiAssetPreviewCanvasData,
    pub mock: UiAssetPreviewMockData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStyleStateData {
    pub hover: bool,
    pub focus: bool,
    pub pressed: bool,
    pub disabled: bool,
    pub selected: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetStylePanelData {
    pub states: UiAssetStyleStateData,
    pub class_items: ModelRc<SharedString>,
    pub theme_source: UiAssetThemeSourceData,
    pub rule: UiAssetStyleRuleData,
    pub matched_rule: UiAssetMatchedStyleRuleData,
    pub rule_declaration: UiAssetStyleRuleDeclarationData,
    pub token: UiAssetStyleTokenData,
    pub can_create_rule: bool,
    pub can_extract_rule: bool,
    pub stylesheet_items: ModelRc<SharedString>,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorPanelData {
    pub widget: UiAssetInspectorWidgetData,
    pub slot: UiAssetInspectorSlotData,
    pub layout: UiAssetInspectorLayoutData,
    pub binding: UiAssetInspectorBindingData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetEditorPaneData {
    pub nodes: ModelRc<TemplatePaneNodeData>,
    pub center_column_node: TemplatePaneNodeData,
    pub designer_panel_node: TemplatePaneNodeData,
    pub designer_canvas_panel_node: TemplatePaneNodeData,
    pub inspector_panel_node: TemplatePaneNodeData,
    pub stylesheet_panel_node: TemplatePaneNodeData,
    pub header: UiAssetPaneHeaderData,
    pub actions: UiAssetActionStateData,
    pub collections: UiAssetCollectionPanelData,
    pub source: UiAssetSourcePanelData,
    pub preview: UiAssetPreviewPanelData,
    pub palette_drag: UiAssetPaletteDragData,
    pub style: UiAssetStylePanelData,
    pub inspector: UiAssetInspectorPanelData,
}
