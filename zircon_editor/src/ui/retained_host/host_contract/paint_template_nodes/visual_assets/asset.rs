use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use super::candidates::{icon_candidates, image_candidates};
use super::keys::visual_asset_cache_key;
use super::loading::{load_pixels_from_candidates, missing_icon_pixels};
use super::pixels::HostPaintImagePixels;
use super::target::{RasterTargetSize, MUI_ICON_DEFAULT_EDGE};
use super::tint::ICON_TINT;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_visual_asset_pixels(
    asset: &UiVisualAssetRef,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, None)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_visual_asset_pixels_for_size(
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
