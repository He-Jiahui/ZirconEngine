use super::super::data::FrameRect;
use super::{HostRgbaFrame, geometry::clipped_frame};

impl HostRgbaFrame {
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
}

#[inline]
fn write_pixel_channels(pixel: &mut [u8], color: [u8; 4]) {
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}

pub(in crate::ui::retained_host::host_contract) fn fill_pixel_span(
    span: &mut [u8],
    color: [u8; 4],
) {
    for pixel in span.chunks_exact_mut(4) {
        write_pixel_channels(pixel, color);
    }
}
