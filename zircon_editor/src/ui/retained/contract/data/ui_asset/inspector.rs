#[path = "inspector/binding.rs"]
mod binding;
#[path = "inspector/layout.rs"]
mod layout;
#[path = "inspector/panel.rs"]
mod panel;
#[path = "inspector/semantic.rs"]
mod semantic;
#[path = "inspector/slot.rs"]
mod slot;
#[path = "inspector/widget.rs"]
mod widget;

pub(crate) fn ui_asset_canvas_anchor_opener_control_id(
    instance_id: &str,
) -> crate::ui::retained_host::primitives::SharedString {
    format!("UiAssetSlotFieldAnchorsValue|{instance_id}").into()
}

pub(crate) use binding::UiAssetInspectorBindingData;
pub(crate) use layout::UiAssetInspectorLayoutData;
pub(crate) use panel::UiAssetInspectorPanelData;
pub(crate) use semantic::UiAssetInspectorSemanticData;
pub(crate) use slot::UiAssetInspectorSlotData;
pub(crate) use widget::{UiAssetInspectorWidgetData, UiAssetInspectorWidgetPropStateData};
