use super::super::super::command::ChromeCommandKind;
use crate::ui::retained_host::host_contract::paint_frame::HostRecordedPaintKind;

use super::super::image::chrome_image_payload_from_recorded_image;

pub(super) fn chrome_command_kind_from_recorded(
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
