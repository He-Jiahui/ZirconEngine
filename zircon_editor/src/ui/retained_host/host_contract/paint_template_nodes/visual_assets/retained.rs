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
mod tests {
    use std::sync::Arc;

    use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

    use super::{retained_image_pixels, retained_image_resource_key};

    #[test]
    fn untinted_retained_image_reuses_the_precomputed_pixel_product() {
        let image = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &[1, 2, 3, 255],
            1,
            1,
        ));
        let product = image
            .pixel_product()
            .expect("fixture image should publish a pixel product");

        let retained = retained_image_pixels(&image, None)
            .expect("fixture image should produce retained paint pixels");

        assert!(Arc::ptr_eq(product.rgba, &retained.rgba));
        assert_eq!(
            retained.resource_key,
            retained_image_resource_key(1, 1, &[1, 2, 3, 255])
        );
    }

    #[test]
    fn tinted_retained_image_reuses_the_shared_visual_variant_cache() {
        let image = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &[1, 2, 3, 255, 4, 5, 6, 128],
            2,
            1,
        ));

        let first = retained_image_pixels(&image, Some([10, 20, 30, 255]))
            .expect("tinted image should produce retained paint pixels");
        let repeated = retained_image_pixels(&image, Some([10, 20, 30, 255]))
            .expect("stable tint should hit the shared visual cache");
        let recolored = retained_image_pixels(&image, Some([30, 20, 10, 255]))
            .expect("different tint should produce another retained variant");

        assert!(Arc::ptr_eq(&first.rgba, &repeated.rgba));
        assert_eq!(first.resource_key, repeated.resource_key);
        assert_eq!(first.rgba.as_ref(), &[10, 20, 30, 255, 10, 20, 30, 128]);
        assert!(!Arc::ptr_eq(&first.rgba, &recolored.rgba));
        assert_ne!(first.resource_key, recolored.resource_key);
    }

    #[test]
    fn tinted_retained_image_cache_separates_content_and_dimensions() {
        let tint = Some([10, 20, 30, 255]);
        let horizontal = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &[1, 2, 3, 255, 4, 5, 6, 128],
            2,
            1,
        ));
        let vertical = Image::from_rgba8(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
            &[1, 2, 3, 255, 4, 5, 6, 128],
            1,
            2,
        ));
        let different_content = Image::from_rgba8(
            SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&[9, 8, 7, 255, 6, 5, 4, 64], 2, 1),
        );

        let horizontal = retained_image_pixels(&horizontal, tint)
            .expect("horizontal image should produce tinted pixels");
        let vertical = retained_image_pixels(&vertical, tint)
            .expect("vertical image should produce tinted pixels");
        let different_content = retained_image_pixels(&different_content, tint)
            .expect("different content should produce tinted pixels");

        assert_eq!((horizontal.width, horizontal.height), (2, 1));
        assert_eq!((vertical.width, vertical.height), (1, 2));
        assert_ne!(horizontal.resource_key, vertical.resource_key);
        assert!(!Arc::ptr_eq(&horizontal.rgba, &vertical.rgba));
        assert_ne!(horizontal.resource_key, different_content.resource_key);
        assert!(!Arc::ptr_eq(&horizontal.rgba, &different_content.rgba));
    }
}
