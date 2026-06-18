use super::*;

mod refresh;

impl RetainedEditorHost {
    pub(super) fn ensure_asset_surface_bridge(&mut self) -> bool {
        if self.asset_surface_bridge.is_some() {
            return true;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "lazy_asset_surface_bridge");
        match callback_dispatch::BuiltinAssetSurfaceTemplateBridge::new_minimal() {
            Ok(bridge) => {
                self.asset_surface_bridge = Some(bridge);
                true
            }
            Err(error) => {
                self.set_status_line(format!("Failed to load asset UI controls: {error}"));
                false
            }
        }
    }

    pub(super) fn reload_default_scene(&mut self) -> Result<(), String> {
        let project_info = self
            .asset_manager
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
            .asset_manager
            .current_project()
            .ok_or_else(|| "Open a project before importing models".to_string())?;
        EditorProjectDocument::ensure_runtime_assets(&project.root_path)
            .map_err(|error| error.to_string())?;

        let source = canonical_model_source_path(&chrome.mesh_import_path)?;
        let paths =
            ProjectPaths::from_root(&project.root_path).map_err(|error| error.to_string())?;
        let (model_uri, display_path) = stage_model_source(&paths, &source)?;

        self.asset_manager
            .import_asset(&model_uri.to_string())
            .map_err(|error| error.to_string())?;
        for derived_uri in derive_animation_assets_from_model_source(
            paths.assets_root(),
            std::path::Path::new(&display_path),
        )? {
            self.asset_manager
                .import_asset(&derived_uri.to_string())
                .map_err(|error| error.to_string())?;
        }
        let material_id = self.default_project_material_id()?;
        self.sync_asset_workspace();
        let model_id =
            resolve_ready_handle::<ModelMarker>(self.resource_manager.as_ref(), &model_uri)?;
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
        let material_uri = ResourceLocator::parse("res://materials/default.zmaterial")
            .map_err(|error| error.to_string())?;
        self.asset_manager
            .import_asset(&material_uri.to_string())
            .map_err(|error| error.to_string())?;
        resolve_ready_handle::<MaterialMarker>(self.resource_manager.as_ref(), &material_uri)
    }

    pub(super) fn sync_asset_workspace(&mut self) {
        let _ = self.editor_asset_manager.refresh_from_runtime_project();
        self.sync_asset_catalog();
        self.sync_asset_resources();
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
    }

    pub(super) fn dispatch_asset_control_changed(
        &mut self,
        source: &str,
        control_id: &str,
        value: &str,
    ) {
        let Some(binding_control_id) = asset_surface_binding_control_id(control_id) else {
            self.set_status_line(format!("Unknown asset change control {control_id}"));
            return;
        };
        let arguments = match binding_control_id {
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
        self.dispatch_asset_surface_control(binding_control_id, UiEventKind::Change, arguments);
    }

    pub(super) fn dispatch_asset_control_clicked(&mut self, _source: &str, control_id: &str) {
        let Some(binding_control_id) = asset_surface_binding_control_id(control_id) else {
            self.set_status_line(format!("Unknown asset click control {control_id}"));
            return;
        };
        match binding_control_id {
            "OpenAssetBrowser" | "LocateSelectedAsset" | "ImportModel" => {
                self.dispatch_asset_surface_control(
                    binding_control_id,
                    UiEventKind::Click,
                    Vec::new(),
                );
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
        if !self.ensure_asset_surface_bridge() {
            return;
        }
        let Some(asset_surface_bridge) = self.asset_surface_bridge.as_ref() else {
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let Some(result) = callback_dispatch::dispatch_builtin_asset_surface_control(
            &self.runtime,
            asset_surface_bridge,
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

fn asset_surface_binding_control_id(action_id: &str) -> Option<&'static str> {
    match action_id {
        "SearchEdited" | "workbench.asset.search.edit" => Some("SearchEdited"),
        "SetKindFilter" | "workbench.asset.kind_filter.set" => Some("SetKindFilter"),
        "SetViewMode" | "workbench.asset.view_mode.set" => Some("SetViewMode"),
        "SetUtilityTab" | "workbench.asset.utility_tab.set" => Some("SetUtilityTab"),
        "OpenAssetBrowser" | "workbench.asset_browser.open" => Some("OpenAssetBrowser"),
        "LocateSelectedAsset" | "workbench.asset.locate_selected" => Some("LocateSelectedAsset"),
        "ImportModel" | "workbench.asset.model.import" => Some("ImportModel"),
        _ => None,
    }
}
