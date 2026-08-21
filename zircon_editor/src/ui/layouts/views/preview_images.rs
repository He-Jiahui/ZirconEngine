use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use resvg::{tiny_skia, usvg};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;

use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

const ICON_PREVIEW_PLACEHOLDER_SIZE: u32 = 24;
const PREVIEW_IMAGE_CACHE_MAX_SOURCES: usize = 128;

pub(crate) fn load_preview_image(source: &str, icon_name: &str) -> Image {
    load_preview_image_for_generation(source, icon_name, 0)
}

pub(crate) fn load_preview_image_for_generation(
    source: &str,
    icon_name: &str,
    resource_generation: u64,
) -> Image {
    {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_cache_lookup");
        if let Some(image) = cached_preview_image(
            &mut preview_image_cache()
                .lock()
                .expect("preview image cache mutex should not be poisoned"),
            source,
            icon_name,
            resource_generation,
        ) {
            return image;
        }
    }

    let image = {
        zircon_runtime::profile_scope!("editor", "retained_host", "preview_image_load_uncached");
        load_preview_image_uncached(source, icon_name)
    };
    insert_preview_image(
        &mut preview_image_cache()
            .lock()
            .expect("preview image cache mutex should not be poisoned"),
        source,
        icon_name,
        PreviewImageCacheEntry {
            resource_generation,
            image: image.clone(),
        },
    );
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

struct PreviewImageCacheEntry {
    resource_generation: u64,
    image: Image,
}

#[derive(Default)]
struct PreviewImageSourceCache {
    icons: HashMap<String, PreviewImageCacheEntry>,
    last_used: u64,
}

#[derive(Default)]
struct PreviewImageCache {
    sources: HashMap<String, PreviewImageSourceCache>,
    clock: u64,
}

impl PreviewImageCache {
    fn new() -> Self {
        Self::default()
    }

    fn next_access_tick(&mut self) -> u64 {
        self.clock = self.clock.wrapping_add(1);
        self.clock
    }
}

fn cached_preview_image(
    cache: &mut PreviewImageCache,
    source: &str,
    icon_name: &str,
    resource_generation: u64,
) -> Option<Image> {
    let access_tick = cache.next_access_tick();
    let source_cache = cache.sources.get_mut(source)?;
    let image = source_cache
        .icons
        .get(icon_name)
        .filter(|entry| entry.resource_generation == resource_generation)
        .map(|entry| entry.image.clone())?;
    source_cache.last_used = access_tick;
    Some(image)
}

fn insert_preview_image(
    cache: &mut PreviewImageCache,
    source: &str,
    icon_name: &str,
    entry: PreviewImageCacheEntry,
) {
    if !cache.sources.contains_key(source) && cache.sources.len() == PREVIEW_IMAGE_CACHE_MAX_SOURCES
    {
        if let Some(stale_source) = cache
            .sources
            .iter()
            .min_by_key(|(_, entry)| entry.last_used)
            .map(|(source, _)| source.clone())
        {
            cache.sources.remove(&stale_source);
        }
    }
    let access_tick = cache.next_access_tick();
    let source_cache = cache.sources.entry(source.to_owned()).or_default();
    source_cache.last_used = access_tick;
    source_cache.icons.insert(icon_name.to_owned(), entry);
}

fn preview_image_cache() -> &'static Mutex<PreviewImageCache> {
    static CACHE: OnceLock<Mutex<PreviewImageCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(PreviewImageCache::new()))
}

#[cfg(test)]
fn clear_preview_image_cache() {
    preview_image_cache()
        .lock()
        .expect("preview image cache mutex should not be poisoned")
        .sources
        .clear();
}

#[cfg(test)]
fn preview_image_cache_len() -> usize {
    preview_image_cache()
        .lock()
        .expect("preview image cache mutex should not be poisoned")
        .sources
        .values()
        .map(|entry| entry.icons.len())
        .sum()
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
    let assets = resolve_editor_asset(Path::new(""));
    preview_image_candidates_from_asset_root(source, icon_name, &assets)
}

fn preview_image_candidates_from_asset_root(
    source: &str,
    icon_name: &str,
    assets: &Path,
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if !source.is_empty() {
        let source = normalized_asset_relative_path(source);
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
    if let Some(stripped) = value.strip_prefix("res://") {
        value = stripped.to_string();
    }
    let mut relative = PathBuf::new();
    for component in Path::new(value.trim_start_matches('/')).components() {
        match component {
            std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::CurDir
            | std::path::Component::ParentDir => {}
            std::path::Component::Normal(value)
                if relative.as_os_str().is_empty() && value == std::ffi::OsStr::new("assets") => {}
            std::path::Component::Normal(value) => relative.push(value),
        }
    }
    relative
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
    fn preview_candidates_keep_windows_absolute_inputs_inside_the_selected_root() {
        let root = Path::new("E:/portable-product/assets");
        let candidates = preview_image_candidates_from_asset_root(
            r"C:\source-tree\logo.svg",
            "folder-open-outline",
            root,
        );

        assert!(!candidates.is_empty());
        assert!(candidates
            .iter()
            .all(|candidate| candidate.starts_with(root)));
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
    fn preview_cache_hit_accepts_borrowed_key_components() {
        let mut cache = PreviewImageCache::new();
        insert_preview_image(
            &mut cache,
            "res://icons/close.svg",
            "close",
            PreviewImageCacheEntry {
                resource_generation: 7,
                image: Image::default(),
            },
        );

        assert!(cached_preview_image(&mut cache, "res://icons/close.svg", "close", 7).is_some());
        assert!(cached_preview_image(&mut cache, "res://icons/close.svg", "close", 8).is_none());
    }

    #[test]
    fn preview_cache_replaces_an_obsolete_resource_generation() {
        let mut cache = PreviewImageCache::new();
        insert_preview_image(
            &mut cache,
            "res://icons/close.svg",
            "close",
            PreviewImageCacheEntry {
                resource_generation: 7,
                image: Image::default(),
            },
        );
        insert_preview_image(
            &mut cache,
            "res://icons/close.svg",
            "close",
            PreviewImageCacheEntry {
                resource_generation: 8,
                image: Image::default(),
            },
        );

        assert_eq!(
            cache
                .sources
                .values()
                .map(|entry| entry.icons.len())
                .sum::<usize>(),
            1
        );
        assert!(cached_preview_image(&mut cache, "res://icons/close.svg", "close", 7).is_none());
        assert!(cached_preview_image(&mut cache, "res://icons/close.svg", "close", 8).is_some());
    }

    #[test]
    fn preview_cache_evicts_the_least_recently_used_source_bucket_at_capacity() {
        let mut cache = PreviewImageCache::new();
        for source_index in 0..PREVIEW_IMAGE_CACHE_MAX_SOURCES {
            insert_preview_image(
                &mut cache,
                &format!("res://icons/{source_index}.svg"),
                "",
                PreviewImageCacheEntry {
                    resource_generation: 1,
                    image: Image::default(),
                },
            );
        }
        assert!(cached_preview_image(&mut cache, "res://icons/0.svg", "", 1).is_some());

        insert_preview_image(
            &mut cache,
            "res://icons/new.svg",
            "",
            PreviewImageCacheEntry {
                resource_generation: 1,
                image: Image::default(),
            },
        );

        assert_eq!(cache.sources.len(), PREVIEW_IMAGE_CACHE_MAX_SOURCES);
        assert!(cache.sources.contains_key("res://icons/0.svg"));
        assert!(!cache.sources.contains_key("res://icons/1.svg"));
        assert!(cache.sources.contains_key("res://icons/new.svg"));
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
