use crate::core::math::{Real, Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 视口导航传给轨道控制器的互斥动作；Focus 更新目标但不必立即改变相机变换。
pub enum OrbitCameraAction {
    None,
    Orbit { previous: Vec2, current: Vec2 },
    Pan { previous: Vec2, current: Vec2 },
    Zoom { delta: Real },
    Focus { target: Vec3 },
}

impl Default for OrbitCameraAction {
    fn default() -> Self {
        Self::None
    }
}
