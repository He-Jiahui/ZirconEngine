use super::super::data::FrameRect;

pub(in crate::ui::slint_host::host_contract) struct HostRgbaFrame {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    paint_clip: Option<FrameRect>,
}

impl HostRgbaFrame {
    pub(in crate::ui::slint_host::host_contract) fn empty(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            bytes: Vec::new(),
            paint_clip: None,
        }
    }

    pub(in crate::ui::slint_host::host_contract) fn filled(
        width: u32,
        height: u32,
        color: [u8; 4],
    ) -> Self {
        let mut bytes = vec![0; width as usize * height as usize * 4];
        for pixel in bytes.chunks_exact_mut(4) {
            pixel.copy_from_slice(&color);
        }
        Self {
            width,
            height,
            bytes,
            paint_clip: None,
        }
    }

    pub(in crate::ui::slint_host::host_contract) fn replace_paint_clip(
        &mut self,
        paint_clip: Option<FrameRect>,
    ) -> Option<FrameRect> {
        std::mem::replace(&mut self.paint_clip, paint_clip)
    }

    pub(in crate::ui::slint_host::host_contract) fn paint_clip(&self) -> Option<&FrameRect> {
        self.paint_clip.as_ref()
    }

    pub(in crate::ui::slint_host::host_contract) fn fill_rect(
        &mut self,
        rect: &FrameRect,
        color: [u8; 4],
    ) {
        let Some((x0, y0, x1, y1)) = self.pixel_rect(rect) else {
            return;
        };
        for y in y0..y1 {
            for x in x0..x1 {
                let offset = ((y as usize * self.width as usize) + x as usize) * 4;
                self.bytes[offset..offset + 4].copy_from_slice(&color);
            }
        }
    }

    pub(in crate::ui::slint_host::host_contract) fn width(&self) -> u32 {
        self.width
    }

    pub(in crate::ui::slint_host::host_contract) fn height(&self) -> u32 {
        self.height
    }

    pub(in crate::ui::slint_host::host_contract) fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(in crate::ui::slint_host::host_contract) fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub(in crate::ui::slint_host::host_contract) fn into_bytes(self) -> Vec<u8> {
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
}
