use super::super::{AssetPointerContentRoute, RetainedEditorHost, UiPoint};
use crate::ui::retained_host::WorkbenchContextMenuRequestData;

const ASSET_DELETE_ACTION_ID: &str = "menu.item.asset.delete";

impl RetainedEditorHost {
    pub(super) fn dispatch_asset_content_context_menu(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        popup_anchor_x: f32,
        popup_anchor_y: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            return;
        };
        let point = UiPoint::new(x, y);
        let Some(route) = self
            .dispatch_prepared_asset_content_pointer(surface_mode, &target, false, |bridge| {
                bridge.route_at(point)
            })
            .flatten()
        else {
            return;
        };
        let AssetPointerContentRoute::Item { asset_uuid, .. } = route else {
            return;
        };
        let Some(index) = target.snapshot.visible_assets.selected_index(&asset_uuid) else {
            self.set_status_line(format!(
                "Asset context target {asset_uuid} is no longer in the current catalog generation"
            ));
            return;
        };
        let Some(asset) = target.snapshot.visible_assets.get(index) else {
            self.set_status_line(format!(
                "Asset context target {asset_uuid} is no longer in the current catalog generation"
            ));
            return;
        };

        self.dispatch_workbench_context_menu_requested(WorkbenchContextMenuRequestData {
            target_control_id: format!("AssetContent:{surface_mode}").into(),
            target_action_id: ASSET_DELETE_ACTION_ID.into(),
            target_dispatch_kind: "asset".into(),
            target_role: "asset-row".into(),
            target_value_text: asset.display_name.clone().into(),
            target_path: format!("workbench://asset/{asset_uuid}").into(),
            popup_anchor_x,
            popup_anchor_y,
            menu_items: vec![
                format!("Delete|action={ASSET_DELETE_ACTION_ID},danger,icon=trash").into(),
            ],
        });
    }
}
