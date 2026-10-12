use crate::ui::retained_host::primitives::{ModelRc, SharedString};

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
