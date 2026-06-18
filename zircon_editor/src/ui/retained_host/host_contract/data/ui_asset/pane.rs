use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::TemplatePaneNodeData;
use super::canvas::UiAssetPaletteDragData;
use super::common::UiAssetStringSelectionData;
use super::inspector::UiAssetInspectorPanelData;
use super::preview::UiAssetPreviewPanelData;
use super::source::UiAssetSourcePanelData;
use super::style::UiAssetStylePanelData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetPaneHeaderData {
    pub asset_id: SharedString,
    pub mode: SharedString,
    pub status: SharedString,
    pub selection: SharedString,
    pub shell_state: SharedString,
    pub emergency_summary: SharedString,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetActionStateData {
    pub can_reload_from_disk: bool,
    pub can_keep_local_and_save: bool,
    pub can_save_local_copy: bool,
    pub can_open_diff_snapshot: bool,
    pub can_save: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub can_emergency_reload: bool,
    pub can_emergency_revert: bool,
    pub can_emergency_open_asset_browser: bool,
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
pub(crate) struct UiAssetDesignerToolStateData {
    pub mode: SharedString,
    pub can_select: bool,
    pub can_resize_slot: bool,
    pub can_preview_interact: bool,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetCollectionPanelData {
    pub palette: UiAssetStringSelectionData,
    pub hierarchy: UiAssetStringSelectionData,
    pub preview: UiAssetStringSelectionData,
}

#[derive(Clone, Default)]
pub(crate) struct UiAssetRuntimeReportData {
    pub action_policy_items: ModelRc<SharedString>,
    pub capability_explanation_items: ModelRc<SharedString>,
    pub host_enforcement_items: ModelRc<SharedString>,
    pub unsafe_action_guidance_items: ModelRc<SharedString>,
    pub locale_preview: UiAssetStringSelectionData,
    pub locale_preview_selected_locale: SharedString,
    pub locale_dependency_items: ModelRc<SharedString>,
    pub locale_extraction_items: ModelRc<SharedString>,
    pub locale_diagnostic_items: ModelRc<SharedString>,
    pub resource_dependency_items: ModelRc<SharedString>,
    pub resource_diagnostic_items: ModelRc<SharedString>,
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
    pub runtime_report: UiAssetRuntimeReportData,
    pub designer_tools: UiAssetDesignerToolStateData,
    pub palette_drag: UiAssetPaletteDragData,
    pub style: UiAssetStylePanelData,
    pub inspector: UiAssetInspectorPanelData,
}
