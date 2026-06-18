use super::{
    ChromeCommand, ChromeCommandKind, ChromeCommandStream, ChromeImagePayload, ChromeImageUvRect,
};
use crate::ui::retained_host::host_contract::chrome_command_stream::atlas::atlas_subimage_rgba;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::{
    draw_rect_clipped, draw_rgba_image_clipped_with_resource_key, draw_rounded_border_clipped,
    draw_rounded_rect_clipped,
};
use crate::ui::retained_host::host_contract::paint_text::draw_text_with_size_and_style;

const FALLBACK_IMAGE_COLOR: [u8; 4] = [42, 58, 78, 255];

pub(in crate::ui::retained_host::host_contract) fn paint_chrome_command_stream_to_frame(
    width: u32,
    height: u32,
    stream: &ChromeCommandStream,
) -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    paint_chrome_command_stream_into_frame(&mut frame, stream);
    frame
}

pub(in crate::ui::retained_host::host_contract) fn repaint_chrome_command_stream_region(
    frame: &mut HostRgbaFrame,
    stream: &ChromeCommandStream,
) -> Option<FrameRect> {
    let damage = stream.damage().cloned()?;
    let previous_clip = frame.replace_paint_clip(Some(damage.clone()));
    paint_chrome_command_stream_into_frame(frame, stream);
    frame.replace_paint_clip(previous_clip);
    Some(damage)
}

fn paint_chrome_command_stream_into_frame(frame: &mut HostRgbaFrame, stream: &ChromeCommandStream) {
    let mut ordered = stream.commands().iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    for (_, command) in ordered {
        paint_chrome_command(frame, command);
    }
}

fn paint_chrome_command(frame: &mut HostRgbaFrame, command: &ChromeCommand) {
    match &command.kind {
        ChromeCommandKind::Quad {
            color,
            corner_radius,
        } => {
            if *corner_radius > 0.0 {
                draw_rounded_rect_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    *color,
                    *corner_radius,
                )
            } else {
                draw_rect_clipped(frame, command.frame.clone(), command.clip.as_ref(), *color)
            }
        }
        ChromeCommandKind::Border {
            color,
            width,
            corner_radius,
        } => {
            if *corner_radius > 0.0 {
                draw_rounded_border_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    *color,
                    *width,
                    *corner_radius,
                )
            } else {
                paint_border_command(frame, &command.frame, command.clip.as_ref(), *color, *width)
            }
        }
        ChromeCommandKind::Text {
            text,
            color,
            size,
            line_height,
            style,
        } => draw_text_with_size_and_style(
            frame,
            command.frame.clone(),
            text,
            command.clip.as_ref(),
            *color,
            *size,
            *line_height,
            *style,
        ),
        ChromeCommandKind::Image { payload } => {
            if let Some(rgba) = payload.rgba.as_ref() {
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
                    draw_rect_clipped(
                        frame,
                        command.frame.clone(),
                        command.clip.as_ref(),
                        FALLBACK_IMAGE_COLOR,
                    );
                }
            } else {
                draw_rect_clipped(
                    frame,
                    command.frame.clone(),
                    command.clip.as_ref(),
                    FALLBACK_IMAGE_COLOR,
                );
            }
        }
        ChromeCommandKind::Clip => {}
    }
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

fn paint_border_command(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    width: f32,
) {
    let width = width.ceil().max(1.0);
    for offset in 0..(width as u32) {
        let offset = offset as f32;
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + offset,
                width: (rect.width - offset * 2.0).max(0.0),
                height: 1.0,
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + rect.height - 1.0 - offset,
                width: (rect.width - offset * 2.0).max(0.0),
                height: 1.0,
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + offset,
                y: rect.y + offset,
                width: 1.0,
                height: (rect.height - offset * 2.0).max(0.0),
            },
            clip,
            color,
        );
        draw_rect_clipped(
            frame,
            FrameRect {
                x: rect.x + rect.width - 1.0 - offset,
                y: rect.y + offset,
                width: 1.0,
                height: (rect.height - offset * 2.0).max(0.0),
            },
            clip,
            color,
        );
    }
}
