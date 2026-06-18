use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use resvg::{tiny_skia, usvg};

use super::{
    retained_image_resource_key, tint_non_transparent_pixels, HostPaintImagePixels,
    RasterTargetSize, MAX_VECTOR_RASTER_EDGE,
};

pub(super) fn render_svg_file_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_pixels(tree, target, tint)
}

pub(super) fn render_svg_tree_pixels(
    tree: Arc<usvg::Tree>,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let pixmap = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_raster");
        let svg_size = tree.size();
        let transform = tiny_skia::Transform::from_scale(
            target.width as f32 / svg_size.width(),
            target.height as f32 / svg_size.height(),
        );
        let mut pixmap = tiny_skia::Pixmap::new(target.width, target.height)?;
        resvg::render(tree.as_ref(), transform, &mut pixmap.as_mut());
        pixmap
    };

    let mut rgba = pixmap.take_demultiplied();
    if let Some(tint) = tint {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_tint");
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(target.width, target.height, &rgba),
        width: target.width,
        height: target.height,
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}

pub(super) fn render_svg_file_image(
    path: &Path,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_image(tree)
}

pub(super) fn render_svg_tree_image(
    tree: Arc<usvg::Tree>,
) -> Option<crate::ui::retained_host::primitives::Image> {
    let size = tree.size();
    let width = size
        .width()
        .ceil()
        .clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32;
    let height = size
        .height()
        .ceil()
        .clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    resvg::render(
        tree.as_ref(),
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let pixels = pixmap.take_demultiplied();
    Some(crate::ui::retained_host::primitives::Image::from_rgba8(
        crate::ui::retained_host::primitives::SharedPixelBuffer::<
            crate::ui::retained_host::primitives::Rgba8Pixel,
        >::clone_from_slice(&pixels, width, height),
    ))
}

pub(super) fn parse_svg_tree_data(
    svg: &[u8],
    resources_dir: Option<PathBuf>,
) -> Option<usvg::Tree> {
    let mut options = usvg::Options {
        resources_dir,
        ..usvg::Options::default()
    };
    if svg_may_need_fonts(svg) {
        options.fontdb = cached_svg_font_db();
    }

    usvg::Tree::from_data(svg, &options).ok()
}

fn load_svg_tree(path: &Path) -> Option<Arc<usvg::Tree>> {
    let cache_key = SvgTreeCacheKey::from_path(path);
    let cache = svg_tree_cache();
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_lookup"
        );
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&cache_key)
        {
            return cached.clone();
        }
    }

    let tree = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_render_svg_parse");
        parse_svg_tree_file(path).map(Arc::new)
    };
    {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_svg_tree_cache_store"
        );
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(cache_key, tree.clone());
    }
    tree
}

fn parse_svg_tree_file(path: &Path) -> Option<usvg::Tree> {
    let svg = fs::read(path).ok()?;
    let resources_dir = fs::canonicalize(path)
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));
    parse_svg_tree_data(&svg, resources_dir)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SvgTreeCacheKey {
    path: PathBuf,
    modified_unix_ns: Option<u128>,
    len: Option<u64>,
}

impl SvgTreeCacheKey {
    fn from_path(path: &Path) -> Self {
        let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let metadata = std::fs::metadata(&path).ok();
        let modified_unix_ns = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_nanos());
        let len = metadata.as_ref().map(std::fs::Metadata::len);
        Self {
            path,
            modified_unix_ns,
            len,
        }
    }
}

fn svg_tree_cache() -> &'static Mutex<BTreeMap<SvgTreeCacheKey, Option<Arc<usvg::Tree>>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<SvgTreeCacheKey, Option<Arc<usvg::Tree>>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn cached_svg_font_db() -> Arc<usvg::fontdb::Database> {
    static SVG_FONT_DB: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();
    SVG_FONT_DB
        .get_or_init(|| {
            zircon_runtime::profile_scope!(
                "editor",
                "host_painter",
                "visual_assets_init_system_font_db"
            );
            let mut database = usvg::fontdb::Database::new();
            database.load_system_fonts();
            Arc::new(database)
        })
        .clone()
}

pub(super) fn svg_may_need_fonts(svg: &[u8]) -> bool {
    let Ok(svg) = std::str::from_utf8(svg) else {
        return false;
    };
    svg.contains("<text") || svg.contains("<tspan") || svg.contains("font-family")
}
