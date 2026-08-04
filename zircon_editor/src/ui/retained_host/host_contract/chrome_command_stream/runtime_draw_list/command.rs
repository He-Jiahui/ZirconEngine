use zircon_runtime::rhi::{UiSurfaceCommand, UiSurfaceCommandKind, UiSurfaceImagePayload};

use super::super::{ChromeCommand, ChromeCommandKind};
use super::geometry::{ui_image_uv_rect, ui_rect};
use super::text_style::{ui_text_font_family, ui_text_font_weight, ui_text_style};

pub(super) fn ui_surface_command_from_chrome(
    command: &ChromeCommand,
    image_pixels_are_in_resource_table: bool,
) -> UiSurfaceCommand {
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
                font_family: Some(ui_text_font_family(*style)),
                font_weight: ui_text_font_weight(*style),
                font_size: *size,
                line_height: *line_height,
                style: ui_text_style(*style),
            },
            ChromeCommandKind::Image { payload } => UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: payload.resource_key.clone(),
                    resource_generation: payload.resource_generation,
                    width: payload.width,
                    height: payload.height,
                    upload_bytes: payload.upload_bytes,
                    rgba: (!image_pixels_are_in_resource_table)
                        .then(|| payload.rgba.clone())
                        .flatten(),
                    atlas_uv: payload.atlas_uv.map(ui_image_uv_rect),
                },
            },
            ChromeCommandKind::Clip => UiSurfaceCommandKind::Clip,
        },
    }
}

pub(super) fn ui_surface_command_from_owned_chrome(command: ChromeCommand) -> UiSurfaceCommand {
    UiSurfaceCommand {
        z_index: command.z_index,
        frame: ui_rect(&command.frame),
        clip: command.clip.as_ref().map(ui_rect),
        kind: match command.kind {
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            } => UiSurfaceCommandKind::Quad {
                color,
                corner_radius,
            },
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            } => UiSurfaceCommandKind::Border {
                color,
                width,
                corner_radius,
            },
            ChromeCommandKind::Text {
                text,
                color,
                size,
                line_height,
                style,
            } => UiSurfaceCommandKind::Text {
                text,
                color,
                font_family: Some(ui_text_font_family(style)),
                font_weight: ui_text_font_weight(style),
                font_size: size,
                line_height,
                style: ui_text_style(style),
            },
            ChromeCommandKind::Image { payload } => UiSurfaceCommandKind::Image {
                payload: UiSurfaceImagePayload {
                    resource_key: payload.resource_key,
                    resource_generation: payload.resource_generation,
                    width: payload.width,
                    height: payload.height,
                    upload_bytes: payload.upload_bytes,
                    rgba: payload.rgba,
                    atlas_uv: payload.atlas_uv.map(ui_image_uv_rect),
                },
            },
            ChromeCommandKind::Clip => UiSurfaceCommandKind::Clip,
        },
    }
}
