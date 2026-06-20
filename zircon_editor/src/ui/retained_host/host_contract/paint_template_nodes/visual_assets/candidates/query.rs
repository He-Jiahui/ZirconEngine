use std::path::PathBuf;

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

use super::super::mui_icons;
use super::paths::{editor_dev_asset_root, normalized_asset_relative_path, workspace_root};
use super::variants::{push_candidate, push_direct_candidate, push_svg_variants};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn image_candidates(
    source: &str,
) -> Vec<PathBuf> {
    let assets = editor_dev_asset_root();
    let mut candidates = Vec::new();
    if !source.is_empty() {
        push_direct_candidate(&mut candidates, source);
        let source = normalized_asset_relative_path(source);
        push_svg_variants(
            &mut candidates,
            runtime_asset_path_with_dev_asset_root(&source, &assets),
        );
        push_svg_variants(&mut candidates, assets.join(&source));
        push_svg_variants(&mut candidates, assets.join("icons").join(&source));
    }
    candidates
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_image_candidates(
    source: &str,
    icon_name: &str,
) -> Vec<PathBuf> {
    let mut candidates = image_candidates(source);
    for candidate in icon_candidates(icon_name) {
        push_candidate(&mut candidates, candidate);
    }
    candidates
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_candidates(
    icon_name: &str,
) -> Vec<PathBuf> {
    let assets = editor_dev_asset_root();
    let mut candidates = Vec::new();
    if !icon_name.is_empty() {
        let icon = normalized_asset_relative_path(icon_name);
        push_svg_variants(&mut candidates, assets.join("icons").join(&icon));
        push_svg_variants(
            &mut candidates,
            assets.join("icons").join("ionicons").join(&icon),
        );
        for candidate in mui_icons::module_candidates(icon_name, &workspace_root()) {
            push_candidate(&mut candidates, candidate);
        }
    }
    candidates
}
