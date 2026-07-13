use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crossbeam_channel::{unbounded, Receiver};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use super::super::project_access::open_project_manager_for_paths;

pub(crate) struct UiAssetWorkspaceWatcher {
    asset_roots: Vec<PathBuf>,
    receiver: Receiver<PathBuf>,
    _watchers: Vec<RecommendedWatcher>,
}

impl UiAssetWorkspaceWatcher {
    pub(crate) fn start(project_root: PathBuf) -> Result<Self, EditorError> {
        let project = open_project_manager_for_paths(&project_root)?;
        let asset_roots = project.project_asset_roots().to_vec();
        let (sender, receiver) = unbounded::<PathBuf>();
        let mut watchers = Vec::with_capacity(asset_roots.len());
        for asset_root in &asset_roots {
            let sender = sender.clone();
            let mut watcher =
                notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                    let Ok(event) = event else {
                        return;
                    };
                    for path in event.paths {
                        let _ = sender.send(path);
                    }
                })?;
            watcher.watch(asset_root, RecursiveMode::Recursive)?;
            watchers.push(watcher);
        }
        Ok(Self {
            asset_roots,
            receiver,
            _watchers: watchers,
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
        if !file_name.ends_with(".zui") {
            return None;
        }
        let roots = self
            .asset_roots
            .iter()
            .filter(|root| path.starts_with(root))
            .collect::<Vec<_>>();
        let [asset_root] = roots.as_slice() else {
            return None;
        };
        let relative = path.strip_prefix(asset_root).ok()?;
        let normalized = relative.to_string_lossy().replace('\\', "/");
        Some(format!("res://{normalized}"))
    }
}

impl EditorUiHost {
    pub(in crate::ui::host) fn restart_ui_asset_workspace_watcher(
        &self,
    ) -> Result<(), EditorError> {
        let Some(project_root) = self.current_project_root()? else {
            *self.lock_ui_asset_workspace_watcher() = None;
            return Ok(());
        };
        let watcher = UiAssetWorkspaceWatcher::start(project_root)?;
        *self.lock_ui_asset_workspace_watcher() = Some(watcher);
        Ok(())
    }

    pub fn poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError> {
        let changed_asset_ids = self
            .lock_ui_asset_workspace_watcher()
            .as_ref()
            .map(UiAssetWorkspaceWatcher::drain_changed_asset_ids)
            .unwrap_or_default();
        self.refresh_ui_asset_workspace_for_changes(changed_asset_ids.clone())?;
        Ok(changed_asset_ids)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use super::UiAssetWorkspaceWatcher;
    use zircon_runtime::asset::project::{ProjectManifest, ProjectPaths};
    use zircon_runtime::asset::AssetUri;
    use zircon_runtime_interface::project::RelPath;

    #[test]
    fn watcher_reports_a_res_uri_for_an_event_created_in_the_second_manifest_root() {
        let root = std::env::temp_dir().join(format!(
            "zircon-editor-dual-root-watcher-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let paths = ProjectPaths::from_root(&root).unwrap();
        let mut manifest = ProjectManifest::new(
            "Watcher",
            AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
            1,
        );
        manifest.asset_roots = vec![
            RelPath::parse("game-assets").unwrap(),
            RelPath::parse("shared-assets").unwrap(),
        ];
        manifest.save(paths.manifest_path()).unwrap();
        let watcher = UiAssetWorkspaceWatcher::start(root.clone()).unwrap();
        let changed = root.join("shared-assets/ui/second-root.zui");
        fs::create_dir_all(changed.parent().unwrap()).unwrap();
        fs::write(&changed, "version = 2").unwrap();

        let mut ids = Vec::new();
        for _ in 0..100 {
            ids = watcher.drain_changed_asset_ids();
            if !ids.is_empty() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(ids.contains(&"res://ui/second-root.zui".to_string()));
        drop(watcher);
        let _ = fs::remove_dir_all(root);
    }
}
