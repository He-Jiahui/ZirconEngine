use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn reload_default_scene(&mut self) -> Result<(), String> {
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

    pub(in crate::ui::retained_host::app) fn import_model_into_project(
        &mut self,
    ) -> Result<(), String> {
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

    pub(in crate::ui::retained_host::app) fn default_project_material_id(
        &self,
    ) -> Result<ResourceHandle<MaterialMarker>, String> {
        let material_uri = ResourceLocator::parse("res://materials/default.zmaterial")
            .map_err(|error| error.to_string())?;
        self.asset_manager
            .import_asset(&material_uri.to_string())
            .map_err(|error| error.to_string())?;
        resolve_ready_handle::<MaterialMarker>(self.resource_manager.as_ref(), &material_uri)
    }

    pub(in crate::ui::retained_host::app) fn sync_asset_workspace(&mut self) {
        let _ = self.editor_asset_manager.refresh_from_runtime_project();
        self.sync_asset_catalog();
        self.sync_asset_resources();
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
    }
}
