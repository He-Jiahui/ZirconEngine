use super::super::super::data::{FrameRect, HostViewportImageData, PaneData};
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_primitives::{
    draw_gpu_image_clipped_with_resource_key, draw_rgba_image_clipped_with_resource_key,
};

pub(in crate::ui::retained_host::host_contract) fn draw_viewport_image(
    frame: &mut HostRgbaFrame,
    pane: &PaneData,
    body: &FrameRect,
    clip: &FrameRect,
    viewport_image: Option<&HostViewportImageData>,
) -> bool {
    if !matches!(pane.kind.as_str(), "Scene" | "Game") {
        return false;
    }
    let Some(image) = viewport_image.filter(|image| image.is_valid()) else {
        return false;
    };
    match image.rgba() {
        Some(rgba) => draw_rgba_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.width,
            image.height,
            rgba,
        ),
        None => draw_gpu_image_clipped_with_resource_key(
            frame,
            body.clone(),
            Some(clip),
            image.resource_key.as_str(),
            image.width,
            image.height,
        ),
    }
}
