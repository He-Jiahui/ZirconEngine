use std::path::{Path, PathBuf};

use super::{
    icon_candidates_from_asset_root, image_candidates_from_asset_root, preview_artifact_candidates,
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
