use std::sync::Arc;

use super::super::super::super::visual_assets::HostPaintImagePixels;

pub(super) fn image_pixels_with_opacity(image: &HostPaintImagePixels, opacity: f32) -> Arc<[u8]> {
    pixels_with_opacity(&image.rgba, opacity)
}

fn pixels_with_opacity(source: &Arc<[u8]>, opacity: f32) -> Arc<[u8]> {
    let opacity = opacity.clamp(0.0, 1.0);
    if opacity == 1.0 {
        return Arc::clone(source);
    }
    let mut rgba = Arc::<[u8]>::from(source.as_ref());
    let pixels = Arc::get_mut(&mut rgba).expect("new pixel storage has one owner");
    for pixel in pixels.chunks_exact_mut(4) {
        pixel[3] = ((pixel[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    }
    rgba
}

#[cfg(test)]
#[path = "pixels/direct_arc_opacity_tests.rs"]
mod direct_arc_opacity_tests;
