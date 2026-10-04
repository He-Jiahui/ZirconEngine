use crate::core::math::Real;

const MIN_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 1.0;
const MAX_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 300.0;
const DEFAULT_DEPTH_OF_FIELD_FOCAL_LENGTH_MM: Real = 50.0;
const MIN_DEPTH_OF_FIELD_FOCUS_RANGE: Real = 0.001;
const DEFAULT_DEPTH_OF_FIELD_FOCUS_RANGE: Real = 3.0;
const MIN_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 3;
const MAX_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 12;
const DEFAULT_DEPTH_OF_FIELD_BOKEH_BLADE_COUNT: u32 = 6;

/// 景深在时间重建前消耗场景深度，并产生后续重建可读取的颜色结果。
/// 相机设置可以保留未启用的镜头参数；只有启用谓词成立时才安排对应通道。
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
#[path = "tests/depth_of_field_settings.rs"]
mod tests;
