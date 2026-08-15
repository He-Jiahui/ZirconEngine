use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn reload_default_scene(&mut self) -> Result<(), String> {
        let asset_manager = self
            .asset_manager_at_use_point()
            .map_err(|error| error.to_string())?;
        let project_info = asset_manager
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
        let authoring_world = self
            .editor_manager
            .prepare_authoring_world(world)
            .map_err(|error| error.to_string())?;
        self.runtime
            .replace_world(authoring_world, project_info.root_path)?;
        Ok(())
    }

    pub(in crate::ui::retained_host::app) fn import_model_into_project(
        &mut self,
    ) -> Result<(), String> {
        let chrome = self.build_chrome();
        let asset_manager = self
            .asset_manager_at_use_point()
            .map_err(|error| error.to_string())?;
        let project = asset_manager
            .current_project()
            .ok_or_else(|| "Open a project before importing models".to_string())?;
        let source = canonical_model_source_path(&chrome.mesh_import_path)?;
        let project_manager =
            ProjectManager::open(&project.root_path).map_err(|error| error.to_string())?;
        let (model_uri, display_path) = stage_model_source(&project_manager, &source)?;

        asset_manager
            .import_asset(&model_uri.to_string())
            .map_err(|error| error.to_string())?;
        for derived_uri in derive_animation_assets_from_model_source(
            &project_manager,
            std::path::Path::new(&display_path),
        )? {
            asset_manager
                .import_asset(&derived_uri.to_string())
                .map_err(|error| error.to_string())?;
        }
        let material_id = self.default_project_material_id()?;
        self.sync_asset_workspace();
        let resource_manager = self
            .resolve_resource_manager()
            .map_err(|error| error.to_string())?;
        let model_id = resolve_ready_handle::<ModelMarker>(resource_manager.as_ref(), &model_uri)?;
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

    pub(in crate::ui::retained_host::app) fn default_project_material_id(
        &self,
    ) -> Result<ResourceHandle<MaterialMarker>, String> {
        let material_uri = ResourceLocator::parse("res://materials/default.zmaterial")
            .map_err(|error| error.to_string())?;
        self.asset_manager_at_use_point()
            .map_err(|error| error.to_string())?
            .import_asset(&material_uri.to_string())
            .map_err(|error| error.to_string())?;
        let resource_manager = self
            .resolve_resource_manager()
            .map_err(|error| error.to_string())?;
        resolve_ready_handle::<MaterialMarker>(resource_manager.as_ref(), &material_uri)
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_workspace(&mut self) {
        if let Ok(editor_asset_manager) = self.editor_asset_manager_at_use_point() {
            let _ = editor_asset_manager.refresh_from_runtime_project();
        }
        self.sync_asset_catalog();
        self.sync_asset_resources();
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
    }
}
