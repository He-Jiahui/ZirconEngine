use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use resvg::{tiny_skia, usvg};
use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use super::theme::PALETTE;

const ICON_TINT: [u8; 4] = PALETTE.text;
const ICON_TINT_ACTIVE: [u8; 4] = PALETTE.focus_ring;
const ICON_TINT_DISABLED: [u8; 4] = PALETTE.text_disabled;
const ICON_TINT_ERROR: [u8; 4] = PALETTE.error;
const ICON_TINT_WARNING: [u8; 4] = PALETTE.warning;
const MAX_VECTOR_RASTER_EDGE: u32 = 4096;

#[derive(Clone)]
pub(super) struct HostPaintImagePixels {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
}

#[derive(Clone, Copy)]
struct RasterTargetSize {
    width: u32,
    height: u32,
}

pub(super) fn slint_image_pixels(
    image: &slint::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let buffer = image.to_rgba8()?;
    let mut rgba = buffer.as_bytes().to_vec();
    if let Some(tint) = tint {
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        width: buffer.width(),
        height: buffer.height(),
        rgba,
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
    if active {
        return Some(ICON_TINT_ACTIVE);
    }
    Some(ICON_TINT)
}

pub(super) fn template_image_pixels(
    preview_image: &slint::Image,
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
    let preview_pixels = || slint_image_pixels(preview_image, tint);
    if prefer_preview_image {
        preview_pixels().or_else(source_pixels)
    } else {
        source_pixels().or_else(preview_pixels)
    }
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
            load_pixels_from_candidates(icon_candidates(icon_name), &key, target, Some(ICON_TINT))
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
    let path = first_existing_path(candidates)?;
    let key = image_pixels_cache_key(base_key, &path, target.filter(|_| is_svg_path(&path)), tint);
    let cache = VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(cached) = cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(&key)
    {
        return cached.clone();
    }

    let loaded = if is_svg_path(&path) {
        target
            .and_then(|target| render_svg_file_pixels(&path, target, tint))
            .or_else(|| {
                load_image_from_path(&path).and_then(|image| slint_image_pixels(&image, tint))
            })
    } else {
        load_image_from_path(&path).and_then(|image| slint_image_pixels(&image, tint))
    };

    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, loaded.clone());
    loaded
}

fn load_image_from_candidates(candidates: Vec<PathBuf>) -> Option<slint::Image> {
    for path in candidates {
        if let Some(image) = load_image_from_path(&path) {
            return Some(image);
        }
    }
    None
}

fn load_image_from_path(path: &Path) -> Option<slint::Image> {
    if !path.exists() {
        return None;
    }
    let image = slint::Image::load_from_path(path).unwrap_or_default();
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
    let svg = fs::read(path).ok()?;
    let mut options = usvg::Options {
        resources_dir: fs::canonicalize(path)
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf)),
        ..usvg::Options::default()
    };
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(&svg, &options).ok()?;
    let svg_size = tree.size();
    let transform = tiny_skia::Transform::from_scale(
        target.width as f32 / svg_size.width(),
        target.height as f32 / svg_size.height(),
    );
    let mut pixmap = tiny_skia::Pixmap::new(target.width, target.height)?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let mut rgba = pixmap.take_demultiplied();
    if let Some(tint) = tint {
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        width: target.width,
        height: target.height,
        rgba,
    };
    image.is_valid().then_some(image)
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
        self.width > 0
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
            template_image_tint(true, true, true, "error", "error"),
            Some(ICON_TINT_DISABLED)
        );
        assert_eq!(
            template_image_tint(true, true, false, "", "error"),
            Some(ICON_TINT_ERROR)
        );
        assert_eq!(
            template_image_tint(true, true, false, "warning", "normal"),
            Some(ICON_TINT_WARNING)
        );
        assert_eq!(
            template_image_tint(true, true, false, "", "normal"),
            Some(ICON_TINT_ACTIVE)
        );
        assert_eq!(
            template_image_tint(false, true, false, "error", "error"),
            None
        );
    }

    fn has_visible_pixel(image: &HostPaintImagePixels) -> bool {
        image.rgba.chunks_exact(4).any(|pixel| pixel[3] > 0)
    }

    fn solid_preview_image(color: [u8; 4]) -> slint::Image {
        let pixels = [color, color, color, color].concat();
        slint::Image::from_rgba8(
            slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(&pixels, 2, 2),
        )
    }
}
