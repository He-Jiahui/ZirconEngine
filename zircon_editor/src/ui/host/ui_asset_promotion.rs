use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;

use super::editor_error::EditorError;
use super::project_access::resolve_project_asset_write_path;

pub(crate) struct UiAssetExternalWidgetTarget {
    pub(crate) source_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
}

pub(crate) struct UiAssetExternalStyleTarget {
    pub(crate) source_path: PathBuf,
    pub(crate) asset_id: String,
    pub(crate) document_id: String,
    pub(crate) display_name: String,
}

pub(crate) fn resolve_external_widget_target(
    project: &ProjectManager,
    preferred_asset_id: &str,
    _component_name: &str,
    preferred_document_id: &str,
) -> Result<UiAssetExternalWidgetTarget, EditorError> {
    let mut suffix = 0usize;
    loop {
        let asset_id = if suffix == 0 {
            preferred_asset_id.to_string()
        } else {
            suffixed_asset_id(preferred_asset_id, suffix)
        };
        let document_id = if suffix == 0 {
            preferred_document_id.to_string()
        } else {
            format!("{preferred_document_id}_{suffix}")
        };
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalWidgetTarget {
                source_path,
                asset_id,
                document_id,
            });
        }
        suffix += 1;
    }
}

pub(crate) fn resolve_external_style_target(
    project: &ProjectManager,
    preferred_asset_id: &str,
    preferred_document_id: &str,
    preferred_display_name: &str,
) -> Result<UiAssetExternalStyleTarget, EditorError> {
    let mut suffix = 0usize;
    loop {
        let asset_id = if suffix == 0 {
            preferred_asset_id.to_string()
        } else {
            suffixed_asset_id(preferred_asset_id, suffix)
        };
        let document_id = if suffix == 0 {
            preferred_document_id.to_string()
        } else {
            format!("{preferred_document_id}_{suffix}")
        };
        let display_name = if suffix == 0 {
            preferred_display_name.to_string()
        } else {
            format!("{preferred_display_name} {suffix}")
        };
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalStyleTarget {
                source_path,
                asset_id,
                document_id,
                display_name,
            });
        }
        suffix += 1;
    }
}

fn suffixed_asset_id(asset_id: &str, suffix: usize) -> String {
    if let Some(base) = asset_id.strip_suffix(".zui") {
        format!("{base}_{suffix}.zui")
    } else {
        format!("{asset_id}_{suffix}")
    }
}
