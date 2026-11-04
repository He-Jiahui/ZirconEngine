use super::super::{AssetPointerContentRoute, RetainedEditorHost, UiPoint};
use crate::ui::retained_host::WorkbenchContextMenuRequestData;
use zircon_runtime::asset::AssetUri;
use zircon_runtime_interface::resource::ResourceScheme;

const ASSET_OPEN_ACTION_ID: &str = "menu.item.asset.open";
const ASSET_DELETE_ACTION_ID: &str = "menu.item.asset.delete";
const ASSET_RENAME_ACTION_ID: &str = "menu.item.asset.rename";
const ASSET_REIMPORT_ACTION_ID: &str = "menu.item.asset.reimport";
const FOLDER_CREATE_ACTION_ID: &str = "menu.item.asset.folder.create";
const FOLDER_OPEN_ACTION_ID: &str = "menu.item.asset.folder.open";
const FOLDER_RENAME_ACTION_ID: &str = "menu.item.asset.folder.rename";
const FOLDER_DELETE_ACTION_ID: &str = "menu.item.asset.folder.delete";

fn encode_asset_locator_for_context_path(locator: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(locator.len() * 2);
    for byte in locator.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

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
        let asset_uuid = match route {
            AssetPointerContentRoute::Item { asset_uuid, .. } => asset_uuid,
            AssetPointerContentRoute::Folder { folder_id, .. } => {
                self.open_asset_folder_context_menu(
                    surface_mode,
                    &folder_id,
                    true,
                    popup_anchor_x,
                    popup_anchor_y,
                );
                return;
            }
            AssetPointerContentRoute::ContentSurface => {
                let current = match self.runtime.editor_snapshot() {
                    Ok(snapshot) => snapshot,
                    Err(error) => {
                        self.set_status_line(error.to_string());
                        return;
                    }
                };
                let workspace = match surface_mode {
                    "activity" => current.asset_activity,
                    "browser" => current.asset_browser,
                    _ => return,
                };
                if let Some(folder_id) = workspace.selected_folder_id {
                    self.open_asset_folder_context_menu(
                        surface_mode,
                        &folder_id,
                        false,
                        popup_anchor_x,
                        popup_anchor_y,
                    );
                }
                return;
            }
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

        let open_label = self
            .runtime
            .context()
            .i18n()
            .translate("menu.assets.open.label");
        let i18n = self.runtime.context().i18n();
        let source = AssetUri::parse(&asset.locator).ok();
        let project_asset = source
            .as_ref()
            .is_some_and(|uri| uri.scheme() == ResourceScheme::Res);
        let writable_source =
            project_asset && source.as_ref().is_some_and(|uri| uri.label().is_none());
        let mut menu_items =
            vec![format!("{open_label}|action={ASSET_OPEN_ACTION_ID},icon=folder").into()];
        if writable_source {
            menu_items.push(
                format!(
                    "{}|action={ASSET_RENAME_ACTION_ID},icon=edit",
                    i18n.translate("menu.assets.rename.label")
                )
                .into(),
            );
        }
        if project_asset {
            menu_items.push(
                format!(
                    "{}|action={ASSET_REIMPORT_ACTION_ID},icon=refresh-cw",
                    i18n.translate("menu.assets.reimport.label")
                )
                .into(),
            );
        }
        if writable_source {
            menu_items.extend([
                "---".into(),
                format!(
                    "{}|action={FOLDER_CREATE_ACTION_ID},icon=folder-plus",
                    i18n.translate("menu.assets.new_folder.label")
                )
                .into(),
                format!(
                    "{}|action={ASSET_DELETE_ACTION_ID},danger,icon=trash",
                    i18n.translate("menu.assets.delete.label")
                )
                .into(),
            ]);
        }
        let encoded_locator = encode_asset_locator_for_context_path(&asset.locator);
        self.dispatch_workbench_context_menu_requested(WorkbenchContextMenuRequestData {
            target_control_id: format!("AssetContent:{surface_mode}").into(),
            target_action_id: ASSET_OPEN_ACTION_ID.into(),
            target_dispatch_kind: "asset".into(),
            target_role: "asset-row".into(),
            target_value_text: asset.display_name.clone().into(),
            target_path: format!("workbench://asset/{asset_uuid}?open_locator={encoded_locator}")
                .into(),
            popup_anchor_x,
            popup_anchor_y,
            menu_items,
        });
    }

    /// Both folder rows and the background retain their URI, independently of selection.
    pub(in crate::ui::retained_host::app) fn open_asset_folder_context_menu(
        &mut self,
        surface_mode: &str,
        folder_id: &str,
        include_folder_actions: bool,
        popup_anchor_x: f32,
        popup_anchor_y: f32,
    ) {
        let current = match self.runtime.editor_snapshot() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        };
        let workspace = match surface_mode {
            "activity" => current.asset_activity,
            "browser" => current.asset_browser,
            _ => return,
        };
        let Some(folder) = workspace
            .folder_tree
            .iter()
            .find(|folder| folder.folder_id == folder_id)
        else {
            let status = self
                .runtime
                .context()
                .i18n()
                .translate("asset.name_prompt.target_changed");
            self.set_status_line(status.as_ref());
            return;
        };
        let is_root = folder_id == "res://";
        if !is_root {
            let Ok(uri) = AssetUri::parse(folder_id) else {
                return;
            };
            if uri.scheme() != ResourceScheme::Res || uri.label().is_some() {
                return;
            }
        }
        let i18n = self.runtime.context().i18n();
        let mut menu_items = vec![format!(
            "{}|action={FOLDER_CREATE_ACTION_ID},icon=folder-plus",
            i18n.translate("menu.assets.new_folder.label")
        )
        .into()];
        if include_folder_actions {
            menu_items.insert(
                0,
                format!(
                    "{}|action={FOLDER_OPEN_ACTION_ID},icon=folder",
                    i18n.translate("menu.assets.open.label")
                )
                .into(),
            );
            if !is_root {
                menu_items.extend([
                    format!(
                        "{}|action={FOLDER_RENAME_ACTION_ID},icon=edit",
                        i18n.translate("menu.assets.rename.label")
                    )
                    .into(),
                    "---".into(),
                    format!(
                        "{}|action={FOLDER_DELETE_ACTION_ID},danger,icon=trash",
                        i18n.translate("menu.assets.delete_empty_folder.label")
                    )
                    .into(),
                ]);
            }
        }
        let encoded_folder = encode_asset_locator_for_context_path(folder_id);
        self.dispatch_workbench_context_menu_requested(WorkbenchContextMenuRequestData {
            target_control_id: format!("AssetFolder:{surface_mode}").into(),
            target_action_id: FOLDER_CREATE_ACTION_ID.into(),
            target_dispatch_kind: "asset".into(),
            target_role: "folder-row".into(),
            target_value_text: folder.display_name.clone().into(),
            target_path: format!("workbench://asset-folder/{encoded_folder}").into(),
            popup_anchor_x,
            popup_anchor_y,
            menu_items,
        });
    }
}
