use std::path::{Path, PathBuf};

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

use super::mui_icons;

pub(super) fn image_candidates(source: &str) -> Vec<PathBuf> {
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

pub(super) fn template_image_candidates(source: &str, icon_name: &str) -> Vec<PathBuf> {
    let mut candidates = image_candidates(source);
    for candidate in icon_candidates(icon_name) {
        push_candidate(&mut candidates, candidate);
    }
    candidates
}

pub(super) fn icon_candidates(icon_name: &str) -> Vec<PathBuf> {
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

pub(super) fn first_existing_path(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|path| path.exists())
}

pub(super) fn is_svg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn push_direct_candidate(candidates: &mut Vec<PathBuf>, source: &str) {
    let path = PathBuf::from(source.trim());
    if path.is_absolute() {
        push_candidate(candidates, path);
    }
}

fn push_svg_variants(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if path.as_os_str().is_empty() {
        return;
    }
    if path.extension().is_some() {
        push_candidate(candidates, path);
        return;
    }
    push_candidate(candidates, path.with_extension("svg"));
    push_candidate(candidates, path);
}

fn push_candidate(candidates: &mut Vec<PathBuf>, path: PathBuf) {
    if !candidates.iter().any(|candidate| candidate == &path) {
        candidates.push(path);
    }
}

fn normalized_asset_relative_path(source: &str) -> PathBuf {
    let mut value = source.trim().replace('\\', "/");
    for prefix in ["res://", "asset://", "assets://"] {
        if let Some(stripped) = value.strip_prefix(prefix) {
            value = stripped.to_string();
            break;
        }
    }
    let value = value.trim_start_matches('/');
    let value = value.strip_prefix("assets/").unwrap_or(value);
    Path::new(value).to_path_buf()
}
