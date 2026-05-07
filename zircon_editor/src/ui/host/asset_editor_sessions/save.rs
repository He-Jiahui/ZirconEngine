use std::fs;
use std::path::{Path, PathBuf};

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::project_access::normalize_ui_asset_asset_id;

impl EditorUiHost {
    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (saved, asset_id, source_path) = {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .save_to_canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (
                saved,
                entry.session.route().asset_id.clone(),
                entry.source_path.clone(),
            )
        };
        fs::write(&source_path, &saved).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.update_disk_baseline(saved.clone());
            entry.conflict = None;
            entry.diff_snapshot = None;
        }
        if asset_id.starts_with("res://") {
            let normalized = normalize_ui_asset_asset_id(&asset_id).to_string();
            let _ = self.asset_manager()?.import_asset(&normalized);
            self.refresh_ui_asset_workspace_for_changes(vec![normalized])?;
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(saved)
    }

    pub fn save_ui_asset_editor_local_copy(
        &self,
        instance_id: &ViewInstanceId,
        copy_path: impl AsRef<Path>,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let saved = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry
                .session
                .canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?
        };
        if let Some(parent) = copy_path.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        }
        fs::write(copy_path.as_ref(), &saved)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        Ok(saved)
    }

    pub fn save_ui_asset_editor_local_copy_next_to_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<PathBuf, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let source_path = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.source_path.clone()
        };
        let copy_path = next_local_copy_path(&source_path)?;
        self.save_ui_asset_editor_local_copy(instance_id, &copy_path)?;
        Ok(copy_path)
    }
}

fn next_local_copy_path(source_path: &Path) -> Result<PathBuf, EditorError> {
    let parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    let stem = source_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("ui_asset");
    let extension = source_path.extension().and_then(|value| value.to_str());
    for index in 0..1000 {
        let suffix = if index == 0 {
            String::new()
        } else {
            format!("-{index}")
        };
        let file_name = match extension {
            Some(extension) if !extension.is_empty() => {
                format!("{stem}.local-copy{suffix}.{extension}")
            }
            _ => format!("{stem}.local-copy{suffix}"),
        };
        let candidate = parent.join(file_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(EditorError::UiAsset(format!(
        "could not allocate a local copy path for {}",
        source_path.display()
    )))
}
