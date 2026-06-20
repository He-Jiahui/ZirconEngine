use super::candidates::template_image_candidates;
use super::keys::template_image_cache_key;
use super::loading::{load_pixels_from_candidates, missing_icon_pixels};
use super::pixels::HostPaintImagePixels;
use super::retained::retained_image_pixels;
use super::target::RasterTargetSize;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn template_image_pixels(
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
