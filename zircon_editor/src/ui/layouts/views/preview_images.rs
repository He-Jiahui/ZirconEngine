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
        candidates.push(resolve_editor_asset(source));
        candidates.push(assets.join(strip_assets_prefix(source)));
        candidates.push(assets.join("icons").join(strip_assets_prefix(source)));
    }
    if !icon_name.is_empty() {
        candidates.push(
            assets
                .join("icons")
                .join("ionicons")
                .join(format!("{icon_name}.svg")),
        );
    }
    candidates
}

fn resolve_editor_asset(source: &str) -> PathBuf {
    runtime_asset_path_with_dev_asset_root(source, editor_dev_asset_root())
}

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn strip_assets_prefix(source: &str) -> &Path {
    let path = Path::new(source.trim_start_matches(['/', '\\']));
    path.strip_prefix("assets").unwrap_or(path)
}
