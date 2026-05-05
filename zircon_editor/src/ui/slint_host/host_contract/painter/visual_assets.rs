use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use zircon_runtime::asset::runtime_asset_path_with_dev_asset_root;
use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

const ICON_TINT: [u8; 4] = [210, 220, 235, 255];

#[derive(Clone)]
pub(super) struct HostPaintImagePixels {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
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

pub(super) fn template_image_tint(is_icon_like: bool) -> Option<[u8; 4]> {
    is_icon_like.then_some(ICON_TINT)
}

pub(super) fn load_visual_asset_pixels(
    asset: &UiVisualAssetRef,
) -> Option<HostPaintImagePixels> {
    let key = visual_asset_cache_key(asset);
    let cache = VISUAL_ASSET_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Some(cached) = cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(&key)
    {
        return cached.clone();
    }

    let loaded = match asset {
        UiVisualAssetRef::Icon(icon_name) => {
            load_image_from_candidates(icon_candidates(icon_name))
                .and_then(|image| slint_image_pixels(&image, Some(ICON_TINT)))
        }
        UiVisualAssetRef::Image(source) => load_image_from_candidates(image_candidates(source))
            .and_then(|image| slint_image_pixels(&image, None)),
    };
    cache
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key, loaded.clone());
    loaded
}

fn load_image_from_candidates(candidates: Vec<PathBuf>) -> Option<slint::Image> {
    for path in candidates {
        if path.exists() {
            let image = slint::Image::load_from_path(&path).unwrap_or_default();
            let size = image.size();
            if size.width > 0 && size.height > 0 {
                return Some(image);
            }
        }
    }
    None
}

fn image_candidates(source: &str) -> Vec<PathBuf> {
    let assets = editor_dev_asset_root();
    let mut candidates = Vec::new();
    if !source.is_empty() {
        candidates.push(runtime_asset_path_with_dev_asset_root(source, &assets));
        candidates.push(assets.join(strip_assets_prefix(source)));
        candidates.push(assets.join("icons").join(strip_assets_prefix(source)));
    }
    candidates
}

fn icon_candidates(icon_name: &str) -> Vec<PathBuf> {
    let assets = editor_dev_asset_root();
    let mut candidates = Vec::new();
    if !icon_name.is_empty() {
        candidates.push(assets.join("icons").join(strip_assets_prefix(icon_name)));
        if icon_name.ends_with(".svg") {
            candidates.push(assets.join("icons").join("ionicons").join(icon_name));
        } else {
            candidates.push(
                assets
                    .join("icons")
                    .join("ionicons")
                    .join(format!("{icon_name}.svg")),
            );
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

fn editor_dev_asset_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}

fn strip_assets_prefix(source: &str) -> &Path {
    let path = Path::new(source.trim_start_matches(|ch| ch == '/' || ch == '\\'));
    path.strip_prefix("assets").unwrap_or(path)
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

static VISUAL_ASSET_CACHE: OnceLock<Mutex<BTreeMap<String, Option<HostPaintImagePixels>>>> =
    OnceLock::new();
