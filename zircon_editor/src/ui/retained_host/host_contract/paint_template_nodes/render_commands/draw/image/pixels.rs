use super::super::super::super::visual_assets::HostPaintImagePixels;

pub(super) fn image_pixels_with_opacity(image: &HostPaintImagePixels, opacity: f32) -> Vec<u8> {
    let opacity = opacity.clamp(0.0, 1.0);
    let mut rgba = image.rgba.clone();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[3] = ((pixel[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    }
    rgba
}
