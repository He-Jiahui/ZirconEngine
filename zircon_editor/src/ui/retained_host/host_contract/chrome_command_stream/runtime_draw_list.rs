use zircon_runtime::rhi::{
    UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceDrawList, UiSurfaceImagePayload,
    UiSurfaceImageUvRect, UiSurfaceRect, UiSurfaceTextStyle,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::{ChromeCommand, ChromeCommandKind, ChromeCommandStream, ChromeImageUvRect};
use crate::ui::retained_host::host_contract::data::FrameRect;

pub(in crate::ui::retained_host::host_contract) fn ui_surface_draw_list_from_stream(
    stream: &ChromeCommandStream,
) -> UiSurfaceDrawList {
    UiSurfaceDrawList::new(
        stream.surface_size(),
        stream.damage().map(ui_rect),
        stream
            .commands()
            .iter()
            .map(ui_surface_command_from_chrome)
            .collect(),
    )
}

fn ui_surface_command_from_chrome(command: &ChromeCommand) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index: command.z_index,
        frame: ui_rect(&command.frame),
        clip: command.clip.as_ref().map(ui_rect),
        kind: match &command.kind {
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            } => UiSurfaceCommandKind::Quad {
                color: *color,
                corner_radius: *corner_radius,
            },
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            } => UiSurfaceCommandKind::Border {
                color: *color,
                width: *width,
                corner_radius: *corner_radius,
            },
            ChromeCommandKind::Text {
                text,
                color,
                size,
                line_height,
                style,
            } => UiSurfaceCommandKind::Text {
                text: text.clone(),
                color: *color,
                font_size: *size,
                line_height: *line_height,
                style: ui_text_style(*style),
            },
            ChromeCommandKind::Image { payload } => UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: payload.resource_key.clone(),
                    width: payload.width,
                    height: payload.height,
                    upload_bytes: payload.upload_bytes,
                    rgba: payload.rgba.clone(),
                    atlas_uv: payload.atlas_uv.map(ui_image_uv_rect),
                },
            },
            ChromeCommandKind::Clip => UiSurfaceCommandKind::Clip,
        },
    }
}

fn ui_text_style(style: UiTextRunPaintStyle) -> UiSurfaceTextStyle {
    match (style.strong, style.emphasis) {
        (true, true) => UiSurfaceTextStyle::StrongEmphasis,
        (true, false) => UiSurfaceTextStyle::Strong,
        (false, true) => UiSurfaceTextStyle::Emphasis,
        (false, false) => UiSurfaceTextStyle::Regular,
    }
}

fn ui_rect(frame: &FrameRect) -> UiSurfaceRect {
    UiSurfaceRect::new(frame.x, frame.y, frame.width, frame.height)
}

fn ui_image_uv_rect(rect: ChromeImageUvRect) -> UiSurfaceImageUvRect {
    UiSurfaceImageUvRect {
        min: rect.min,
        max: rect.max,
    }
}

#[cfg(test)]
mod tests;
