use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crossbeam_channel::{unbounded, Receiver};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;

pub(crate) struct UiAssetWorkspaceWatcher {
    assets_root: PathBuf,
    receiver: Receiver<PathBuf>,
    _watcher: RecommendedWatcher,
}

impl UiAssetWorkspaceWatcher {
    pub(crate) fn start(project_root: PathBuf) -> Result<Self, EditorError> {
        let assets_root = project_root.join("assets");
        let (sender, receiver) = unbounded::<PathBuf>();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                let Ok(event) = event else {
                    return;
                };
                for path in event.paths {
                    let _ = sender.send(path);
                }
            })
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        watcher
            .watch(&assets_root, RecursiveMode::Recursive)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        Ok(Self {
            assets_root,
            receiver,
            _watcher: watcher,
        })
    }

    pub(crate) fn drain_changed_asset_ids(&self) -> Vec<String> {
        let mut asset_ids = BTreeSet::new();
        while let Ok(path) = self.receiver.try_recv() {
            if let Some(asset_id) = self.asset_id_for_path(&path) {
                let _ = asset_ids.insert(asset_id);
            }
        }
        asset_ids.into_iter().collect()
    }

    fn asset_id_for_path(&self, path: &Path) -> Option<String> {
        let file_name = path.file_name()?.to_string_lossy();
        if !file_name.ends_with(".ui.toml") {
            return None;
        }
        let relative = path.strip_prefix(&self.assets_root).ok()?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        Some(format!("res://{normalized}"))
    }
}

impl EditorUiHost {
    pub(in crate::ui::host) fn restart_ui_asset_workspace_watcher(
        &self,
    ) -> Result<(), EditorError> {
        let Some(project_root) = self.current_project_root()? else {
            *self.ui_asset_workspace_watcher.lock().unwrap() = None;
            return Ok(());
        };
        let watcher = UiAssetWorkspaceWatcher::start(project_root).ok();
        *self.ui_asset_workspace_watcher.lock().unwrap() = watcher;
        Ok(())
    }

    pub fn poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError> {
        let changed_asset_ids = self
            .ui_asset_workspace_watcher
            .lock()
            .unwrap()
            .as_ref()
            .map(UiAssetWorkspaceWatcher::drain_changed_asset_ids)
            .unwrap_or_default();
        self.refresh_ui_asset_workspace_for_changes(changed_asset_ids.clone())?;
        Ok(changed_asset_ids)
    }
}
