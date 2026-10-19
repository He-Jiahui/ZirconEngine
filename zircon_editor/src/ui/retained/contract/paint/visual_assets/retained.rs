use std::sync::Arc;

use super::keys::retained_image_resource_key;
use super::loading::{
    cached_visual_asset_variant, image_pixels_cache_key, store_visual_asset_pixels,
};
use super::pixels::HostPaintImagePixels;
use super::tint::tint_non_transparent_pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_pixels(
    image: &crate::ui::retained_host::primitives::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let product = image.pixel_product()?;
    let base_key = product.resource_key.as_ref();
    if let Some(tint) = tint {
        if let Some(cached) = cached_visual_asset_variant(base_key, None, Some(tint)) {
            return cached;
        }
        let cache_key = image_pixels_cache_key(base_key, None, Some(tint));
        let mut rgba = product.rgba.as_ref().to_vec();
        tint_non_transparent_pixels(&mut rgba, tint);
        let image = HostPaintImagePixels {
            source_resource_key: None,
            source_path: None,
            resource_key: (retained_image_resource_key(product.width, product.height, &rgba))
                .into(),
            width: product.width,
            height: product.height,
            rgba: rgba.into(),
            atlas: None,
        };
        let image = image.is_valid().then_some(image);
        store_visual_asset_pixels(cache_key, &base_key, std::iter::empty(), image.clone());
        return image;
    }

    let image = HostPaintImagePixels {
        source_resource_key: None,
        source_path: None,
        resource_key: Arc::clone(product.resource_key),
        width: product.width,
        height: product.height,
        rgba: Arc::clone(product.rgba),
        atlas: None,
    };
    image.is_valid().then_some(image)
}

#[cfg(test)]
#[path = "tests/retained.rs"]
mod tests;
