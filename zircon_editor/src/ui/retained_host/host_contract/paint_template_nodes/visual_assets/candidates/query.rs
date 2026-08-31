use std::path::PathBuf;

use super::super::mui_icons;
use super::aliases::shell_icon_alias;
use super::paths::{
    editor_asset_root, is_editor_dev_asset_root, normalized_asset_relative_path, workspace_root,
};
use super::variants::{push_candidate, push_svg_variants};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_candidates(
    source: &str,
) -> Vec<PathBuf> {
    let assets = editor_asset_root();
    image_candidates_from_asset_root(source, &assets)
}

fn image_candidates_from_asset_root(source: &str, assets: &std::path::Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
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
    let mut candidates = Vec::new();
    if source_path.is_absolute() {
        push_candidate(&mut candidates, source_path);
        return candidates;
    }
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
    let mut candidates = Vec::new();
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
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        icon_candidates_from_asset_root, image_candidates_from_asset_root,
        preview_artifact_candidates,
    };

    #[test]
    fn packaged_image_candidates_remain_inside_the_selected_asset_root() {
        let root = Path::new("E:/portable-product/assets");
        let candidates = image_candidates_from_asset_root(r"C:\source-tree\logo.svg", root);

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.starts_with(root)));
    }

    #[test]
    fn generated_preview_artifacts_preserve_their_absolute_source_identity() {
        #[cfg(windows)]
        let source = r"E:\project\.zircon\cache\editor-previews\grid.png";
        #[cfg(not(windows))]
        let source = "/project/.zircon/cache/editor-previews/grid.png";

        assert_eq!(
            preview_artifact_candidates(source).first(),
            Some(&PathBuf::from(source))
        );
    }

    #[test]
    fn packaged_icon_candidates_do_not_use_development_modules() {
        let root = Path::new("E:/portable-product/assets");
        let candidates = icon_candidates_from_asset_root("Search", root, false);

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.starts_with(root)));
        assert!(candidates
            .iter()
            .all(|candidate| !candidate.to_string_lossy().contains("dev/material-ui")));
    }

    #[test]
    fn search_field_semantic_icons_resolve_to_canonical_packaged_candidates() {
        let root = Path::new("E:/portable-product/assets");
        let search = icon_candidates_from_asset_root("search", root, false);
        let clear = icon_candidates_from_asset_root("close-outline", root, false);

        assert_eq!(
            search.first(),
            Some(&root.join("icons/zircon_editor_shell/controls/search.svg"))
        );
        assert!(clear
            .iter()
            .any(|candidate| candidate == &root.join("icons/ionicons/close-outline.svg")));
    }
}
