pub(in crate::ui::retained_host::host_contract::paint_template_nodes) struct ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) width: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) height: u32,
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) rgba: Vec<u8>,
}

impl ChartRaster {
    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn transparent(
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            width,
            height,
            rgba: vec![0; width as usize * height as usize * 4],
        }
    }

    pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn center(
        &self,
    ) -> (f32, f32) {
        (self.width as f32 * 0.5, self.height as f32 * 0.5)
    }

    pub(super) fn normalized_point(&self, point: (f32, f32)) -> (f32, f32) {
        (
            point.0.clamp(0.0, 1.0) * (self.width.saturating_sub(1)) as f32,
            point.1.clamp(0.0, 1.0) * (self.height.saturating_sub(1)) as f32,
        )
    }

    pub(super) fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
        let offset = ((y as usize * self.width as usize) + x as usize) * 4;
        self.rgba[offset..offset + 4].copy_from_slice(&color);
    }
}
