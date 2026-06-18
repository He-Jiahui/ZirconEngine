use super::{
    ChromeCommand, ChromeCommandKind, ChromeCommandLayer, ChromeImagePayload, ChromeImageUvRect,
};
use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};
use crate::ui::retained_host::host_contract::paint_frame::{
    HostRecordedPaintCommand, HostRecordedPaintKind,
};
use crate::ui::retained_host::host_contract::paint_recording::record_host_frame_commands;

pub(super) struct ChromeCommandExtraction {
    pub(super) commands: Vec<ChromeCommand>,
    pub(super) clipped_damage: Option<FrameRect>,
}

pub(super) fn extract_chrome_commands(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandExtraction {
    let (recorded_commands, clipped_damage) =
        record_host_frame_commands(surface_size.0, surface_size.1, presentation, damage);
    let full_rebuild = clipped_damage.is_none();
    let commands = recorded_commands
        .into_iter()
        .filter_map(|command| {
            chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
        })
        .collect();
    ChromeCommandExtraction {
        commands,
        clipped_damage,
    }
}

fn chrome_command_from_recorded(
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
    let kind = match command.kind {
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
        } => {
            let payload = if let Some(atlas) = atlas {
                let atlas_rgba = include_image_bytes.then_some(atlas.rgba).flatten();
                let upload_bytes = atlas_rgba
                    .as_ref()
                    .map(|rgba| rgba.len() as u64)
                    .unwrap_or_else(|| u64::from(atlas.width) * u64::from(atlas.height) * 4);
                ChromeImagePayload {
                    resource_key: atlas.resource_key,
                    width: atlas.width,
                    height: atlas.height,
                    upload_bytes,
                    rgba: atlas_rgba,
                    atlas_uv: Some(ChromeImageUvRect {
                        min: atlas.uv.min,
                        max: atlas.uv.max,
                    }),
                }
            } else {
                let upload_bytes = rgba
                    .as_ref()
                    .map(|rgba| rgba.len() as u64)
                    .unwrap_or_else(|| u64::from(width) * u64::from(height) * 4);
                ChromeImagePayload {
                    resource_key,
                    width,
                    height,
                    upload_bytes,
                    rgba: include_image_bytes.then_some(rgba).flatten(),
                    atlas_uv: None,
                }
            };
            ChromeCommandKind::Image { payload }
        }
    };
    Some(ChromeCommand {
        layer,
        z_index: command.z_index,
        frame: command.frame,
        clip: command.clip_frame,
        kind,
    })
}

#[cfg(test)]
pub(super) fn chrome_command_from_recorded_for_test(
    command: HostRecordedPaintCommand,
    full_rebuild: bool,
    include_image_bytes: bool,
) -> Option<ChromeCommand> {
    chrome_command_from_recorded(command, full_rebuild, include_image_bytes)
}

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}
