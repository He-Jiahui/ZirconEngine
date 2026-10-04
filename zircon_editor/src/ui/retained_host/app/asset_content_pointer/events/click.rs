use super::super::super::{callback_dispatch, RetainedEditorHost, UiPoint};
use crate::core::editor_event::{EditorAssetEvent, EditorEvent, EditorEventSource};
use crate::ui::retained_host::event_bridge::{apply_record_effects, UiHostEventEffects};
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;
use std::time::Instant;
use zircon_runtime::asset::AssetUri;
use zircon_runtime_interface::resource::ResourceKind;

impl RetainedEditorHost {
    fn clear_pending_asset_activation_click(&mut self, surface_mode: &str) {
        match surface_mode {
            "activity" => self.activity_asset_pointer.activation_clicks.clear(),
            "browser" => self.browser_asset_pointer.activation_clicks.clear(),
            _ => {}
        }
    }

    pub(in crate::ui::retained_host::app) fn asset_content_pointer_clicked(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.use_committed_pointer_layout();
        self.focus_callback_source_window();
        let Some(target) =
            self.prepare_asset_content_pointer_target(surface_mode, width, height, false)
        else {
            self.clear_pending_asset_activation_click(surface_mode);
            return;
        };
        if !self.sync_prepared_asset_content_pointer_list(surface_mode, &target, false) {
            self.clear_pending_asset_activation_click(surface_mode);
            return;
        }

        if !self.ensure_asset_surface_bridge() {
            self.clear_pending_asset_activation_click(surface_mode);
            return;
        }
        let Some(bridge) = self.asset_surface_bridge.as_ref() else {
            self.clear_pending_asset_activation_click(surface_mode);
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let runtime = &self.runtime;
        let point = UiPoint::new(x, y);
        let dispatch = match surface_mode {
            "activity" => callback_dispatch::dispatch_shared_asset_content_pointer_click(
                runtime,
                bridge,
                &mut self.activity_asset_pointer.content_bridge,
                point,
            ),
            "browser" => callback_dispatch::dispatch_shared_asset_content_pointer_click(
                runtime,
                bridge,
                &mut self.browser_asset_pointer.content_bridge,
                point,
            ),
            _ => {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            }
        };

        match dispatch {
            Ok(dispatch) => {
                let activated_uuid =
                    dispatch.pointer.route.as_ref().and_then(|route| {
                        match route {
                    crate::ui::retained_host::asset_pointer::AssetPointerContentRoute::Item {
                        asset_uuid, ..
                    } => Some(asset_uuid.clone()),
                    _ => None,
                }
                    });
                let activation_asset = activated_uuid
                    .as_deref()
                    .and_then(|uuid| target.snapshot.visible_assets.selected_index(uuid))
                    .and_then(|index| target.snapshot.visible_assets.get(index));
                let asset_locator = activation_asset.map(|asset| asset.locator.as_str());
                let resource_revision = activation_asset.and_then(|asset| asset.resource_revision);
                let activation_candidate = match surface_mode {
                    "activity" => self
                        .activity_asset_pointer
                        .activation_clicks
                        .register_click(
                            activated_uuid.as_deref(),
                            asset_locator,
                            resource_revision,
                            &target.snapshot.visible_assets,
                            target.snapshot.catalog_revision,
                            target.snapshot.view_mode,
                            Instant::now(),
                        ),
                    "browser" => self.browser_asset_pointer.activation_clicks.register_click(
                        activated_uuid.as_deref(),
                        asset_locator,
                        resource_revision,
                        &target.snapshot.visible_assets,
                        target.snapshot.catalog_revision,
                        target.snapshot.view_mode,
                        Instant::now(),
                    ),
                    _ => false,
                };
                self.write_asset_content_pointer_state(surface_mode, dispatch.pointer.state);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
                if surface_mode == "browser" && activation_candidate {
                    if let Some(asset_uuid) = activated_uuid {
                        self.open_double_clicked_asset(&target.snapshot, &asset_uuid);
                    }
                }
            }
            Err(error) => {
                self.clear_pending_asset_activation_click(surface_mode);
                self.set_status_line(error);
            }
        }
    }

    fn open_double_clicked_asset(&mut self, snapshot: &AssetWorkspaceSnapshot, asset_uuid: &str) {
        let Some(index) = snapshot.visible_assets.selected_index(asset_uuid) else {
            let status = self
                .runtime
                .context()
                .i18n()
                .translate("asset.activation.target_unavailable");
            self.set_status_line(status.as_ref());
            return;
        };
        let Some(asset) = snapshot.visible_assets.get(index) else {
            let status = self
                .runtime
                .context()
                .i18n()
                .translate("asset.activation.target_unavailable");
            self.set_status_line(status.as_ref());
            return;
        };
        let asset_locator = asset.locator.clone();
        let asset_kind = asset.kind;
        let asset_revision = asset.resource_revision;
        let current = self.runtime.editor_snapshot().asset_browser;
        if current.catalog_revision != snapshot.catalog_revision
            || current.view_mode != snapshot.view_mode
            || !current
                .visible_assets
                .shares_item_identity_with(&snapshot.visible_assets)
            || current
                .visible_assets
                .selected_index(asset_uuid)
                .and_then(|index| {
                    current.visible_assets.get(index).map(|current| {
                        current.locator == asset_locator
                            && current.resource_revision == asset_revision
                    })
                })
                != Some(true)
        {
            let status = self
                .runtime
                .context()
                .i18n()
                .translate("asset.activation.target_changed");
            self.set_status_line(status.as_ref());
            return;
        }

        if asset_kind == ResourceKind::Scene {
            let result = AssetUri::parse(&asset_locator)
                .map_err(|error| error.to_string())
                .and_then(|scene_uri| self.open_startup_scene(scene_uri));
            match result {
                Ok(()) => self.mark_render_and_presentation_dirty(),
                Err(error) => self.set_status_line(error),
            }
            return;
        }

        match self.runtime.dispatch_event(
            EditorEventSource::RetainedHost,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.clone(),
            }),
        ) {
            Ok(record) => {
                let mut effects = UiHostEventEffects::default();
                apply_record_effects(&mut effects, &record);
                self.apply_dispatch_effects(effects);
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
