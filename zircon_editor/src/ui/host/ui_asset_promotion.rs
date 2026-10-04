use std::{fmt::Write as _, path::PathBuf};

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
    let mut asset_id = initial_asset_id(preferred_asset_id);
    loop {
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalWidgetTarget {
                source_path,
                asset_id,
                document_id: candidate_document_id(preferred_document_id, suffix),
            });
        }
        suffix += 1;
        reset_suffixed_asset_id(&mut asset_id, preferred_asset_id, suffix);
    }
}

pub(crate) fn resolve_external_style_target(
    project: &ProjectManager,
    preferred_asset_id: &str,
    preferred_document_id: &str,
    preferred_display_name: &str,
) -> Result<UiAssetExternalStyleTarget, EditorError> {
    let mut suffix = 0usize;
    let mut asset_id = initial_asset_id(preferred_asset_id);
    loop {
        let source_path = resolve_project_asset_write_path(project, &asset_id)?;
        if !source_path.exists() {
            return Ok(UiAssetExternalStyleTarget {
                source_path,
                asset_id,
                document_id: candidate_document_id(preferred_document_id, suffix),
                display_name: candidate_display_name(preferred_display_name, suffix),
            });
        }
        suffix += 1;
        reset_suffixed_asset_id(&mut asset_id, preferred_asset_id, suffix);
    }
}

fn initial_asset_id(asset_id: &str) -> String {
    let mut candidate = String::with_capacity(asset_id.len() + 21);
    candidate.push_str(asset_id);
    candidate
}

fn reset_suffixed_asset_id(target: &mut String, asset_id: &str, suffix: usize) {
    target.clear();
    if let Some(base) = asset_id.strip_suffix(".zui") {
        target.push_str(base);
        write!(target, "_{suffix}.zui").expect("writing to a String cannot fail");
    } else {
        target.push_str(asset_id);
        write!(target, "_{suffix}").expect("writing to a String cannot fail");
    }
}

fn candidate_document_id(document_id: &str, suffix: usize) -> String {
    if suffix == 0 {
        document_id.to_owned()
    } else {
        format!("{document_id}_{suffix}")
    }
}

fn candidate_display_name(display_name: &str, suffix: usize) -> String {
    if suffix == 0 {
        display_name.to_owned()
    } else {
        format!("{display_name} {suffix}")
    }
}

#[cfg(test)]
#[path = "tests/ui_asset_promotion_optimization_tests.rs"]
mod optimization_tests;
