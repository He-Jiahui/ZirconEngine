use crate::ui::retained_host::primitives::{ModelRc, SharedString};

use super::super::common::UiAssetStringSelectionData;

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
