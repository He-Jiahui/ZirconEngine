use crate::ui::asset_editor;

use super::binding::binding_detail_rows;
use super::layout::layout_detail_rows;
use super::row_model::UiAssetDetailFieldSection;
use super::slot::slot_detail_rows;
use super::widget::widget_detail_rows;

pub(super) fn ui_asset_detail_field_sections(
    data: &asset_editor::UiAssetEditorPanePresentation,
    prop_state_rows: &[asset_editor::UiAssetEditorWidgetPropStateItem],
) -> Vec<UiAssetDetailFieldSection> {
    vec![
        UiAssetDetailFieldSection {
            section_control_id: "InspectorWidgetSection",
            detail_id: "widget",
            rows: widget_detail_rows(data, prop_state_rows),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorSlotSection",
            detail_id: "slot",
            rows: slot_detail_rows(data),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorLayoutSection",
            detail_id: "layout",
            rows: layout_detail_rows(data),
        },
        UiAssetDetailFieldSection {
            section_control_id: "InspectorBindingSection",
            detail_id: "binding",
            rows: binding_detail_rows(data),
        },
    ]
}
