use crate::core::math::Real;

const MIN_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 1.0;
const MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 300.0;
const DEFAULT_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 50.0;
const MIN_DEPTH_OF_FIELD_FOCUS_RANGE: Real = 0.001;
const DEFAULT_DEPTH_OF_FIELD_FOCUS_RANGE: Real = 3.0;
const MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 3;
const MAX_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 12;
const DEFAULT_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderDepthOfFieldSettings {
    pub focus_distance: Real,
    pub focus_range: Real,
    pub aperture: Real,
    pub focal_length_mm: Real,
    pub max_blur_radius: Real,
    pub bokeh_blade_count: u32,
    pub bokeh_rotation_radians: Real,
}

impl Default for RenderDepthOfFieldSettings {
    fn default() -> Self {
        Self {
            focus_distance: 10.0,
            focus_range: DEFAULT_DEPTH_OF_FIELD_FOCUS_RANGE,
            aperture: 0.0,
            focal_length_mm: DEFAULT_DEPTH_OF_FIELD_FOCAL_LENGTH_MM,
            max_blur_radius: 0.0,
            bokeh_blade_count: DEFAULT_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT,
            bokeh_rotation_radians: 0.0,
        }
    }
}

impl RenderDepthOfFieldSettings {
    pub fn is_enabled(self) -> bool {
        self.aperture > 0.0 || self.max_blur_radius > 0.0
    }

    pub fn render_focus_distance(self) -> Real {
        self.focus_distance.max(0.0)
    }

    pub fn render_focus_range(self) -> Real {
        self.focus_range.max(MIN_DEPTH_OF_FIELD_FOCUS_RANGE)
    }

    pub fn render_aperture(self) -> Real {
        self.aperture.max(0.0)
    }

    pub fn render_focal_length_mm(self) -> Real {
        self.focal_length_mm.clamp(
            MIN_DEPTH_OF_FIELD_FOCAL_LENGTH_MM,
            MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM,
        )
    }

    pub fn render_max_blur_radius(self) -> Real {
        self.max_blur_radius.max(0.0)
    }

    pub fn render_bokeh_blade_count(self) -> u32 {
        self.bokeh_blade_count.clamp(
            MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT,
            MAX_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RenderDepthOfFieldSettings, MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM,
        MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT, MIN_DEPTH_OF_FIELD_FOCUS_RANGE,
    };

    #[test]
    fn depth_of_field_lens_settings_are_sanitized_for_renderer_upload() {
        let settings = RenderDepthOfFieldSettings {
            focus_distance: -2.0,
            focus_range: -1.0,
            aperture: -0.5,
            focal_length_mm: 400.0,
            max_blur_radius: -3.0,
            bokeh_blade_count: 2,
            ..Default::default()
        };

        assert_eq!(settings.render_focus_distance(), 0.0);
        assert_eq!(
            settings.render_focus_range(),
            MIN_DEPTH_OF_FIELD_FOCUS_RANGE
        );
        assert_eq!(settings.render_aperture(), 0.0);
        assert_eq!(
            settings.render_focal_length_mm(),
            MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM
        );
        assert_eq!(settings.render_max_blur_radius(), 0.0);
        assert_eq!(
            settings.render_bokeh_blade_count(),
            MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT
        );
    }
}
