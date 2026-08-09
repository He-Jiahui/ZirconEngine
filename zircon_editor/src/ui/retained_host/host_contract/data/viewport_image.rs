use std::sync::Arc;

use crate::scene::viewport::{CapturedFrame, RenderViewportHandle, RenderViewportProduct};

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageData {
    pub(crate) resource_key: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Option<Arc<[u8]>>,
}

impl HostViewportImageData {
    pub(crate) fn from_captured_frame(
        viewport: RenderViewportHandle,
        frame: CapturedFrame,
    ) -> Option<Self> {
        let width = frame.width;
        let height = frame.height;
        let generation = frame.generation;
        let image = Self {
            resource_key: viewport_image_resource_key(viewport, generation),
            width,
            height,
            rgba: Some(frame.rgba.into()),
        };
        image.is_valid().then_some(image)
    }

    pub(crate) fn from_viewport_product(product: RenderViewportProduct) -> Option<Self> {
        let image = Self {
            resource_key: product.resource_key().to_string(),
            width: product.width(),
            height: product.height(),
            rgba: None,
        };
        (product.is_valid() && image.is_valid()).then_some(image)
    }

    pub(crate) fn rgba(&self) -> Option<&[u8]> {
        self.rgba.as_deref()
    }

    pub(crate) fn is_valid(&self) -> bool {
        // GPU texture cache entries are keyed by resource_key, so a drawable
        // viewport image must never use the empty default key.
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self
                .rgba
                .as_ref()
                .is_none_or(|rgba| rgba.len() == self.width as usize * self.height as usize * 4)
    }
}

fn viewport_image_resource_key(viewport: RenderViewportHandle, generation: u64) -> String {
    format!("viewport:{}:{generation}", viewport.raw())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn viewport_image_resource_key_tracks_viewport_generation() {
        let red = viewport_image(3, 7, &[255, 0, 0, 255]);
        let blue = viewport_image(3, 8, &[0, 0, 255, 255]);

        assert_ne!(red.resource_key, blue.resource_key);
        assert_eq!(red.resource_key, "viewport:3:7");
        assert_eq!(blue.resource_key, "viewport:3:8");
    }

    #[test]
    fn viewport_image_requires_resource_key_to_be_valid() {
        let image = HostViewportImageData {
            resource_key: String::new(),
            width: 1,
            height: 1,
            rgba: Some(vec![255, 255, 255, 255].into()),
        };

        assert!(!image.is_valid());
    }

    #[test]
    fn viewport_image_clones_share_the_captured_rgba_payload() {
        let image = viewport_image(3, 7, &[255, 0, 0, 255]);
        let cloned = image.clone();

        assert!(Arc::ptr_eq(
            image.rgba.as_ref().expect("capture payload"),
            cloned.rgba.as_ref().expect("capture payload"),
        ));
    }

    #[test]
    fn viewport_product_keeps_the_gpu_resource_key_without_cpu_pixels() {
        let product = RenderViewportProduct::new(RenderViewportHandle::new(3), 640, 360, 11);

        let image = HostViewportImageData::from_viewport_product(product)
            .expect("valid GPU product should transfer into host data");

        assert_eq!(image.resource_key, "viewport:3:11");
        assert!(image.rgba.is_none());
        assert!(image.is_valid());
    }

    fn viewport_image(viewport: u64, generation: u64, rgba: &[u8]) -> HostViewportImageData {
        HostViewportImageData::from_captured_frame(
            RenderViewportHandle::new(viewport),
            CapturedFrame::new(1, 1, rgba.to_vec(), generation),
        )
        .expect("valid capture should transfer into host data")
    }
}
