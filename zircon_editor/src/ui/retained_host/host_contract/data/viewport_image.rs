use crate::ui::retained_host::primitives::Image;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageData {
    pub(crate) resource_key: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl HostViewportImageData {
    pub(crate) fn from_image(image: &Image) -> Option<Self> {
        let buffer = image.to_rgba8()?;
        let rgba = buffer.as_bytes().to_vec();
        let image = Self {
            resource_key: viewport_image_resource_key(buffer.width(), buffer.height(), &rgba),
            width: buffer.width(),
            height: buffer.height(),
            rgba,
        };
        image.is_valid().then_some(image)
    }

    pub(crate) fn is_valid(&self) -> bool {
        // GPU texture cache entries are keyed by resource_key, so a drawable
        // viewport image must never use the empty default key.
        !self.resource_key.is_empty()
            && self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }
}

fn viewport_image_resource_key(width: u32, height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    width.hash(&mut hasher);
    height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("viewport:{width}x{height}:{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::primitives::{Image, Rgba8Pixel, SharedPixelBuffer};

    #[test]
    fn viewport_image_resource_key_tracks_same_size_content() {
        let red = viewport_image(&[255, 0, 0, 255]);
        let blue = viewport_image(&[0, 0, 255, 255]);

        assert_ne!(red.resource_key, blue.resource_key);
        assert!(red.resource_key.starts_with("viewport:1x1:"));
        assert!(blue.resource_key.starts_with("viewport:1x1:"));
    }

    #[test]
    fn viewport_image_requires_resource_key_to_be_valid() {
        let image = HostViewportImageData {
            resource_key: String::new(),
            width: 1,
            height: 1,
            rgba: vec![255, 255, 255, 255],
        };

        assert!(!image.is_valid());
    }

    fn viewport_image(rgba: &[u8]) -> HostViewportImageData {
        let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba, 1, 1);
        let image = Image::from_rgba8(buffer);
        HostViewportImageData::from_image(&image).expect("valid image should project")
    }
}
