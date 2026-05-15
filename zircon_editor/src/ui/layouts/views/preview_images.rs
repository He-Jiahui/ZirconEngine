use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use resvg::{tiny_skia, usvg};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

const ICON_PREVIEW_PLACEHOLDER_SIZE: u32 = 24;

pub(crate) fn load_preview_image(source: &str, icon_name: &str) -> Image {
    let key = PreviewImageCacheKey::new(source, icon_name);
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_cache_lookup");
        if let Some(image) = preview_image_cache()
            .lock()
            .expect("preview image cache mutex should not be poisoned")
            .get(&key)
            .cloned()
        {
            return image;
        }
    }

    let image = {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_load_uncached");
        load_preview_image_uncached(source, icon_name)
    };
    preview_image_cache()
        .lock()
        .expect("preview image cache mutex should not be poisoned")
        .insert(key, image.clone());
    image
}

fn load_preview_image_uncached(source: &str, icon_name: &str) -> Image {
    if source.trim().is_empty() && !icon_name.trim().is_empty() {
        return load_icon_preview_placeholder(icon_name);
    }

    for path in preview_image_candidates(source, icon_name) {
        if path.exists() {
            return load_preview_image_from_path(&path).unwrap_or_default();
        }
    }
    Image::default()
}

fn load_icon_preview_placeholder(icon_name: &str) -> Image {
    for path in preview_image_candidates("", icon_name) {
        if path.exists() {
            return icon_preview_placeholder();
        }
    }
    Image::default()
}

fn icon_preview_placeholder() -> Image {
    static PLACEHOLDER: OnceLock<Image> = OnceLock::new();
    PLACEHOLDER
        .get_or_init(|| {
            let pixel_count =
                (ICON_PREVIEW_PLACEHOLDER_SIZE * ICON_PREVIEW_PLACEHOLDER_SIZE) as usize;
            let mut pixels = Vec::with_capacity(pixel_count * 4);
            for _ in 0..pixel_count {
                pixels.extend_from_slice(&[255, 255, 255, 255]);
            }
            Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
                &pixels,
                ICON_PREVIEW_PLACEHOLDER_SIZE,
                ICON_PREVIEW_PLACEHOLDER_SIZE,
            ))
        })
        .clone()
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct PreviewImageCacheKey {
    source: String,
    icon_name: String,
}

impl PreviewImageCacheKey {
    fn new(source: &str, icon_name: &str) -> Self {
        Self {
            source: source.to_string(),
            icon_name: icon_name.to_string(),
        }
    }
}

fn preview_image_cache() -> &'static Mutex<BTreeMap<PreviewImageCacheKey, Image>> {
    static CACHE: OnceLock<Mutex<BTreeMap<PreviewImageCacheKey, Image>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

#[cfg(test)]
fn clear_preview_image_cache() {
    preview_image_cache()
        .lock()
        .expect("preview image cache mutex should not be poisoned")
        .clear();
}

#[cfg(test)]
fn preview_image_cache_len() -> usize {
    preview_image_cache()
        .lock()
        .expect("preview image cache mutex should not be poisoned")
        .len()
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
    if svg_may_need_fonts(&svg) {
        options.fontdb = cached_svg_font_db();
    }

    let tree = {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_svg_parse");
        usvg::Tree::from_data(&svg, &options).ok()?
    };
    let (width, height, pixmap) = {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_svg_raster");
        let size = tree.size();
        let width = size.width().ceil().max(1.0) as u32;
        let height = size.height().ceil().max(1.0) as u32;
        let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
        resvg::render(
            &tree,
            tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        (width, height, pixmap)
    };
    let pixels = pixmap.take_demultiplied();
    Some(Image::from_rgba8(
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&pixels, width, height),
    ))
}

fn cached_svg_font_db() -> Arc<usvg::fontdb::Database> {
    static SVG_FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    SVG_FONT_DB
        .get_or_init(|| {
            zircon_runtime::profile_scope!(
                "editor",
                "retained_host",
                "preview_image_init_system_font_db"
            );
            let mut database = usvg::fontdb::Database::new();
            database.load_system_fonts();
            Arc::new(database)
        })
        .clone()
}

fn svg_may_need_fonts(svg: &[u8]) -> bool {
    let Ok(svg) = std::str::from_utf8(svg) else {
        return false;
    };
    svg.contains("<text") || svg.contains("<tspan") || svg.contains("font-family")
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
        clear_preview_image_cache();
        let image = load_preview_image("", "folder-open-outline");
        let size = image.size();

        assert!(size.width > 0);
        assert!(size.height > 0);
        assert!(image.to_rgba8().is_some());
    }

    #[test]
    fn preview_loader_uses_fixed_metadata_for_icon_only_nodes() {
        clear_preview_image_cache();

        let image = load_preview_image("", "folder-open-outline");
        let size = image.size();

        assert_eq!(size.width, ICON_PREVIEW_PLACEHOLDER_SIZE);
        assert_eq!(size.height, ICON_PREVIEW_PLACEHOLDER_SIZE);
    }

    #[test]
    fn preview_loader_reuses_cached_svg_icons() {
        clear_preview_image_cache();

        let first = load_preview_image("", "folder-open-outline");
        let len_after_first = preview_image_cache_len();
        let second = load_preview_image("", "folder-open-outline");
        let len_after_second = preview_image_cache_len();

        assert_eq!(len_after_first, 1);
        assert_eq!(len_after_second, 1);
        assert_eq!(first.size(), second.size());
    }

    #[test]
    fn preview_svg_font_scan_is_reserved_for_text_svg() {
        assert!(!svg_may_need_fonts(
            br#"<svg viewBox="0 0 16 16"><path d="M0 0h16v16H0z"/></svg>"#
        ));
        assert!(svg_may_need_fonts(
            br#"<svg viewBox="0 0 16 16"><text x="0" y="12">A</text></svg>"#
        ));
        assert!(svg_may_need_fonts(
            br#"<svg viewBox="0 0 16 16"><path style="font-family:Arial" /></svg>"#
        ));
    }
}
