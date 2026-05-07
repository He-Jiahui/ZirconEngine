use std::path::{Path, PathBuf};

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

pub(crate) fn load_preview_image(source: &str, icon_name: &str) -> slint::Image {
    for path in preview_image_candidates(source, icon_name) {
        if path.exists() {
            return slint::Image::load_from_path(&path).unwrap_or_default();
        }
    }
    slint::Image::default()
}

fn preview_image_candidates(source: &str, icon_name: &str) -> Vec<PathBuf> {
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut candidates = Vec::new();
    if !source.is_empty() {
        push_direct_candidate(&mut candidates, source);
        let source = normalized_asset_relative_path(source);
        push_svg_variants(&mut candidates, resolve_editor_asset(&source));
        push_svg_variants(&mut candidates, assets.join(&source));
        push_svg_variants(&mut candidates, assets.join("icons").join(&source));
    }
    if !icon_name.is_empty() {
        let icon = normalized_asset_relative_path(icon_name);
        push_svg_variants(&mut candidates, assets.join("icons").join(&icon));
        push_svg_variants(
            &mut candidates,
            assets.join("icons").join("ionicons").join(&icon),
        );
    }
    candidates
}

fn resolve_editor_asset(source: impl AsRef<Path>) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(source, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
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
