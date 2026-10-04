use std::path::PathBuf;

use super::super::mui_icons;
use super::aliases::shell_icon_alias;
use super::paths::{
    editor_asset_root, is_editor_dev_asset_root, normalized_asset_relative_path, workspace_root,
};
use super::variants::{push_candidate, push_svg_variants};

// These bounds describe packaged candidate shape only; development-module
// discovery remains an explicitly extensible path below.
const MAX_PACKAGED_IMAGE_CANDIDATES: usize = 4;
const MAX_PREVIEW_ARTIFACT_CANDIDATES: usize = 5;
const MAX_PACKAGED_ICON_CANDIDATES: usize = 6;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_candidates(
    source: &str,
) -> Vec<PathBuf> {
    let assets = editor_asset_root();
    image_candidates_from_asset_root(source, &assets)
}

fn image_candidates_from_asset_root(source: &str, assets: &std::path::Path) -> Vec<PathBuf> {
    let mut candidates = if source.is_empty() {
        Vec::new()
    } else {
        Vec::with_capacity(MAX_PACKAGED_IMAGE_CANDIDATES)
    };
    if !source.is_empty() {
        let source = normalized_asset_relative_path(source);
        push_svg_variants(&mut candidates, assets.join(&source));
        push_svg_variants(&mut candidates, assets.join("icons").join(&source));
    }
    candidates
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn preview_artifact_candidates(
    source: &str,
) -> Vec<PathBuf> {
    let source = source.trim();
    if source.is_empty() {
        return Vec::new();
    }

    let source_path = PathBuf::from(source);
    if source_path.is_absolute() {
        let mut candidates = Vec::with_capacity(1);
        push_candidate(&mut candidates, source_path);
        return candidates;
    }
    let mut candidates = Vec::with_capacity(MAX_PREVIEW_ARTIFACT_CANDIDATES);
    if !source.contains("://") {
        push_candidate(&mut candidates, workspace_root().join(source_path));
    }
    for candidate in image_candidates(source) {
        push_candidate(&mut candidates, candidate);
    }
    candidates
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_candidates(
    icon_name: &str,
) -> Vec<PathBuf> {
    let assets = editor_asset_root();
    icon_candidates_from_asset_root(icon_name, &assets, is_editor_dev_asset_root(&assets))
}

fn icon_candidates_from_asset_root(
    icon_name: &str,
    assets: &std::path::Path,
    include_development_modules: bool,
) -> Vec<PathBuf> {
    let mut candidates = if icon_name.is_empty() {
        Vec::new()
    } else {
        Vec::with_capacity(MAX_PACKAGED_ICON_CANDIDATES)
    };
    if !icon_name.is_empty() {
        if let Some(shell_alias) = shell_icon_alias(icon_name) {
            push_svg_variants(&mut candidates, assets.join("icons").join(shell_alias));
        }
        let icon = normalized_asset_relative_path(icon_name);
        push_svg_variants(&mut candidates, assets.join("icons").join(&icon));
        push_svg_variants(
            &mut candidates,
            assets.join("icons").join("ionicons").join(&icon),
        );
        if include_development_modules {
            for candidate in mui_icons::module_candidates(icon_name, &workspace_root()) {
                push_candidate(&mut candidates, candidate);
            }
        }
    }
    candidates
}

#[cfg(test)]
#[path = "query/tests/capacity_tests.rs"]
mod capacity_tests;

#[cfg(test)]
#[path = "tests/query.rs"]
mod tests;
