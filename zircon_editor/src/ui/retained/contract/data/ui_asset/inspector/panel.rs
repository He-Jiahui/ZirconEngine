use super::binding::UiAssetInspectorBindingData;
use super::layout::UiAssetInspectorLayoutData;
use super::slot::UiAssetInspectorSlotData;
use super::widget::UiAssetInspectorWidgetData;

#[derive(Clone, Default)]
pub(crate) struct UiAssetInspectorPanelData {
    pub widget: UiAssetInspectorWidgetData,
    pub slot: UiAssetInspectorSlotData,
    pub layout: UiAssetInspectorLayoutData,
    pub binding: UiAssetInspectorBindingData,
}
