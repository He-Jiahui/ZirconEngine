use super::backend_refresh::plan_asset_backend_refresh;
use super::*;

impl SlintEditorHost {
    pub(super) fn refresh_project_assets(&mut self) -> Result<(), String> {
        let mut changes = Vec::new();
        while let Ok(change) = self.asset_change_events.try_recv() {
            changes.push(change);
        }
        if !changes.is_empty() {
            self.editor_asset_server
                .refresh_from_runtime_project()
                .map_err(|error| error.to_string())?;
        }
        let mut editor_changes = Vec::new();
        while let Ok(change) = self.editor_asset_change_events.try_recv() {
            editor_changes.push(change);
        }
        let mut resource_changes = Vec::new();
        while let Ok(change) = self.resource_change_events.try_recv() {
            resource_changes.push(change);
        }
        if changes.is_empty() && editor_changes.is_empty() && resource_changes.is_empty() {
            return Ok(());
        }

        let selected_asset_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        let default_scene_uri = self
            .asset_server
            .current_project()
            .map(|project| project.default_scene_uri);
        let plan = plan_asset_backend_refresh(
            selected_asset_uuid.as_deref(),
            default_scene_uri.as_deref(),
            &changes,
            &editor_changes,
            &resource_changes,
        );

        if plan.sync_catalog {
            self.sync_asset_catalog();
        }
        if plan.sync_resources {
            self.sync_asset_resources();
        }
        if plan.refresh_selected_asset_details {
            self.refresh_selected_asset_details();
        }
        if plan.refresh_visible_asset_previews {
            self.refresh_visible_asset_previews();
        }
        if plan.reload_default_scene {
            self.reload_default_scene()?;
        }
        let mut invalidation = HostInvalidationMask::NONE;
        if plan.mark_render_dirty {
            invalidation.insert(HostInvalidationMask::RENDER);
        }
        if plan.mark_presentation_dirty {
            invalidation.insert(HostInvalidationMask::PRESENTATION_DATA);
        }
        self.invalidate_host(invalidation);

        Ok(())
    }

    pub(super) fn reload_default_scene(&mut self) -> Result<(), String> {
        let project_info = self
            .asset_server
            .current_project()
            .ok_or_else(|| "no directory project is currently open".to_string())?;
        let mut project =
            ProjectManager::open(&project_info.root_path).map_err(|error| error.to_string())?;
        project
            .scan_and_import()
            .map_err(|error| error.to_string())?;
        let scene_uri = ResourceLocator::parse(&project_info.default_scene_uri)
            .map_err(|error| error.to_string())?;
        let world =
            Scene::load_scene_from_uri(&project, &scene_uri).map_err(|error| error.to_string())?;
        let level = self
            .editor_manager
            .create_runtime_level(world)
            .map_err(|error| error.to_string())?;
        self.runtime.replace_world(level, project_info.root_path);
        Ok(())
    }

    pub(super) fn import_model_into_project(&mut self) -> Result<(), String> {
        let chrome = self.build_chrome();
        let project = self
            .asset_server
            .current_project()
            .ok_or_else(|| "Open a project before importing models".to_string())?;
        EditorProjectDocument::ensure_runtime_assets(&project.root_path)
            .map_err(|error| error.to_string())?;

        let source = canonical_model_source_path(&chrome.mesh_import_path)?;
        let paths =
            ProjectPaths::from_root(&project.root_path).map_err(|error| error.to_string())?;
        let (model_uri, display_path) = stage_model_source(&paths, &source)?;

        self.asset_server
            .import_asset(&model_uri.to_string())
            .map_err(|error| error.to_string())?;
        for derived_uri in derive_animation_assets_from_model_source(
            paths.assets_root(),
            std::path::Path::new(&display_path),
        )? {
            self.asset_server
                .import_asset(&derived_uri.to_string())
                .map_err(|error| error.to_string())?;
        }
        let material_id = self.default_project_material_id()?;
        self.sync_asset_workspace();
        let model_id =
            resolve_ready_handle::<ModelMarker>(self.resource_server.as_ref(), &model_uri)?;
        if self
            .runtime
            .import_mesh_asset(model_id, material_id, display_path)?
        {
            self.mark_render_and_presentation_dirty();
        } else {
            self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
        }
        Ok(())
    }

    pub(super) fn default_project_material_id(
        &self,
    ) -> Result<ResourceHandle<MaterialMarker>, String> {
        let material_uri = ResourceLocator::parse("res://materials/default.material.toml")
            .map_err(|error| error.to_string())?;
        self.asset_server
            .import_asset(&material_uri.to_string())
            .map_err(|error| error.to_string())?;
        resolve_ready_handle::<MaterialMarker>(self.resource_server.as_ref(), &material_uri)
    }

    pub(super) fn sync_asset_workspace(&mut self) {
        let _ = self.editor_asset_server.refresh_from_runtime_project();
        self.sync_asset_catalog();
        self.sync_asset_resources();
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
    }

    pub(super) fn sync_asset_catalog(&mut self) {
        self.runtime
            .sync_asset_catalog(self.editor_asset_server.catalog_snapshot());
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(super) fn sync_asset_resources(&mut self) {
        self.runtime
            .sync_asset_resources(self.resource_server.list_resources());
        self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
    }

    pub(super) fn refresh_selected_asset_details(&mut self) {
        let selected_uuid = self
            .runtime
            .editor_snapshot()
            .asset_activity
            .selected_asset_uuid;
        self.runtime.sync_asset_details(
            selected_uuid
                .as_deref()
                .and_then(|uuid| self.editor_asset_server.asset_details(uuid)),
        );
    }

    pub(super) fn refresh_visible_asset_previews(&mut self) {
        if self.asset_server.current_project().is_none() {
            return;
        }

        let chrome = self.build_chrome();
        let mut visible = BTreeSet::new();

        if asset_surface_visible(&chrome, ViewContentKind::Assets) {
            visible.extend(
                chrome
                    .asset_activity
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_activity.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        if asset_surface_visible(&chrome, ViewContentKind::AssetBrowser) {
            visible.extend(
                chrome
                    .asset_browser
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_browser.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        for uuid in visible {
            let _ = self
                .editor_asset_server
                .request_preview_refresh(&uuid, true);
        }
    }

    pub(super) fn dispatch_asset_control_changed(
        &mut self,
        source: &str,
        control_id: &str,
        value: &str,
    ) {
        let arguments = match control_id {
            "SearchEdited" | "SetKindFilter" => vec![UiBindingValue::string(value)],
            "SetViewMode" | "SetUtilityTab" => vec![
                UiBindingValue::string(source),
                UiBindingValue::string(value),
            ],
            _ => {
                self.set_status_line(format!("Unknown asset change control {control_id}"));
                return;
            }
        };
        self.dispatch_asset_surface_control(control_id, UiEventKind::Change, arguments);
    }

    pub(super) fn dispatch_asset_control_clicked(&mut self, _source: &str, control_id: &str) {
        match control_id {
            "OpenAssetBrowser" | "LocateSelectedAsset" | "ImportModel" => {
                self.dispatch_asset_surface_control(control_id, UiEventKind::Click, Vec::new());
            }
            _ => {
                self.set_status_line(format!("Unknown asset click control {control_id}"));
            }
        }
    }

    pub(super) fn dispatch_asset_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        self.focus_callback_source_window();
        let Some(result) = callback_dispatch::dispatch_builtin_asset_surface_control(
            &self.runtime,
            &self.asset_surface_bridge,
            control_id,
            event_kind,
            arguments,
        ) else {
            self.set_status_line(format!("Unknown asset surface control {control_id}"));
            return;
        };
        self.apply_dispatch_result(result);
    }
}
