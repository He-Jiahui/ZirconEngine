use crate::core::math::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 轨道目标属于持续状态；视口平移必须同时更新相机位置和 target，后续旋转/缩放才围绕同一焦点。
pub struct OrbitCameraState {
    pub enabled: bool,
    pub target: Vec3,
}

impl Default for OrbitCameraState {
    fn default() -> Self {
        Self {
            enabled: true,
            target: Vec3::ZERO,
        }
    }
}
