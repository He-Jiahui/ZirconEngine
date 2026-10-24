use super::super::super::data::{FrameRect, HostViewportImageSet, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{
    draw_gpu_image_clipped_with_resource_key, draw_shared_rgba_image_clipped_with_resource_key,
};

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_images: &HostViewportImageSet,
    surface_key: Option<&str>,
) -> bool {
    if pane.kind.as_str() == "Game" && std::env::var_os("ZIRCON_TRACE_EDITOR_INTERACTION").is_some()
    {
        static SAMPLES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        if SAMPLES.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 8 {
            eprintln!(
                "editor_interaction kind=game_paint available={} body={:?} clip={:?} image={:?}",
                viewport_images.game().is_some_and(|image| image.is_valid()),
                body,
                clip,
                viewport_images.game().map(|image| (
                    image.resource_key.as_str(),
                    image.width,
                    image.height,
                    image
                        .rgba()
                        .map(|rgba| (rgba.get(..4), rgba.get(rgba.len() / 2..rgba.len() / 2 + 4)))
                ))
            );
        }
    }
    let Some(image) = viewport_images
        .for_surface(surface_key.unwrap_or(""), pane.kind.as_str())
        .filter(|image| image.is_valid())
    else {
        return false;
    };
    let drew_base = match image.rgba() {
        Some(rgba) => draw_shared_rgba_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.resource_generation,
            image.width,
            image.height,
            rgba,
        ),
        None => draw_gpu_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.resource_generation,
            image.width,
            image.height,
        ),
    };
    if drew_base {
        if let Some(overlay) = image.overlay() {
            let scale_x = body.width / image.width as f32;
            let scale_y = body.height / image.height as f32;
            let overlay_frame = FrameRect {
                x: body.x + overlay.x as f32 * scale_x,
                y: body.y + overlay.y as f32 * scale_y,
                width: overlay.width as f32 * scale_x,
                height: overlay.height as f32 * scale_y,
            };
            draw_shared_rgba_image_clipped_with_resource_key(
                frame,
                overlay_frame,
                Some(clip),
                overlay.resource_key.as_str(),
                0,
                overlay.width,
                overlay.height,
                &overlay.rgba,
            );
        }
    }
    true
}

#[cfg(test)]
#[path = "tests/viewport.rs"]
mod tests;
