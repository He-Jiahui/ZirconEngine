use super::keys::retained_image_resource_key;
use super::pixels::HostPaintImagePixels;
use super::tint::tint_non_transparent_pixels;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn retained_image_pixels(
    image: &crate::ui::retained_host::primitives::Image,
    tint: Option<[u8; 4]>,
) -> Option<HostPaintImagePixels> {
    let buffer = image.to_rgba8()?;
    let mut rgba = buffer.as_bytes().to_vec();
    if let Some(tint) = tint {
        tint_non_transparent_pixels(&mut rgba, tint);
    }
    let image = HostPaintImagePixels {
        resource_key: retained_image_resource_key(buffer.width(), buffer.height(), &rgba),
        width: buffer.width(),
        height: buffer.height(),
        rgba: rgba.into(),
        atlas: None,
    };
    image.is_valid().then_some(image)
}
