use crate::scene::viewport::{CapturedFrame, RenderViewportHandle};

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageData {
    pub(crate) resource_key: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
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
            rgba: frame.rgba,
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

fn viewport_image_resource_key(viewport: RenderViewportHandle, generation: u64) -> String {
    format!("viewport:{}:{generation}", viewport.raw())
}

#[cfg(test)]
mod tests {
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
            rgba: vec![255, 255, 255, 255],
        };

        assert!(!image.is_valid());
    }

    fn viewport_image(viewport: u64, generation: u64, rgba: &[u8]) -> HostViewportImageData {
        HostViewportImageData::from_captured_frame(
            RenderViewportHandle::new(viewport),
            CapturedFrame::new(1, 1, rgba.to_vec(), generation),
        )
        .expect("valid capture should transfer into host data")
    }
}
