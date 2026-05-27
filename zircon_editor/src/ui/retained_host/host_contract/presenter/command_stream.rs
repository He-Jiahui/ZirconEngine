use std::collections::BTreeSet;

use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::{FrameRect, HostWindowPresentationData};
use super::super::painter::{
    draw_rect_clipped, draw_rgba_image_clipped_with_resource_key, draw_rounded_border_clipped,
    draw_rounded_rect_clipped, draw_text_with_size_and_style, record_host_frame_commands,
    HostRecordedPaintCommand, HostRecordedPaintKind, HostRgbaFrame,
};

const FALLBACK_IMAGE_COLOR: [u8; 4] = [42, 58, 78, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ChromeCommandLayer {
    Static,
    Dynamic,
    Text,
    Viewport,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ChromeCommandKind {
    Quad {
        color: [u8; 4],
        corner_radius: f32,
    },
    Border {
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    },
    Text {
        text: String,
        color: [u8; 4],
        size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    },
    Image {
        payload: ChromeImagePayload,
    },
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeImagePayload {
    pub(super) resource_key: String,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) upload_bytes: u64,
    pub(super) rgba: Option<Vec<u8>>,
    pub(super) atlas_uv: Option<ChromeImageUvRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ChromeImageUvRect {
    pub(super) min: [f32; 2],
    pub(super) max: [f32; 2],
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeCommand {
    pub(super) layer: ChromeCommandLayer,
    pub(super) z_index: i32,
    pub(super) frame: FrameRect,
    pub(super) clip: Option<FrameRect>,
    pub(super) kind: ChromeCommandKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ChromeCommandStreamStats {
    pub(super) command_count: usize,
    pub(super) static_command_count: usize,
    pub(super) dynamic_command_count: usize,
    pub(super) text_command_count: usize,
    pub(super) image_command_count: usize,
    pub(super) clip_command_count: usize,
    pub(super) image_upload_bytes: u64,
    pub(super) draw_call_count: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ChromeCommandStream {
    surface_size: (u32, u32),
    damage: Option<FrameRect>,
    full_rebuild: bool,
    commands: Vec<ChromeCommand>,
}

impl ChromeCommandStream {
    pub(super) fn full_rebuild(surface_size: (u32, u32)) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: None,
            full_rebuild: true,
            commands: Vec::new(),
        }
    }

    pub(super) fn patch(surface_size: (u32, u32), damage: FrameRect) -> Self {
        Self {
            surface_size: clamp_surface_size(surface_size),
            damage: Some(damage),
            full_rebuild: false,
            commands: Vec::new(),
        }
    }

    pub(super) fn is_full_rebuild(&self) -> bool {
        self.full_rebuild
    }

    pub(super) fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    pub(super) fn damage(&self) -> Option<&FrameRect> {
        self.damage.as_ref()
    }

    pub(super) fn commands(&self) -> &[ChromeCommand] {
        &self.commands
    }

    pub(super) fn push_quad(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Quad {
                color,
                corner_radius,
            },
        );
    }

    pub(super) fn push_border(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    ) {
        self.push_command(
            layer,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }

    pub(super) fn push_text(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        size: f32,
    ) {
        self.push_command(
            ChromeCommandLayer::Text,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Text {
                text: text.into(),
                color,
                size,
                line_height: size.max(1.0) * 1.2,
                style: UiTextRunPaintStyle::default(),
            },
        );
    }

    pub(super) fn push_image(
        &mut self,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        payload: ChromeImagePayload,
    ) {
        self.push_command(
            ChromeCommandLayer::Viewport,
            z_index,
            frame,
            clip,
            ChromeCommandKind::Image { payload },
        );
    }

    pub(super) fn push_clip(&mut self, layer: ChromeCommandLayer, z_index: i32, frame: FrameRect) {
        self.push_command(
            layer,
            z_index,
            frame.clone(),
            Some(frame),
            ChromeCommandKind::Clip,
        );
    }

    pub(super) fn stats(&self) -> ChromeCommandStreamStats {
        let mut uploaded_image_keys = BTreeSet::new();
        let mut stats = ChromeCommandStreamStats {
            command_count: self.commands.len(),
            ..ChromeCommandStreamStats::default()
        };
        for command in &self.commands {
            match command.layer {
                ChromeCommandLayer::Static => stats.static_command_count += 1,
                ChromeCommandLayer::Dynamic => stats.dynamic_command_count += 1,
                ChromeCommandLayer::Text => stats.text_command_count += 1,
                ChromeCommandLayer::Viewport => stats.dynamic_command_count += 1,
            }
            match &command.kind {
                ChromeCommandKind::Quad { .. }
                | ChromeCommandKind::Border { .. }
                | ChromeCommandKind::Text { .. } => stats.draw_call_count += 1,
                ChromeCommandKind::Image { payload } => {
                    stats.image_command_count += 1;
                    if payload.rgba.is_some()
                        && uploaded_image_keys.insert(payload.resource_key.as_str())
                    {
                        stats.image_upload_bytes = stats
                            .image_upload_bytes
                            .saturating_add(payload.upload_bytes);
                    }
                    stats.draw_call_count += 1;
                }
                ChromeCommandKind::Clip => stats.clip_command_count += 1,
            }
        }
        stats
    }

    fn push_recorded_command(
        &mut self,
        command: HostRecordedPaintCommand,
        include_image_bytes: bool,
    ) {
        let layer = match &command.kind {
            HostRecordedPaintKind::Text { .. } => ChromeCommandLayer::Text,
            HostRecordedPaintKind::Image { .. } => ChromeCommandLayer::Viewport,
            HostRecordedPaintKind::Quad { .. } | HostRecordedPaintKind::Border { .. } => {
                if self.full_rebuild {
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
        self.push_command(
            layer,
            command.z_index,
            command.frame,
            command.clip_frame,
            kind,
        );
    }

    fn push_command(
        &mut self,
        layer: ChromeCommandLayer,
        z_index: i32,
        frame: FrameRect,
        clip: Option<FrameRect>,
        kind: ChromeCommandKind,
    ) {
        if !visible_frame(&frame) {
            return;
        }
        self.commands.push(ChromeCommand {
            layer,
            z_index,
            frame,
            clip,
            kind,
        });
    }
}

pub(super) fn build_chrome_command_stream(
    presentation: &HostWindowPresentationData,
    surface_size: (u32, u32),
    damage: Option<&FrameRect>,
    include_image_bytes: bool,
) -> ChromeCommandStream {
    let surface_size = clamp_surface_size(surface_size);
    let (recorded_commands, clipped_damage) =
        record_host_frame_commands(surface_size.0, surface_size.1, presentation, damage);
    let mut stream = if let Some(damage) = clipped_damage.clone() {
        ChromeCommandStream::patch(surface_size, damage.clone())
    } else {
        ChromeCommandStream::full_rebuild(surface_size)
    };
    if let Some(damage) = clipped_damage {
        stream.push_clip(ChromeCommandLayer::Dynamic, 0, damage);
    }
    for command in recorded_commands {
        stream.push_recorded_command(command, include_image_bytes);
    }
    stream
}

pub(super) fn paint_chrome_command_stream_to_frame(
    width: u32,
    height: u32,
    stream: &ChromeCommandStream,
) -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    paint_chrome_command_stream_into_frame(&mut frame, stream);
    frame
}

pub(super) fn repaint_chrome_command_stream_region(
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

fn atlas_subimage_rgba(
    atlas_width: u32,
    atlas_height: u32,
    rgba: &[u8],
    atlas_uv: ChromeImageUvRect,
) -> Option<(u32, u32, Vec<u8>)> {
    let (x0, y0, x1, y1) = atlas_uv_pixel_rect(atlas_width, atlas_height, atlas_uv)?;
    let width = x1.checked_sub(x0)?;
    let height = y1.checked_sub(y0)?;
    if width == 0 || height == 0 || rgba.len() != atlas_width as usize * atlas_height as usize * 4 {
        return None;
    }
    let mut subimage = Vec::with_capacity(width as usize * height as usize * 4);
    let atlas_width = atlas_width as usize;
    let width = width as usize;
    for y in y0 as usize..y1 as usize {
        let start = ((y * atlas_width) + x0 as usize) * 4;
        let end = start + width * 4;
        subimage.extend_from_slice(&rgba[start..end]);
    }
    Some((width as u32, height, subimage))
}

fn atlas_uv_pixel_rect(
    atlas_width: u32,
    atlas_height: u32,
    atlas_uv: ChromeImageUvRect,
) -> Option<(u32, u32, u32, u32)> {
    if atlas_width == 0
        || atlas_height == 0
        || !atlas_uv.min[0].is_finite()
        || !atlas_uv.min[1].is_finite()
        || !atlas_uv.max[0].is_finite()
        || !atlas_uv.max[1].is_finite()
        || atlas_uv.min[0] < 0.0
        || atlas_uv.min[1] < 0.0
        || atlas_uv.max[0] > 1.0
        || atlas_uv.max[1] > 1.0
        || atlas_uv.min[0] >= atlas_uv.max[0]
        || atlas_uv.min[1] >= atlas_uv.max[1]
    {
        return None;
    }
    let x0 = (atlas_uv.min[0] * atlas_width as f32).round() as u32;
    let y0 = (atlas_uv.min[1] * atlas_height as f32).round() as u32;
    let x1 = (atlas_uv.max[0] * atlas_width as f32).round() as u32;
    let y1 = (atlas_uv.max[1] * atlas_height as f32).round() as u32;
    (x0 < x1 && y0 < y1 && x1 <= atlas_width && y1 <= atlas_height).then_some((x0, y0, x1, y1))
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

fn visible_frame(frame: &FrameRect) -> bool {
    frame.x.is_finite()
        && frame.y.is_finite()
        && frame.width.is_finite()
        && frame.height.is_finite()
        && frame.width > 0.0
        && frame.height > 0.0
}

fn clamp_surface_size(size: (u32, u32)) -> (u32, u32) {
    (size.0.max(1), size.1.max(1))
}

#[cfg(test)]
mod atlas_tests;

#[cfg(test)]
mod tests;
