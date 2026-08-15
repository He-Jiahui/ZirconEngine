use crate::ui::asset_editor;

use super::row_model::{push_detail_row, UiAssetDetailFieldRow};

pub(super) fn binding_detail_rows(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> Vec<UiAssetDetailFieldRow> {
    let mut rows = Vec::new();
    push_detail_row(
        &mut rows,
        "Binding ID",
        &data.inspector_binding_id,
        "binding.id.set",
        "UiAssetBindingFieldId",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Event",
        &data.inspector_binding_event,
        "binding.event.set",
        "UiAssetBindingFieldEvent",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Route",
        &data.inspector_binding_route,
        "binding.route.set",
        "UiAssetBindingFieldRoute",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Route target",
        &data.inspector_binding_route_target,
        "binding.route_target.set",
        "UiAssetBindingFieldRouteTarget",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    push_detail_row(
        &mut rows,
        "Action target",
        &data.inspector_binding_action_target,
        "binding.action_target.set",
        "UiAssetBindingFieldActionTarget",
        !data.inspector_can_edit_binding,
        data.inspector_can_edit_binding,
    );
    rows
}
