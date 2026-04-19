use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use crate::scene::viewport::CapturedFrame;

pub(super) fn import_frame_image(frame: &CapturedFrame) -> Result<(u64, Image), String> {
    if frame.width == 0 || frame.height == 0 {
        return Err("render framework returned a zero-sized viewport frame".to_string());
    }

    let expected_len = frame.width as usize * frame.height as usize * 4;
    if frame.rgba.len() != expected_len {
        return Err(format!(
            "render framework returned {} RGBA bytes for a {}x{} frame",
            frame.rgba.len(),
            frame.width,
            frame.height
        ));
    }

    let buffer =
        SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&frame.rgba, frame.width, frame.height);
    Ok((frame.generation, Image::from_rgba8(buffer)))
}
