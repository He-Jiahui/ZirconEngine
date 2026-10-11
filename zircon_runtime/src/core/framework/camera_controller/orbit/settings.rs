use crate::core::math::Real;

/// 轨道灵敏度以指针像素差和滚轮增量为输入：orbit_sensitivity 产生弧度，pan_sensitivity 按相机距离换算世界位移。
#[derive(Clone, Copy, Debug, PartialEq)]
/// 轨道导航的角度、平移和缩放策略；控制器围绕 state.target 计算，最小距离防止穿过焦点。
///
/// # 字段与约束
///
/// `orbit_sensitivity` 把指针差量换算弧度；`pan_sensitivity` 乘以相机到目标的距离，得到每单位指针差量的世界位移。
/// `zoom_fraction` 决定一步缩放的距离比例，默认控制器只使用输入增量的符号；平移和缩放系数使用时取非负。
/// `pitch_min`、`pitch_max` 是弧度边界，控制器使用前排序。
/// `min_distance` 是缩放步长计算的距离下限，使用时至少为控制器 epsilon；它不是对传入变换的几何校验，
/// 不会在构造或零缩放输入时自动把相机移到下限之外。
///
/// # 生命周期与参考
///
/// 设置按值保存在控制器中，不持有目标或相机；修改副本不影响已有实例。
/// 参见 [`crate::input::camera_controller::OrbitCameraController::settings_mut`] 与
/// [`crate::core::framework::camera_controller::OrbitCameraState`]。
pub struct OrbitCameraSettings {
    pub orbit_sensitivity: Real,
    pub pan_sensitivity: Real,
    pub zoom_fraction: Real,
    pub min_distance: Real,
    pub pitch_min: Real,
    pub pitch_max: Real,
}

impl Default for OrbitCameraSettings {
    /// 提供新轨道控制器使用的默认角度、平移和缩放策略。
    ///
    /// # 返回值
    ///
    /// 返回轨道系数 0.01、平移系数 0.0015、缩放比例 0.15、距离下限 0.25，俯仰边界为正负 1.4 弧度。
    /// 距离下限用于缩放步长计算，不保证构造后相机与目标已经满足该几何距离。
    ///
    /// # 使用时机与参考
    ///
    /// 结果按值交给控制器，不绑定目标、不校验相机，也不改变已有实例的设置。
    /// 参见 [`OrbitCameraSettings`] 与 [`crate::input::camera_controller::OrbitCameraController::new`]。
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
