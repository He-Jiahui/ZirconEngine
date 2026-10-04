use crate::core::math::Real;

/// 轨道灵敏度以指针像素差和滚轮增量为输入：orbit_sensitivity 产生弧度，pan_sensitivity 按相机距离换算世界位移。
#[derive(Clone, Copy, Debug, PartialEq)]
/// 轨道导航的角度、平移和缩放策略；控制器围绕 state.target 计算，最小距离防止穿过焦点。
pub struct OrbitCameraSettings {
    pub orbit_sensitivity: Real,
    pub pan_sensitivity: Real,
    pub zoom_fraction: Real,
    pub min_distance: Real,
    pub pitch_min: Real,
    pub pitch_max: Real,
}

impl Default for OrbitCameraSettings {
    fn default() -> Self {
        Self {
            orbit_sensitivity: 0.01,
            pan_sensitivity: 0.0015,
            zoom_fraction: 0.15,
            min_distance: 0.25,
            pitch_min: -1.4,
            pitch_max: 1.4,
        }
    }
}
