use super::super::command::{ChromeCommand, ChromeCommandKind, ChromeCommandLayer};
use crate::ui::retained_host::host_contract::paint_frame::{
    HostRecordedPaintCommand, HostRecordedPaintKind,
};

use super::image::chrome_image_payload_from_recorded_image;
use super::visibility::visible_frame;

pub(super) fn chrome_command_from_recorded(
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
    include_image_bytes: bool,
) -> Option<ChromeCommand> {
    if !visible_frame(&command.frame) {
        return None;
    }
    let layer = match &command.kind {
        HostRecordedPaintKind::Text { .. } => ChromeCommandLayer::Text,
        HostRecordedPaintKind::Image { .. } => ChromeCommandLayer::Viewport,
        HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. } => {
            if full_rebuild {
                ChromeCommandLayer::Static
            } else {
                ChromeCommandLayer::Dynamic
            }
        }
    };
    let kind = chrome_command_kind_from_recorded(command.kind, include_image_bytes);
    Some(ChromeCommand {
        layer,
        z_index: command.z_index,
        frame: command.frame,
        clip: command.clip_frame,
        kind,
    })
}

fn chrome_command_kind_from_recorded(
    kind: HostRecordedPaintKind,
    include_image_bytes: bool,
) -> ChromeCommandKind {
    match kind {
        HostRecordedPaintKind::Quad {
            color,
            corner_radius,
        } => ChromeCommandKind::Quad {
            color,
            corner_radius,
        },
        HostRecordedPaintKind::Border {
            color,
            width,
            corner_radius,
        } => ChromeCommandKind::Border {
            color,
            width,
            corner_radius,
        },
        HostRecordedPaintKind::Text {
            text,
            color,
            font_size,
            line_height,
            style,
        } => ChromeCommandKind::Text {
            text,
            color,
            size: font_size,
            line_height,
            style,
        },
        HostRecordedPaintKind::Image {
            resource_key,
            width,
            height,
            rgba,
            atlas,
        } => ChromeCommandKind::Image {
            payload: chrome_image_payload_from_recorded_image(
                resource_key,
                width,
                height,
                rgba,
                atlas,
                include_image_bytes,
            ),
        },
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::host_contract) fn chrome_command_from_recorded_for_test(
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
    include_image_bytes: bool,
) -> Option<ChromeCommand> {
    chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
}
