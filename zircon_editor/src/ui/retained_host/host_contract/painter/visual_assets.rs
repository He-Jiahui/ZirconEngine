use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use resvg::{tiny_skia, usvg};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use super::sprite_atlas::{resolve_editor_sprite_atlas_image, HostPaintAtlasImage};
use super::theme::PALETTE;

mod mui_icons;

const ICON_TINT: [u8; 4] = PALETTE.text;
const ICON_TINT_ACTIVE: [u8; 4] = PALETTE.focus_ring;
const ICON_TINT_DISABLED: [u8; 4] = PALETTE.text_disabled;
const ICON_TINT_ERROR: [u8; 4] = PALETTE.error;
const ICON_TINT_WARNING: [u8; 4] = PALETTE.warning;
const MAX_VECTOR_RASTER_EDGE: u32 = 4096;
const MUI_ICON_DEFAULT_EDGE: u32 = 24;

#[derive(Clone)]
pub(super) struct HostPaintImagePixels {
    pub(super) resource_key: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
    pub(super) atlas: Option<HostPaintAtlasImage>,
}

impl HostPaintImagePixels {
    fn with_resource_key(mut self, resource_key: impl Into<String>) -> Self {
        self.resource_key = resource_key.into();
        self
    }

    fn with_atlas(mut self, atlas: Option<HostPaintAtlasImage>) -> Self {
        self.atlas = atlas;
        self
    }
}

#[derive(Clone, Copy)]
struct RasterTargetSize {
    width: u32,
    height: u32,
}

pub(super) fn retained_image_pixels(
    image: &crate::ui::retained_host::primitives::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let buffer = image.to_rgba8()?;
    let mut rgba = buffer.as_bytes().to_vec();
    if let Some(tint) = tint {
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(buffer.width(), buffer.height(), &rgba),
        width: buffer.width(),
        height: buffer.height(),
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}

pub(super) fn raster_size_from_frame(width: f32, height: f32) -> Option<(u32, u32)> {
    let target = RasterTargetSize::from_frame(width, height)?;
    Some((target.width, target.height))
}

pub(super) fn template_image_tint(
    is_icon_like: bool,
    active: bool,
    disabled: bool,
    text_tone: &str,
    validation_level: &str,
    style_tint: Option<[u8; 4]>,
) -> Option<[u8; 4]> {
    if !is_icon_like {
        return None;
    }
    if disabled {
        return Some(ICON_TINT_DISABLED);
    }
    if validation_level.eq_ignore_ascii_case("error") || text_tone.eq_ignore_ascii_case("error") {
        return Some(ICON_TINT_ERROR);
    }
    if validation_level.eq_ignore_ascii_case("warning") || text_tone.eq_ignore_ascii_case("warning")
    {
        return Some(ICON_TINT_WARNING);
    }
    if let Some(style_tint) = style_tint {
        return Some(style_tint);
    }
    if active {
        return Some(ICON_TINT_ACTIVE);
    }
    Some(ICON_TINT)
}

pub(super) fn template_image_pixels(
    preview_image: &crate::ui::retained_host::primitives::Image,
    media_source: &str,
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    prefer_preview_image: bool,
) -> Option<HostPaintImagePixels> {
    let target = RasterTargetSize::new(target_width, target_height);
    let key = template_image_cache_key(media_source, icon_name);
    let source_pixels = || {
        load_pixels_from_candidates(
            template_image_candidates(media_source, icon_name),
            &key,
            target,
            tint,
        )
    };
    let preview_pixels = || retained_image_pixels(preview_image, tint);
    let pixels = if prefer_preview_image {
        preview_pixels().or_else(source_pixels)
    } else {
        source_pixels().or_else(preview_pixels)
    };
    pixels.or_else(|| {
        (!icon_name.trim().is_empty())
            .then_some(())
            .and_then(|_| target)
            .and_then(|target| missing_icon_pixels(&key, target, tint))
    })
}

pub(super) fn load_visual_asset_pixels(asset: &UiVisualAssetRef) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, None)
}

pub(super) fn load_visual_asset_pixels_for_size(
    asset: &UiVisualAssetRef,
    target_width: u32,
    target_height: u32,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, RasterTargetSize::new(target_width, target_height))
}

fn load_visual_asset_pixels_for_target(
    asset: &UiVisualAssetRef,
    target: Option<RasterTargetSize>,
) -> Option<HostPaintImagePixels> {
    let key = visual_asset_cache_key(asset);
    match asset {
        UiVisualAssetRef::Icon(icon_name) => {
            let target = target.unwrap_or(RasterTargetSize {
                width: MUI_ICON_DEFAULT_EDGE,
                height: MUI_ICON_DEFAULT_EDGE,
            });
            load_pixels_from_candidates(
                icon_candidates(icon_name),
                &key,
                Some(target),
                Some(ICON_TINT),
            )
            .or_else(|| missing_icon_pixels(&key, target, Some(ICON_TINT)))
        }
        UiVisualAssetRef::Image(source) => {
            load_pixels_from_candidates(image_candidates(source), &key, target, None)
        }
    }
}

fn load_pixels_from_candidates(
    candidates: Vec<PathBuf>,
    base_key: &str,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let path = {
        zircon_runtime::profile_scope!(
            "editor",
            "host_painter",
            "visual_assets_first_existing_path"
        );
        first_existing_path(candidates)?
    };
    let key = image_pixels_cache_key(
        base_key,
        &path,
        target.filter(|_| is_svg_path(&path) || mui_icons::is_module_path(&path)),
        tint,
    );
    let cache = VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_lookup");
        if let Some(cached) = cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .get(&key)
        {
            return cached.clone();
        }
    }

    let loaded = {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_load_pixels");
        if mui_icons::is_module_path(&path) {
            let target = target.unwrap_or(RasterTargetSize {
                width: MUI_ICON_DEFAULT_EDGE,
                height: MUI_ICON_DEFAULT_EDGE,
            });
            mui_icons::render_module_pixels(&path, target, tint)
        } else if is_svg_path(&path) {
            target
                .and_then(|target| render_svg_file_pixels(&path, target, tint))
                .or_else(|| {
                    load_image_from_path(&path)
                        .and_then(|image| retained_image_pixels(&image, tint))
                })
        } else {
            load_image_from_path(&path).and_then(|image| retained_image_pixels(&image, tint))
        }
    }
    .map(|pixels| {
        if tint.is_none() {
            pixels.with_atlas(resolve_editor_sprite_atlas_image(base_key, &path))
        } else {
            pixels.with_resource_key(key.clone())
        }
    });

    {
        zircon_runtime::profile_scope!("editor", "host_painter", "visual_assets_cache_store");
        cache
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .insert(key, loaded.clone());
    }
    loaded
}

fn load_image_from_candidates(
    candidates: Vec<PathBuf>,
) -> Option<crate::ui::retained_host::primitives::Image> {
    for path in candidates {
        if let Some(image) = load_image_from_path(&path) {
            return Some(image);
        }
    }
    None
}

fn missing_icon_pixels(
    base_key: &str,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let color = tint.unwrap_or(ICON_TINT);
    let mut rgba = vec![0; target.width as usize * target.height as usize * 4];
    let edge = target.width.min(target.height);
    let stroke = (edge / 10).clamp(1, 3);
    let max_x = target.width.saturating_sub(1);
    let max_y = target.height.saturating_sub(1);

    for y in 0..target.height {
        for x in 0..target.width {
            let border = x < stroke
                || y < stroke
                || max_x.saturating_sub(x) < stroke
                || max_y.saturating_sub(y) < stroke;
            let diagonal = x.abs_diff(y) < stroke || x.abs_diff(max_y.saturating_sub(y)) < stroke;
            if !border && !diagonal {
                continue;
            }
            let offset = ((y * target.width + x) as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }

    let image = HostPaintImagePixels {
        resource_key: format!("missing-icon:{base_key}:{}x{}", target.width, target.height),
        width: target.width,
        height: target.height,
        rgba,
        atlas: None,
    };
    image.is_valid().then_some(image)
}

fn load_image_from_path(path: &Path) -> Option<crate::ui::retained_host::primitives::Image> {
    if !path.exists() {
        return None;
    }
    if mui_icons::is_module_path(path) {
        return mui_icons::render_module_image(path);
    }
    if is_svg_path(path) {
        return render_svg_file_image(path);
    }
    let image =
        crate::ui::retained_host::primitives::Image::load_from_path(path).unwrap_or_default();
    let size = image.size();
    (size.width > 0 && size.height > 0).then_some(image)
}

fn image_candidates(source: &str) -> Vec<PathBuf> {
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

fn template_image_candidates(source: &str, icon_name: &str) -> Vec<PathBuf> {
    let mut candidates = image_candidates(source);
    for candidate in icon_candidates(icon_name) {
        push_candidate(&mut candidates, candidate);
    }
    candidates
}

fn icon_candidates(icon_name: &str) -> Vec<PathBuf> {
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

fn visual_asset_cache_key(asset: &UiVisualAssetRef) -> String {
    match asset {
        UiVisualAssetRef::Icon(icon_name) => format!("icon:{icon_name}"),
        UiVisualAssetRef::Image(source) => format!("image:{source}"),
    }
}

fn template_image_cache_key(source: &str, icon_name: &str) -> String {
    if !icon_name.is_empty() {
        return format!("template-icon:{icon_name}");
    }
    format!("template-image:{source}")
}

fn image_pixels_cache_key(
    base_key: &str,
    path: &Path,
    target: Option<RasterTargetSize>,
    tint: Option<[u8; 4]>,
) -> String {
    let size_key = target
        .map(|target| format!("{}x{}", target.width, target.height))
        .unwrap_or_else(|| "intrinsic".to_string());
    let tint_key = tint
        .map(|tint| {
            format!(
                "{:02x}{:02x}{:02x}{:02x}",
                tint[0], tint[1], tint[2], tint[3]
            )
        })
        .unwrap_or_else(|| "none".to_string());
    format!("{base_key}:{size_key}:tint:{tint_key}:{}", path.display())
}

fn retained_image_resource_key(width: u32, height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("retained-image:{width}x{height}:{:016x}", hasher.finish())
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

fn first_existing_path(candidates: Vec<PathBuf>) -> Option<PathBuf> {
    candidates.into_iter().find(|path| path.exists())
}

fn is_svg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}

fn render_svg_file_pixels(
    path: &Path,
    target: RasterTargetSize,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_pixels(tree, target, tint)
}

fn render_svg_tree_pixels(
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

fn render_svg_file_image(path: &Path) -> Option<crate::ui::retained_host::primitives::Image> {
    let tree = load_svg_tree(path)?;
    render_svg_tree_image(tree)
}

fn render_svg_tree_image(
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

fn parse_svg_tree_data(svg: &[u8], resources_dir: Option<PathBuf>) -> Option<usvg::Tree> {
    let mut options = usvg::Options {
        resources_dir,
        ..usvg::Options::default()
    };
    if svg_may_need_fonts(svg) {
        options.fontdb = cached_svg_font_db();
    }

    usvg::Tree::from_data(svg, &options).ok()
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

fn svg_may_need_fonts(svg: &[u8]) -> bool {
    let Ok(svg) = std::str::from_utf8(svg) else {
        return false;
    };
    svg.contains("<text") || svg.contains("<tspan") || svg.contains("font-family")
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

fn tint_non_transparent_pixels(rgba: &mut [u8], tint: [u8; 4]) {
    for pixel in rgba.chunks_exact_mut(4) {
        if pixel[3] == 0 {
            continue;
        }
        pixel[0] = tint[0];
        pixel[1] = tint[1];
        pixel[2] = tint[2];
    }
}

impl HostPaintImagePixels {
    fn is_valid(&self) -> bool {
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }
}

impl RasterTargetSize {
    fn new(width: u32, height: u32) -> Option<Self> {
        (width > 0 && height > 0).then_some(Self { width, height })
    }

    fn from_frame(width: f32, height: f32) -> Option<Self> {
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return None;
        }
        Self::new(
            width.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
            height.ceil().clamp(1.0, MAX_VECTOR_RASTER_EDGE as f32) as u32,
        )
    }
}

static VISUAL_ASSET_CACHE: OnceLock<Mutex<BTreeMap<String, Option<HostPaintImagePixels>>>> =
    OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_svg_icon_pixels_follow_requested_target_size() {
        let icon = UiVisualAssetRef::Icon("folder-open-outline".to_string());

        let small = load_visual_asset_pixels_for_size(&icon, 16, 16)
            .expect("runtime SVG icon should render at a requested small size");
        let large = load_visual_asset_pixels_for_size(&icon, 48, 48)
            .expect("runtime SVG icon should render at a requested large size");

        assert_eq!((small.width, small.height), (16, 16));
        assert_eq!((large.width, large.height), (48, 48));
        assert_ne!(small.rgba.len(), large.rgba.len());
        assert!(has_visible_pixel(&large));
    }

    #[test]
    fn template_svg_icon_pixels_follow_requested_target_size() {
        let preview = load_image_from_candidates(icon_candidates("folder-open-outline"))
            .expect("test icon should load through the editor icon resolver");

        let small = template_image_pixels(
            &preview,
            "",
            "folder-open-outline",
            18,
            18,
            Some(ICON_TINT),
            false,
        )
        .expect("template SVG icon should render at a requested small size");
        let large = template_image_pixels(
            &preview,
            "",
            "folder-open-outline",
            54,
            54,
            Some(ICON_TINT),
            false,
        )
        .expect("template SVG icon should render at a requested large size");

        assert_eq!((small.width, small.height), (18, 18));
        assert_eq!((large.width, large.height), (54, 54));
        assert_ne!(small.rgba.len(), large.rgba.len());
        assert!(has_visible_pixel(&large));
    }

    #[test]
    fn editor_pages_template_icons_have_readable_16px_raster_footprints() {
        let preview = crate::ui::retained_host::primitives::Image::default();
        for icon in EDITOR_PAGES_WIRED_TEMPLATE_ICONS {
            let pixels = template_image_pixels(&preview, "", icon, 16, 16, Some(ICON_TINT), false)
                .unwrap_or_else(|| panic!("{icon} should render through the template icon path"));
            let footprint = icon_readability_footprint(&pixels)
                .unwrap_or_else(|| panic!("{icon} should produce visible 16px pixels"));

            println!(
                "ICON_16PX_READABILITY icon={icon} visible={} span={}x{}",
                footprint.visible_pixels, footprint.span_width, footprint.span_height
            );
            assert_eq!((pixels.width, pixels.height), (16, 16));
            assert!(
                footprint.visible_pixels >= 12,
                "{icon} produced only {} visible pixels at 16px",
                footprint.visible_pixels
            );
            assert!(
                footprint.span_width >= 6 && footprint.span_height >= 6,
                "{icon} collapsed to a {}x{} footprint at 16px",
                footprint.span_width,
                footprint.span_height
            );
            assert!(
                footprint.visible_pixels < (pixels.width * pixels.height) as usize,
                "{icon} filled the whole 16px slot instead of a readable icon silhouette"
            );
        }
    }

    #[test]
    fn mui_material_icon_modules_render_from_local_dev_source() {
        let add = UiVisualAssetRef::Icon("mui:Add".to_string());
        let add_pixels = load_visual_asset_pixels_for_size(&add, 24, 24)
            .expect("MUI Add icon should render from the local dev source");
        let add_large_pixels = load_visual_asset_pixels_for_size(&add, 36, 36)
            .expect("MUI Add icon should re-render when target size changes");
        assert_eq!((add_pixels.width, add_pixels.height), (24, 24));
        assert_eq!((add_large_pixels.width, add_large_pixels.height), (36, 36));
        assert!(has_visible_pixel(&add_pixels));

        let search = UiVisualAssetRef::Icon("@mui/icons-material/Search".to_string());
        let search_pixels = load_visual_asset_pixels_for_size(&search, 32, 32)
            .expect("prefixed MUI Search icon should render from the local dev source");
        assert_eq!((search_pixels.width, search_pixels.height), (32, 32));
        assert!(has_visible_pixel(&search_pixels));
    }

    #[test]
    fn template_mui_icon_ligatures_render_from_local_dev_source() {
        let preview = crate::ui::retained_host::primitives::Image::default();

        let folder = template_image_pixels(&preview, "", "folder", 24, 24, Some(ICON_TINT), false)
            .expect("MUI Icon ligature should resolve to the local Material icon module");
        let add_circle =
            template_image_pixels(&preview, "", "add_circle", 32, 32, Some(ICON_TINT), false)
                .expect("snake-case MUI Icon ligature should resolve to PascalCase module source");

        assert_eq!((folder.width, folder.height), (24, 24));
        assert_eq!((add_circle.width, add_circle.height), (32, 32));
        assert!(has_visible_pixel(&folder));
        assert!(has_visible_pixel(&add_circle));
    }

    #[test]
    fn template_missing_icon_pixels_keep_visible_fallback() {
        let preview = crate::ui::retained_host::primitives::Image::default();

        let missing = template_image_pixels(
            &preview,
            "",
            "missing_zircon_mui_icon",
            20,
            20,
            Some(ICON_TINT_ERROR),
            false,
        )
        .expect("missing template icons should produce deterministic fallback pixels");

        assert_eq!((missing.width, missing.height), (20, 20));
        assert!(missing
            .rgba
            .chunks_exact(4)
            .any(|pixel| pixel == ICON_TINT_ERROR.as_slice()));
    }

    #[test]
    fn svg_font_scan_is_reserved_for_text_svg() {
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

    #[test]
    fn template_plain_image_can_use_projected_preview_pixels_as_authority() {
        let preview = solid_preview_image([201, 42, 33, 255]);

        let image = template_image_pixels(
            &preview,
            "ui/editor/showcase_checker.svg",
            "",
            32,
            32,
            None,
            true,
        )
        .expect("plain Image nodes should consume projected preview pixels");

        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(&image.rgba[0..4], &[201, 42, 33, 255]);
    }

    #[test]
    fn template_icon_tint_uses_material_state_priority() {
        assert_eq!(
            template_image_tint(true, true, true, "error", "error", Some(ICON_TINT_ACTIVE)),
            Some(ICON_TINT_DISABLED)
        );
        assert_eq!(
            template_image_tint(true, true, false, "", "error", Some(ICON_TINT_ACTIVE)),
            Some(ICON_TINT_ERROR)
        );
        assert_eq!(
            template_image_tint(
                true,
                true,
                false,
                "warning",
                "normal",
                Some(ICON_TINT_ACTIVE),
            ),
            Some(ICON_TINT_WARNING)
        );
        assert_eq!(
            template_image_tint(true, true, false, "", "normal", Some(ICON_TINT_ERROR)),
            Some(ICON_TINT_ERROR)
        );
        assert_eq!(
            template_image_tint(true, true, false, "", "normal", None),
            Some(ICON_TINT_ACTIVE)
        );
        assert_eq!(
            template_image_tint(false, true, false, "error", "error", Some(ICON_TINT_ERROR)),
            None
        );
    }

    fn has_visible_pixel(image: &HostPaintImagePixels) -> bool {
        image.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0)
    }

    const EDITOR_PAGES_WIRED_TEMPLATE_ICONS: &[&str] = &[
        "editor_pages/workbench/menu/open-project.svg",
        "editor_pages/workbench/menu/save-all.svg",
        "editor_pages/workbench/dock/reset-layout.svg",
        "editor_pages/asset_browser/navigation/folder.svg",
        "editor_pages/hierarchy/entity/scene.svg",
        "editor_pages/console_profiler/logs/log-info.svg",
        "editor_pages/scene_viewport/tools/universal-transform.svg",
        "editor_pages/scene_viewport/display/lit.svg",
        "editor_pages/scene_viewport/display/grid-overlay.svg",
        "editor_pages/scene_viewport/snapping/grid-snap.svg",
        "editor_pages/scene_viewport/snapping/angle-snap.svg",
        "editor_pages/scene_viewport/snapping/scale-snap.svg",
        "editor_pages/scene_viewport/display/gizmo-visibility.svg",
        "editor_pages/scene_viewport/camera/frame-selection.svg",
        "editor_pages/scene_viewport/play/play.svg",
        "editor_pages/scene_viewport/play/stop.svg",
        "editor_pages/scene_viewport/camera/perspective.svg",
        "editor_pages/asset_browser/import_pipeline/import-settings.svg",
        "editor_pages/asset_browser/references/reference.svg",
        "editor_pages/asset_browser/navigation/search.svg",
        "editor_pages/asset_browser/import_pipeline/import.svg",
        "editor_pages/asset_browser/navigation/recent.svg",
        "editor_pages/workbench/tabs/close-tab.svg",
        "editor_pages/graph_editor/nodes/state-node.svg",
        "editor_pages/animation_timeline/transport/timeline-play.svg",
        "editor_pages/console_profiler/profiling/frame-time.svg",
        "editor_pages/console_profiler/diagnostics/watch.svg",
        "editor_pages/build_plugins/package/package.svg",
        "editor_pages/build_plugins/plugins/plugin.svg",
    ];

    struct IconReadabilityFootprint {
        visible_pixels: usize,
        span_width: u32,
        span_height: u32,
    }

    fn icon_readability_footprint(
        image: &HostPaintImagePixels,
    ) -> Option<IconReadabilityFootprint> {
        let mut visible_pixels = 0usize;
        let mut min_x = image.width;
        let mut min_y = image.height;
        let mut max_x = 0u32;
        let mut max_y = 0u32;

        for y in 0..image.height {
            for x in 0..image.width {
                let alpha = image.rgba[((y * image.width + x) as usize * 4) + 3];
                if alpha == 0 {
                    continue;
                }
                visible_pixels += 1;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        (visible_pixels > 0).then_some(IconReadabilityFootprint {
            visible_pixels,
            span_width: max_x - min_x + 1,
            span_height: max_y - min_y + 1,
        })
    }

    fn solid_preview_image(color: [u8; 4]) -> crate::ui::retained_host::primitives::Image {
        let pixels = [color, color, color, color].concat();
        crate::ui::retained_host::primitives::Image::from_rgba8(
            crate::ui::retained_host::primitives::SharedPixelBuffer::<
                crate::ui::retained_host::primitives::Rgba8Pixel,
            >::clone_from_slice(&pixels, 2, 2),
        )
    }
}
