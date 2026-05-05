use slint::Image;

#[derive(Clone, Default)]
pub(crate) struct HostViewportImageData {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl HostViewportImageData {
    pub(crate) fn from_image(image: &Image) -> Option<Self> {
        let buffer = image.to_rgba8()?;
        let rgba = buffer.as_bytes().to_vec();
        let image = Self {
            width: buffer.width(),
            height: buffer.height(),
            rgba,
        };
        image.is_valid().then_some(image)
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.rgba.len() == self.width as usize * self.height as usize * 4
    }
}
