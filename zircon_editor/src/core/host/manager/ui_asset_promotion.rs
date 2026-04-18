use std::path::{Path, PathBuf};

pub(super) struct UiAssetExternalWidgetTarget {
    pub(super) source_path: PathBuf,
    pub(super) asset_id: String,
    pub(super) document_id: String,
}

pub(super) struct UiAssetExternalStyleTarget {
    pub(super) source_path: PathBuf,
    pub(super) asset_id: String,
    pub(super) document_id: String,
    pub(super) display_name: String,
}

pub(super) fn resolve_external_widget_target(
    project_root: &Path,
    preferred_asset_id: &str,
    _component_name: &str,
    preferred_document_id: &str,
) -> UiAssetExternalWidgetTarget {
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
        let source_path = asset_id_to_path(project_root, &asset_id);
        if !source_path.exists() {
            return UiAssetExternalWidgetTarget {
                source_path,
                asset_id,
                document_id,
            };
        }
        suffix += 1;
    }
}

pub(super) fn resolve_external_style_target(
    project_root: &Path,
    preferred_asset_id: &str,
    preferred_document_id: &str,
    preferred_display_name: &str,
) -> UiAssetExternalStyleTarget {
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
        let source_path = asset_id_to_path(project_root, &asset_id);
        if !source_path.exists() {
            return UiAssetExternalStyleTarget {
                source_path,
                asset_id,
                document_id,
                display_name,
            };
        }
        suffix += 1;
    }
}

fn asset_id_to_path(project_root: &Path, asset_id: &str) -> PathBuf {
    let relative = asset_id.strip_prefix("res://").unwrap_or(asset_id);
    project_root.join("assets").join(relative)
}

fn suffixed_asset_id(asset_id: &str, suffix: usize) -> String {
    if let Some(base) = asset_id.strip_suffix(".ui.toml") {
        format!("{base}_{suffix}.ui.toml")
    } else {
        format!("{asset_id}_{suffix}")
    }
}
