use super::super::super::{
    ChromeCommand, ChromeImagePayload, ChromeImageUvRect, atlas::atlas_subimage_rgba,
};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::{
    draw_rect_clipped, draw_rgba_image_clipped_with_resource_key,
};

const FALLBACK_IMAGE_COLOR: [u8; 4] = [42, 58, 78, 255];

pub(super) fn paint_image_command(
    frame: &mut HostRgbaFrame,
    command: &ChromeCommand,
    payload: &ChromeImagePayload,
) {
    let Some(rgba) = payload.rgba.as_ref() else {
        paint_fallback_image(frame, command);
        return;
    };
    let painted = if let Some(atlas_uv) = payload.atlas_uv {
        paint_atlas_image_payload(frame, command, payload, rgba, atlas_uv)
    } else {
        draw_rgba_image_clipped_with_resource_key(
            frame,
            command.frame.clone(),
            command.clip.as_ref(),
            payload.resource_key.as_str(),
            payload.width,
            payload.height,
            rgba,
        )
    };
    if !painted {
        paint_fallback_image(frame, command);
    }
}

fn paint_fallback_image(frame: &mut HostRgbaFrame, command: &ChromeCommand) {
    draw_rect_clipped(
        frame,
        command.frame.clone(),
        command.clip.as_ref(),
        FALLBACK_IMAGE_COLOR,
    );
}

fn paint_atlas_image_payload(
    frame: &mut HostRgbaFrame,
    command: &ChromeCommand,
    payload: &ChromeImagePayload,
    rgba: &[u8],
    atlas_uv: ChromeImageUvRect,
) -> bool {
    let Some((width, height, subimage)) =
        atlas_subimage_rgba(payload.width, payload.height, rgba, atlas_uv)
    else {
        return false;
    };
    draw_rgba_image_clipped_with_resource_key(
        frame,
        command.frame.clone(),
        command.clip.as_ref(),
        payload.resource_key.as_str(),
        width,
        height,
        &subimage,
    )
}
