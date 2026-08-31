use crate::ui::retained_host::host_contract::paint_color::{
    blend_premultiplied_linear_srgb_pixel, srgb_byte_to_linear,
};

const CHART_RASTER_SAMPLES_PER_AXIS: u32 = 4;

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

    pub(super) fn sample_pixel(
        &mut self,
        x: u32,
        y: u32,
        sample: impl Fn(f32, f32) -> Option<[u8; 4]>,
    ) {
        let mut premultiplied_linear = [0.0_f32; 3];
        let mut alpha_sum = 0.0_f32;
        for sample_y in 0..CHART_RASTER_SAMPLES_PER_AXIS {
            for sample_x in 0..CHART_RASTER_SAMPLES_PER_AXIS {
                let px = x as f32 + (sample_x as f32 + 0.5) / CHART_RASTER_SAMPLES_PER_AXIS as f32;
                let py = y as f32 + (sample_y as f32 + 0.5) / CHART_RASTER_SAMPLES_PER_AXIS as f32;
                let Some(color) = sample(px, py) else {
                    continue;
                };
                let alpha = f32::from(color[3]) / 255.0;
                alpha_sum += alpha;
                for channel in 0..3 {
                    premultiplied_linear[channel] += srgb_byte_to_linear(color[channel]) * alpha;
                }
            }
        }

        let sample_count = (CHART_RASTER_SAMPLES_PER_AXIS * CHART_RASTER_SAMPLES_PER_AXIS) as f32;
        let source_alpha = alpha_sum / sample_count;
        if source_alpha <= 0.0 {
            return;
        }
        premultiplied_linear
            .iter_mut()
            .for_each(|channel| *channel /= sample_count);

        let offset = ((y as usize * self.width as usize) + x as usize) * 4;
        blend_premultiplied_linear_srgb_pixel(
            &mut self.rgba[offset..offset + 4],
            premultiplied_linear,
            source_alpha,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::ChartRaster;

    #[test]
    fn half_coverage_white_over_black_resolves_in_linear_light() {
        let mut raster = ChartRaster {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        };

        raster.sample_pixel(0, 0, |x, _| (x < 0.5).then_some([255, 255, 255, 255]));

        assert!((187..=189).contains(&raster.rgba[0]));
        assert_eq!(raster.rgba[0], raster.rgba[1]);
        assert_eq!(raster.rgba[1], raster.rgba[2]);
        assert_eq!(raster.rgba[3], 255);
    }
}
