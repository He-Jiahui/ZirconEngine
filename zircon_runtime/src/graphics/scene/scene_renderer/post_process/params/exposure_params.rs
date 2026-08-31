use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    DEFAULT_CAMERA_EXPOSURE_EV100, EXPOSURE_HISTOGRAM_BIN_COUNT, RenderExposureMode,
    RenderExposureSettings,
};
use crate::core::math::UVec2;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct ExposureParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport_and_mode: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) range_and_filter: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) speeds_and_compensation: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) manual_and_default: [f32; 4],
}

impl ExposureParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn new(
        viewport_size: UVec2,
        settings: RenderExposureSettings,
        delta_seconds: f32,
    ) -> Self {
        let (min_ev100, max_ev100) = settings.render_histogram_range();
        let (low_percent, high_percent) = settings.render_filter_range();
        let pixel_count = viewport_size.x.max(1) as f32 * viewport_size.y.max(1) as f32;
        Self {
            viewport_and_mode: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                exposure_mode_id(settings.mode),
                EXPOSURE_HISTOGRAM_BIN_COUNT,
            ],
            range_and_filter: [min_ev100, max_ev100, low_percent, high_percent],
            speeds_and_compensation: [
                settings.render_speed_brighten(),
                settings.render_speed_darken(),
                settings.compensation_ev,
                delta_seconds.max(0.0),
            ],
            manual_and_default: [
                settings.manual_ev100,
                DEFAULT_CAMERA_EXPOSURE_EV100,
                pixel_count,
                0.0,
            ],
        }
    }
}

pub(in crate::graphics::scene::scene_renderer) fn default_exposure_buffer_words() -> [f32; 4] {
    [
        1.0,
        DEFAULT_CAMERA_EXPOSURE_EV100,
        DEFAULT_CAMERA_EXPOSURE_EV100,
        0.0,
    ]
}

fn exposure_mode_id(mode: RenderExposureMode) -> u32 {
    match mode {
        RenderExposureMode::Manual => 0,
        RenderExposureMode::Histogram => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposure_params_preserve_the_authoritative_frame_delta() {
        let params = ExposureParams::new(
            UVec2::new(640, 360),
            RenderExposureSettings::default(),
            0.125,
        );

        assert_eq!(params.speeds_and_compensation[3], 0.125);
    }
}
