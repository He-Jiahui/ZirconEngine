use super::super::{
    binding_inspector::{build_binding_fields, UiAssetBindingInspectorFields},
    hierarchy_projection::{build_component_contract_items, build_inspector_items},
    inspector_fields::{
        build_inspector_fields, build_selected_node_prop_state_items, UiAssetInspectorFields,
    },
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group,
        build_structured_layout_semantic_fields, build_structured_slot_semantic_fields,
        UiAssetInspectorSemanticGroup, UiAssetStructuredLayoutSemanticFields,
        UiAssetStructuredSlotSemanticFields,
    },
    preview_mock::{
        build_preview_mock_fields, build_preview_state_graph_items, UiAssetPreviewMockFields,
    },
    root_class_policy_state::root_class_policy_label,
    runtime_report_state::UiAssetRuntimeReportProjection,
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::asset_editor::{
    presentation::UiAssetEditorWidgetPropStateItem, UiAssetEditorReflectionModel,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) struct UiAssetInspectorPaneData {
    pub(super) preview_mock_fields: UiAssetPreviewMockFields,
    pub(super) preview_state_graph_items: Vec<String>,
    pub(super) inspector_fields: UiAssetInspectorFields,
    pub(super) binding_fields: UiAssetBindingInspectorFields,
    pub(super) runtime_report: UiAssetRuntimeReportProjection,
    pub(super) slot_semantic_group: UiAssetInspectorSemanticGroup,
    pub(super) structured_slot_semantic: UiAssetStructuredSlotSemanticFields,
    pub(super) slot_semantic_selected_index: i32,
    pub(super) slot_semantic_path: String,
    pub(super) slot_semantic_value: String,
    pub(super) layout_semantic_group: UiAssetInspectorSemanticGroup,
    pub(super) structured_layout_semantic: UiAssetStructuredLayoutSemanticFields,
    pub(super) layout_semantic_selected_index: i32,
    pub(super) layout_semantic_path: String,
    pub(super) layout_semantic_value: String,
    pub(super) widget_prop_state_items: Vec<String>,
    pub(super) widget_prop_state_rows: Vec<UiAssetEditorWidgetPropStateItem>,
    pub(super) inspector_items: Vec<String>,
    pub(super) component_root_class_policy: String,
    pub(super) can_edit_component_root_class_policy: bool,
    pub(super) promote_asset_id: String,
    pub(super) promote_component_name: String,
    pub(super) promote_document_id: String,
    pub(super) can_edit_promote_draft: bool,
}

impl UiAssetEditorSession {
    pub(super) fn inspector_pane_presentation(
        &self,
        reflection: &UiAssetEditorReflectionModel,
        can_promote_to_external_widget: bool,
    ) -> UiAssetInspectorPaneData {
        zircon_runtime::profile_scope!("editor", "asset_editor.presentation", "inspector",);
        let preview_mock_fields = build_preview_mock_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
        );
        let preview_state_graph_items =
            build_preview_state_graph_items(&self.last_valid_document, &self.preview_mock_state);
        let inspector_fields = build_inspector_fields(&self.last_valid_document, &self.selection);
        let binding_fields = build_binding_fields(
            &self.last_valid_document,
            &self.selection,
            &self.preview_mock_state,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        let runtime_report = self.runtime_report_projection();
        let slot_semantic_group =
            build_slot_semantic_group(&self.last_valid_document, &self.selection);
        let structured_slot_semantic =
            build_structured_slot_semantic_fields(&self.last_valid_document, &self.selection);
        let (slot_semantic_selected_index, slot_semantic_path, slot_semantic_value) = self
            .selected_slot_semantic_path
            .as_deref()
            .and_then(|path| {
                slot_semantic_group
                    .entries
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                slot_semantic_group
                    .entries
                    .get(index)
                    .map(|entry| (index as i32, entry.path.clone(), entry.literal.clone()))
            })
            .unwrap_or_else(|| (-1, String::new(), String::new()));
        let layout_semantic_group =
            build_layout_semantic_group(&self.last_valid_document, &self.selection);
        let structured_layout_semantic =
            build_structured_layout_semantic_fields(&self.last_valid_document, &self.selection);
        let (layout_semantic_selected_index, layout_semantic_path, layout_semantic_value) = self
            .selected_layout_semantic_path
            .as_deref()
            .and_then(|path| {
                layout_semantic_group
                    .entries
                    .iter()
                    .position(|entry| entry.path.as_str() == path)
            })
            .and_then(|index| {
                layout_semantic_group
                    .entries
                    .get(index)
                    .map(|entry| (index as i32, entry.path.clone(), entry.literal.clone()))
            })
            .unwrap_or_else(|| (-1, String::new(), String::new()));
        let widget_prop_state_rows =
            build_selected_node_prop_state_items(&self.last_valid_document, &self.selection);
        let widget_prop_state_items = widget_prop_state_rows
            .iter()
            .map(|item| item.display.clone())
            .collect::<Vec<_>>();
        let component_root_class_policy = self
            .selected_component_root_class_policy()
            .map(root_class_policy_label);
        let mut inspector_items = build_inspector_items(reflection);
        inspector_items.extend(widget_prop_state_items.clone());
        inspector_items.extend(build_component_contract_items(component_root_class_policy));
        let promote_draft = self.selected_promote_widget_draft();
        record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneInspectorBuildCount, 1.0);
        UiAssetInspectorPaneData {
            preview_mock_fields,
            preview_state_graph_items,
            inspector_fields,
            binding_fields,
            runtime_report,
            slot_semantic_group,
            structured_slot_semantic,
            slot_semantic_selected_index,
            slot_semantic_path,
            slot_semantic_value,
            layout_semantic_group,
            structured_layout_semantic,
            layout_semantic_selected_index,
            layout_semantic_path,
            layout_semantic_value,
            widget_prop_state_items,
            widget_prop_state_rows,
            inspector_items,
            component_root_class_policy: component_root_class_policy
                .unwrap_or_default()
                .to_string(),
            can_edit_component_root_class_policy: self
                .can_edit_selected_component_root_class_policy(),
            promote_asset_id: promote_draft
                .as_ref()
                .map(|draft| draft.asset_id.clone())
                .unwrap_or_default(),
            promote_component_name: promote_draft
                .as_ref()
                .map(|draft| draft.component_name.clone())
                .unwrap_or_default(),
            promote_document_id: promote_draft
                .as_ref()
                .map(|draft| draft.document_id.clone())
                .unwrap_or_default(),
            can_edit_promote_draft: can_promote_to_external_widget,
        }
    }
}
