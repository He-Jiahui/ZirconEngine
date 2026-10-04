use std::sync::Arc;

use super::keys::{retained_image_resource_key, retained_image_resource_key_from_fingerprint};
use super::loading::{
    cached_visual_asset_pixels, image_pixels_cache_key, store_visual_asset_pixels,
};
use super::pixels::HostPaintImagePixels;
use super::tint::tint_non_transparent_pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_pixels(
    image: &crate::ui::retained_host::primitives::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let product = image.pixel_product()?;
    let base_key = retained_image_resource_key_from_fingerprint(
        product.width,
        product.height,
        product.content_fingerprint,
    );
    if let Some(tint) = tint {
        let cache_key = image_pixels_cache_key(&base_key, None, Some(tint));
        if let Some(cached) = cached_visual_asset_pixels(&cache_key) {
            return cached;
        }
        let mut rgba = product.rgba.as_ref().to_vec();
        tint_non_transparent_pixels(&mut rgba, tint);
        let image = HostPaintImagePixels {
            resource_key: retained_image_resource_key(product.width, product.height, &rgba),
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
        resource_key: base_key,
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
