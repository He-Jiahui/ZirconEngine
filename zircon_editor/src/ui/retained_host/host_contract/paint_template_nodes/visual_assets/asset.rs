use zircon_runtime_interface::ui::surface::UiVisualAssetRef;

use super::candidates::{icon_candidates, image_candidates};
use super::keys::visual_asset_cache_key;
use super::loading::{load_pixels_from_candidates, missing_icon_pixels};
use super::pixels::HostPaintImagePixels;
use super::target::{RasterTargetSize, MUI_ICON_DEFAULT_EDGE};
use super::tint::ICON_TINT;
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_visual_asset_pixels(
    asset: &UiVisualAssetRef,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(asset, None, None, false)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_visual_asset_pixels_for_size(
    asset: &UiVisualAssetRef,
    target_width: u32,
    target_height: u32,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(
        asset,
        RasterTargetSize::new(target_width, target_height),
        damage_frame,
        false,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_vector_visual_asset_pixels_for_size(
    asset: &UiVisualAssetRef,
    target_width: u32,
    target_height: u32,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    load_visual_asset_pixels_for_target(
        asset,
        RasterTargetSize::new(target_width, target_height),
        damage_frame,
        true,
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn load_existing_icon_asset_pixels_for_size(
    icon_name: &str,
    target_width: u32,
    target_height: u32,
    tint: Option<[u8; 4]>,
    damage_frame: Option<FrameRect>,
) -> Option<HostPaintImagePixels> {
    let target = RasterTargetSize::new(target_width, target_height)?;
    let target = vector_cache_target(Some(target), icon_source_is_vector(icon_name))?;
    let key = visual_asset_cache_key(&UiVisualAssetRef::Icon(icon_name.to_owned()));
    load_pixels_from_candidates(
        || icon_candidates(icon_name),
        &key,
        Some(target),
        tint,
        damage_frame,
    )
}

fn load_visual_asset_pixels_for_target(
    asset: &UiVisualAssetRef,
    target: Option<RasterTargetSize>,
    damage_frame: Option<FrameRect>,
    explicit_vector: bool,
) -> Option<HostPaintImagePixels> {
    let key = visual_asset_cache_key(asset);
    let target = vector_cache_target(
        target,
        explicit_vector
            || matches!(asset, UiVisualAssetRef::Icon(icon_name) if icon_source_is_vector(icon_name))
            || matches!(asset, UiVisualAssetRef::Image(source) if source_is_svg(source)),
    );
    match asset {
        UiVisualAssetRef::Icon(icon_name) => {
            let target = target.unwrap_or(RasterTargetSize {
                width: MUI_ICON_DEFAULT_EDGE,
                height: MUI_ICON_DEFAULT_EDGE,
            });
            load_pixels_from_candidates(
                || icon_candidates(icon_name),
                &key,
                Some(target),
                Some(ICON_TINT),
                damage_frame,
            )
            .or_else(|| missing_icon_pixels(&key, target, Some(ICON_TINT)))
        }
        UiVisualAssetRef::Image(source) => load_pixels_from_candidates(
            || image_candidates(source),
            &key,
            target,
            None,
            damage_frame,
        ),
    }
}

pub(super) fn vector_cache_target(
    target: Option<RasterTargetSize>,
    is_vector: bool,
) -> Option<RasterTargetSize> {
    if is_vector {
        target.map(RasterTargetSize::vector_cache_bucket)
    } else {
        target
    }
}

pub(super) fn source_is_svg(source: &str) -> bool {
    source_extension(source).is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
}

pub(super) fn icon_source_is_vector(icon_name: &str) -> bool {
    let icon_name = icon_name.trim();
    !icon_name.is_empty() && !source_is_known_raster(icon_name)
}

fn source_is_known_raster(source: &str) -> bool {
    source_extension(source).is_some_and(|extension| {
        [
            "png", "jpg", "jpeg", "webp", "bmp", "gif", "ico", "tga", "tif", "tiff",
        ]
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
    })
}

fn source_extension(source: &str) -> Option<&str> {
    source
        .split(|character| character == '?' || character == '#')
        .next()
        .and_then(|path| path.rsplit_once('.'))
        .map(|(_, extension)| extension)
}

#[cfg(test)]
mod tests {
    use super::{icon_source_is_vector, source_is_svg, vector_cache_target, RasterTargetSize};

    #[test]
    fn svg_source_detection_accepts_resource_suffixes_without_classifying_bitmaps() {
        assert!(source_is_svg("res://icons/save.SVG#theme=dark"));
        assert!(source_is_svg("asset://icons/save.svg?generation=7"));
        assert!(!source_is_svg("asset://icons/save.png"));
        assert!(!source_is_svg("asset://icons/svg-preview.png"));
    }

    #[test]
    fn semantic_icons_are_vector_but_explicit_bitmap_icons_remain_exact() {
        assert!(icon_source_is_vector("folder-open-outline"));
        assert!(icon_source_is_vector("toolbar/save.svg?theme=dark"));
        assert!(!icon_source_is_vector("thumbnails/save.PNG#g=7"));
        assert!(!icon_source_is_vector(""));
    }

    #[test]
    fn small_vector_and_bitmap_targets_preserve_the_exact_device_pixel_extent() {
        let target = RasterTargetSize::new(17, 19);

        assert_eq!(vector_cache_target(target, false), target);
        assert_eq!(vector_cache_target(target, true), target);
    }

    #[test]
    fn non_square_vector_targets_preserve_exact_device_pixel_geometry() {
        let target = RasterTargetSize::new(41, 43);

        assert_eq!(vector_cache_target(target, true), target);
    }
}
