use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

use super::super::data::FrameRect;
use super::geometry::intersect;
use super::sprite_atlas::HostPaintAtlasImage;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) enum HostRecordedPaintKind {
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
        font_size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    },
    Image {
        resource_key: String,
        width: u32,
        height: u32,
        rgba: Option<Vec<u8>>,
        atlas: Option<HostPaintAtlasImage>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::ui::retained_host::host_contract) struct HostRecordedPaintCommand {
    pub frame: FrameRect,
    pub clip_frame: Option<FrameRect>,
    pub z_index: i32,
    pub kind: HostRecordedPaintKind,
}

#[derive(Clone, Debug, Default)]
struct HostPaintRecording {
    commands: Vec<HostRecordedPaintCommand>,
    next_z_index: i32,
    record_only: bool,
}

pub(in crate::ui::retained_host::host_contract) struct HostRgbaFrame {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    paint_clip: Option<FrameRect>,
    recording: Option<HostPaintRecording>,
}

impl HostRgbaFrame {
    pub(in crate::ui::retained_host::host_contract) fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
            paint_clip: None,
            recording: None,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn filled(
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Self {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        fill_pixel_span(&mut bytes, color);
        Self {
            width,
            height,
            bytes,
            paint_clip: None,
            recording: None,
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn recording_only(
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
            paint_clip: None,
            recording: Some(HostPaintRecording {
                record_only: true,
                ..HostPaintRecording::default()
            }),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn replace_paint_clip(
        &mut self,
        paint_clip: Option<FrameRect>,
    ) -> Option<FrameRect> {
        std::mem::replace(&mut self.paint_clip, paint_clip)
    }

    pub(in crate::ui::retained_host::host_contract) fn paint_clip(&self) -> Option<&FrameRect> {
        self.paint_clip.as_ref()
    }

    pub(in crate::ui::retained_host::host_contract) fn is_recording(&self) -> bool {
        self.recording.is_some()
    }

    pub(in crate::ui::retained_host::host_contract) fn record_only(&self) -> bool {
        self.recording
            .as_ref()
            .is_some_and(|recording| recording.record_only)
    }

    pub(in crate::ui::retained_host::host_contract) fn record_quad(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        color: [u8; 4],
        corner_radius: f32,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Quad {
                color,
                corner_radius,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn record_border(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        color: [u8; 4],
        width: f32,
        corner_radius: f32,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Border {
                color,
                width,
                corner_radius,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn record_text(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        text: impl Into<String>,
        color: [u8; 4],
        font_size: f32,
        line_height: f32,
        style: UiTextRunPaintStyle,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Text {
                text: text.into(),
                color,
                font_size,
                line_height,
                style,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn record_image(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        resource_key: impl Into<String>,
        width: u32,
        height: u32,
        rgba: Option<Vec<u8>>,
        atlas: Option<HostPaintAtlasImage>,
    ) {
        self.record_command(
            frame,
            clip_frame,
            HostRecordedPaintKind::Image {
                resource_key: resource_key.into(),
                width,
                height,
                rgba,
                atlas,
            },
        );
    }

    pub(in crate::ui::retained_host::host_contract) fn into_recorded_commands(
        self,
    ) -> Vec<HostRecordedPaintCommand> {
        self.recording
            .map(|recording| recording.commands)
            .unwrap_or_default()
    }

    pub(in crate::ui::retained_host::host_contract) fn fill_rect(
        &mut self,
        rect: &FrameRect,
        color: [u8; 4],
    ) {
        let clip_frame = self.paint_clip.clone();
        let Some(rect) = clipped_frame(rect, clip_frame.as_ref()) else {
            return;
        };
        if self.is_recording() {
            self.record_quad(rect.clone(), clip_frame, color, 0.0);
            if self.record_only() {
                return;
            }
        }
        let Some((x0, y0, x1, y1)) = self.pixel_rect(&rect) else {
            return;
        };
        for y in y0..y1 {
            let row_start = ((y as usize * self.width as usize) + x0 as usize) * 4;
            let row_end = ((y as usize * self.width as usize) + x1 as usize) * 4;
            fill_pixel_span(&mut self.bytes[row_start..row_end], color);
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn width(&self) -> u32 {
        self.width
    }

    pub(in crate::ui::retained_host::host_contract) fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::ui::retained_host::host_contract) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub(in crate::ui::retained_host::host_contract) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    fn pixel_rect(&self, rect: &FrameRect) -> Option<(u32, u32, u32, u32)> {
        if self.width == 0
            || self.height == 0
            || !rect.x.is_finite()
            || !rect.y.is_finite()
            || !rect.width.is_finite()
            || !rect.height.is_finite()
            || rect.width <= 0.0
            || rect.height <= 0.0
        {
            return None;
        }
        let x0 = rect.x.floor().max(0.0).min(self.width as f32) as u32;
        let y0 = rect.y.floor().max(0.0).min(self.height as f32) as u32;
        let x1 = (rect.x + rect.width).ceil().max(0.0).min(self.width as f32) as u32;
        let y1 = (rect.y + rect.height)
            .ceil()
            .max(0.0)
            .min(self.height as f32) as u32;
        (x0 < x1 && y0 < y1).then_some((x0, y0, x1, y1))
    }

    fn record_command(
        &mut self,
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        kind: HostRecordedPaintKind,
    ) {
        let Some(recording) = self.recording.as_mut() else {
            return;
        };
        if !visible_frame(&frame) {
            return;
        }
        let z_index = recording.next_z_index;
        recording.next_z_index = recording.next_z_index.saturating_add(1);
        recording.commands.push(HostRecordedPaintCommand {
            frame,
            clip_frame,
            z_index,
            kind,
        });
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

fn clipped_frame(frame: &FrameRect, clip: Option<&FrameRect>) -> Option<FrameRect> {
    match clip {
        Some(clip) => intersect(frame, clip),
        None if visible_frame(frame) => Some(frame.clone()),
        None => None,
    }
}

#[inline]
fn write_pixel_channels(pixel: &mut [u8], color: [u8; 4]) {
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}

fn fill_pixel_span(span: &mut [u8], color: [u8; 4]) {
    for pixel in span.chunks_exact_mut(4) {
        write_pixel_channels(pixel, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_rect_replaces_contiguous_row_span() {
        let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
        frame.fill_rect(
            &FrameRect {
                x: 1.0,
                y: 0.0,
                width: 2.0,
                height: 1.0,
            },
            [10, 20, 30, 255],
        );

        assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
    }

    #[test]
    fn recording_only_collects_quad_without_allocating_pixels() {
        let mut frame = HostRgbaFrame::recording_only(16, 12);

        frame.fill_rect(
            &FrameRect {
                x: 1.0,
                y: 2.0,
                width: 3.0,
                height: 4.0,
            },
            [10, 20, 30, 255],
        );

        let commands = frame.into_recorded_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].z_index, 0);
        assert!(matches!(
            commands[0].kind,
            HostRecordedPaintKind::Quad {
                color: [10, 20, 30, 255],
                corner_radius: 0.0,
            }
        ));
    }

    #[test]
    fn fill_rect_respects_active_paint_clip() {
        let mut frame = HostRgbaFrame::filled(4, 2, [0, 0, 0, 255]);
        frame.replace_paint_clip(Some(FrameRect {
            x: 1.0,
            y: 0.0,
            width: 2.0,
            height: 1.0,
        }));

        frame.fill_rect(
            &FrameRect {
                x: 0.0,
                y: 0.0,
                width: 4.0,
                height: 2.0,
            },
            [10, 20, 30, 255],
        );

        assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[4..8], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[8..12], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[16..20], &[0, 0, 0, 255]);
    }
}
