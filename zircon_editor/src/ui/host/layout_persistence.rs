use std::collections::BTreeMap;

use crate::ui::workbench::layout::{MainPageId, WorkbenchLayout};
use crate::ui::workbench::layout_persistence_document::{
    decode_default_layout_value, decode_named_layout_presets_value,
    decode_page_layout_presets_value, encode_default_layout_value,
    encode_named_layout_presets_value, encode_page_layout_presets_value,
};
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
            .set_value(DEFAULT_LAYOUT_KEY, encode_default_layout_value(layout)?)
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn preset_names(&self) -> Result<Vec<String>, EditorError> {
        let mut names = Vec::new();
        names.extend(list_layout_preset_assets(
            self.asset_manager()?.current_project_asset_uris(),
        ));
        names.extend(self.load_presets()?.into_keys());
        names.sort_unstable();
        names.dedup();
        Ok(names)
    }

    pub(super) fn load_global_default_layout(&self) -> Option<WorkbenchLayout> {
        let config = self.config_manager().ok()?;
        let value = config.get_value(DEFAULT_LAYOUT_KEY)?;
        match decode_default_layout_value(value) {
            Ok(layout) => Some(layout),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "discarding an invalid global workbench layout and restoring the builtin layout"
                );
                None
            }
        }
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
        if let Some(project) = self.current_project_snapshot()? {
            let path = save_layout_preset_asset(&project, name, &self.current_layout())?;
            let locator = project.project_uri_for_source_path(&path)?;
            let _ = self.asset_manager()?.import_asset(&locator.to_string())?;
            return Ok(());
        }
        let mut presets = self.load_presets()?;
        presets.insert(name.to_string(), self.current_layout());
        self.config_manager()?
            .set_value(
                PRESET_LAYOUTS_KEY,
                encode_named_layout_presets_value(presets)?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }

    pub(super) fn load_preset(&self, name: &str) -> Result<bool, EditorError> {
        if let Some(project) = self.current_project_snapshot()? {
            if let Some(layout) = load_layout_preset_asset(&project, name)? {
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
        match decode_named_layout_presets_value(value) {
            Ok(presets) => Ok(presets),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "discarding invalid named workbench presets"
                );
                Ok(BTreeMap::new())
            }
        }
    }

    fn load_page_layout_store(&self) -> Result<LayoutPresetPersistenceStore, EditorError> {
        let Some(value) = self.config_manager()?.get_value(PAGE_USER_LAYOUTS_KEY) else {
            return Ok(LayoutPresetPersistenceStore::default());
        };
        match decode_page_layout_presets_value(value) {
            Ok(store) => Ok(store),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "discarding invalid user/page workbench presets"
                );
                Ok(LayoutPresetPersistenceStore::default())
            }
        }
    }

    fn save_page_layout_store(
        &self,
        store: LayoutPresetPersistenceStore,
    ) -> Result<(), EditorError> {
        self.config_manager()?
            .set_value(
                PAGE_USER_LAYOUTS_KEY,
                encode_page_layout_presets_value(store)?,
            )
            .map_err(|error| EditorError::Project(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io;

    use zircon_runtime::scene::world::SceneProjectError;

    use super::EditorError;

    #[test]
    fn scene_project_conversion_preserves_the_typed_source_chain() {
        let error: EditorError =
            SceneProjectError::from(io::Error::other("layout preset source")).into();
        let source = error
            .source()
            .expect("EditorError should expose its source");

        assert!(source.downcast_ref::<SceneProjectError>().is_some());
        assert!(source.source().is_some());
    }
}
