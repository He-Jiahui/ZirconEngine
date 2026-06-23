use std::collections::BTreeMap;

use crate::ui::workbench::layout::{MainPageId, WorkbenchLayout};
use crate::ui::workbench::project::{
    list_layout_preset_assets, load_layout_preset_asset, save_layout_preset_asset,
};
use crate::ui::workbench::{
    LayoutPresetName, LayoutPresetPersistenceStore, LayoutPresetRestoreResult, LayoutPresetScope,
};

use super::editor_error::EditorError;
use super::editor_ui_host::EditorUiHost;

const DEFAULT_LAYOUT_KEY: &str = "editor.workbench.default_layout";
const PRESET_LAYOUTS_KEY: &str = "editor.workbench.presets";
const PAGE_USER_LAYOUTS_KEY: &str = "editor.workbench.page_user_layouts";

impl EditorUiHost {
    pub(super) fn save_global_default_layout(&self) -> Result<(), EditorError> {
        let layout = self.current_layout();
        let config = self.config_manager()?;
        config
            .set_value(
                DEFAULT_LAYOUT_KEY,
                serde_json::to_value(layout)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn preset_names(&self) -> Result<Vec<String>, EditorError> {
        let mut names = Vec::new();
        if let Some(project_root) = self.current_project_root()? {
            names.extend(
                list_layout_preset_assets(&project_root)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            );
        }
        names.extend(self.load_presets()?.into_keys());
        names.sort();
        names.dedup();
        Ok(names)
    }

    pub(super) fn load_global_default_layout(&self) -> Option<WorkbenchLayout> {
        let config = self.config_manager().ok()?;
        let value = config.get_value(DEFAULT_LAYOUT_KEY)?;
        serde_json::from_value(value).ok()
    }

    pub(super) fn save_page_layout(
        &self,
        user_id: &str,
        page_id: &MainPageId,
    ) -> Result<(), EditorError> {
        let layout = self.current_layout();
        let mut store = self.load_page_layout_store()?;
        store.persist_layout_snapshot(
            LayoutPresetScope::new(user_id.to_string(), page_id.clone()),
            LayoutPresetName::Authoring,
            &layout,
        );
        self.save_page_layout_store(store)
    }

    pub(super) fn restore_page_layout(
        &self,
        user_id: &str,
        page_id: &MainPageId,
    ) -> Result<LayoutPresetRestoreResult, EditorError> {
        let store = self.load_page_layout_store()?;
        let scope = LayoutPresetScope::new(user_id.to_string(), page_id.clone());
        let mut session = self.lock_session();
        let restored = store.restore_into_layout(&scope, &mut session.layout);
        self.recompute_session_metadata(&mut session);
        Ok(restored)
    }

    pub(super) fn save_preset(&self, name: &str) -> Result<(), EditorError> {
        if let Some(project_root) = self.current_project_root()? {
            save_layout_preset_asset(&project_root, name, &self.current_layout())
                .map_err(|error| EditorError::Project(error.to_string()))?;
            return Ok(());
        }
        let mut presets = self.load_presets()?;
        presets.insert(name.to_string(), self.current_layout());
        self.config_manager()?
            .set_value(
                PRESET_LAYOUTS_KEY,
                serde_json::to_value(presets)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn load_preset(&self, name: &str) -> Result<bool, EditorError> {
        if let Some(project_root) = self.current_project_root()? {
            if let Some(layout) = load_layout_preset_asset(&project_root, name)
                .map_err(|error| EditorError::Project(error.to_string()))?
            {
                let mut session = self.lock_session();
                session.layout = layout;
                self.recompute_session_metadata(&mut session);
                return Ok(true);
            }
        }
        let presets = self.load_presets()?;
        let layout = presets
            .get(name)
            .cloned()
            .ok_or_else(|| EditorError::Layout(format!("missing preset {name}")))?;
        let mut session = self.lock_session();
        session.layout = layout;
        self.recompute_session_metadata(&mut session);
        Ok(true)
    }

    fn load_presets(&self) -> Result<BTreeMap<String, WorkbenchLayout>, EditorError> {
        let Some(value) = self.config_manager()?.get_value(PRESET_LAYOUTS_KEY) else {
            return Ok(BTreeMap::new());
        };
        serde_json::from_value(value).map_err(|error| EditorError::Project(error.to_string()))
    }

    fn load_page_layout_store(&self) -> Result<LayoutPresetPersistenceStore, EditorError> {
        let Some(value) = self.config_manager()?.get_value(PAGE_USER_LAYOUTS_KEY) else {
            return Ok(LayoutPresetPersistenceStore::default());
        };
        serde_json::from_value(value).map_err(|error| EditorError::Project(error.to_string()))
    }

    fn save_page_layout_store(
        &self,
        store: LayoutPresetPersistenceStore,
    ) -> Result<(), EditorError> {
        self.config_manager()?
            .set_value(
                PAGE_USER_LAYOUTS_KEY,
                serde_json::to_value(store)
                    .map_err(|error| EditorError::Project(error.to_string()))?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }
}
