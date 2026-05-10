use std::path::{Path, PathBuf};

use resvg::{tiny_skia, usvg};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

pub(crate) fn load_preview_image(source: &str, icon_name: &str) -> Image {
    for path in preview_image_candidates(source, icon_name) {
        if path.exists() {
            return load_preview_image_from_path(&path).unwrap_or_default();
        }
    }
    Image::default()
}

fn load_preview_image_from_path(path: &Path) -> Option<Image> {
    if is_svg_path(path) {
        return render_svg_preview_image(path);
    }
    Image::load_from_path(path).ok()
}

fn render_svg_preview_image(path: &Path) -> Option<Image> {
    let svg = std::fs::read(path).ok()?;
    let mut options = usvg::Options {
        resources_dir: std::fs::canonicalize(path)
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf)),
        ..usvg::Options::default()
    };
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(&svg, &options).ok()?;
    let size = tree.size();
    let width = size.width().ceil().max(1.0) as u32;
    let height = size.height().ceil().max(1.0) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let pixels = pixmap.take_demultiplied();
    Some(Image::from_rgba8(
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&pixels, width, height),
    ))
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

fn is_svg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_loader_rasterizes_svg_icon_candidates() {
        let image = load_preview_image("", "folder-open-outline");
        let size = image.size();

        assert!(size.width > 0);
        assert!(size.height > 0);
        assert!(image.to_rgba8().is_some());
    }
}
