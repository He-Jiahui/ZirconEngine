use crate::ui::asset_editor;
use crate::ui::retained_host as host_contract;

pub(super) fn to_host_contract_ui_asset_actions(
    data: &asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetActionStateData {
    host_contract::UiAssetActionStateData {
        can_reload_from_disk: data.can_reload_from_disk,
        can_keep_local_and_save: data.can_keep_local_and_save,
        can_save_local_copy: data.can_save_local_copy,
        can_open_diff_snapshot: data.can_open_diff_snapshot,
        can_save: data.can_save,
        can_undo: data.can_undo,
        can_redo: data.can_redo,
        can_emergency_reload: data.can_emergency_reload,
        can_emergency_revert: data.can_emergency_revert,
        can_emergency_open_asset_browser: data.can_emergency_open_asset_browser,
        can_insert_child: data.can_insert_child,
        can_insert_after: data.can_insert_after,
        can_move_up: data.can_move_up,
        can_move_down: data.can_move_down,
        can_reparent_into_previous: data.can_reparent_into_previous,
        can_reparent_into_next: data.can_reparent_into_next,
        can_reparent_outdent: data.can_reparent_outdent,
        can_open_reference: data.can_open_reference,
        can_convert_to_reference: data.can_convert_to_reference,
        can_extract_component: data.can_extract_component,
        can_promote_to_external_widget: data.can_promote_to_external_widget,
        can_wrap_in_vertical_box: data.can_wrap_in_vertical_box,
        can_unwrap: data.can_unwrap,
        can_create_rule: data.can_create_rule,
        can_extract_rule: data.can_extract_rule,
    }
}
