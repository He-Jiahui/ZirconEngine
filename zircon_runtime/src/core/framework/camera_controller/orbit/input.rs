use crate::core::math::{clamp_viewport_size, Real, UVec2, Vec2, Vec3};

use super::OrbitCameraAction;

#[derive(Clone, Copy, Debug, PartialEq)]
/// 编辑器视口和动态 Runtime API 共用的导航输入；当前控制器读取动作和焦点状态。
///
/// # 使用时机与约束
///
/// 调用方先选择动作，再设置视口尺寸和输入焦点，把返回值交给轨道控制器。
/// 默认动作为空、尺寸为一像素、焦点有效；字段公开，直接构造不等于经过尺寸校正。
/// `with_viewport_size` 会校正零尺寸，但当前轨道更新数学没有消费这个字段，不能承诺按视口归一化拖拽。
/// 焦点无效或控制器禁用时，动作不会更新目标或变换；输入值本身不改变窗口焦点。
///
/// # 生命周期与参考
///
/// 这是本次动作的值，不持有相机或目标；持续目标保存在控制器状态中。
/// 参见 [`Self::with_focus_active`]、[`Self::with_viewport_size`] 与
/// [`crate::input::camera_controller::OrbitCameraController::update`]。
pub struct OrbitCameraInput {
    pub action: OrbitCameraAction,
    // TODO: [CR-FRAMEWORK-CAMERA-0001] 确认视口尺寸是否应参与轨道拖拽归一化；两个调用端持续传入该值，但控制器目前未读取。
    pub viewport_size: UVec2,
    pub focus_active: bool,
}

impl OrbitCameraInput {
    /// 构造用于没有导航动作时传递的轨道输入。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回空动作、默认尺寸和有效焦点的值；它不重置控制器已有目标，也不执行相机更新。
    /// 若宿主当前失焦，调用方仍需显式设置焦点状态。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_focus_active`] 与 [`Self::with_viewport_size`]。
    pub fn none() -> Self {
        Self::default()
    }

    /// 构造围绕控制器持续目标旋转的拖拽输入。
    ///
    /// # 参数
    ///
    /// - `previous`：前一次指针采样点。
    /// - `current`：当前采样点；两点须采用同一坐标空间和单位。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回携带两个采样点的动作值，不立即旋转相机；默认焦点有效，视口尺寸尚为默认值。
    /// 调用方按实际焦点和视口补充输入，再交给持续持有目标的控制器。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_focus_active`]、[`Self::with_viewport_size`] 与
    /// [`crate::input::camera_controller::OrbitCameraController::update`]。
    pub fn orbit(previous: Vec2, current: Vec2) -> Self {
        Self {
            action: OrbitCameraAction::Orbit { previous, current },
            ..Self::default()
        }
    }

    /// 构造沿相机平面拖拽的平移动作。
    ///
    /// # 参数
    ///
    /// - `previous`：前一次指针采样点。
    /// - `current`：当前采样点；两点须采用同一坐标空间和单位。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回本次动作值，默认焦点有效；构造不会修改目标或相机。
    /// 控制器消费动作时同时平移相机和持续目标，使后续轨道操作沿用新焦点。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_focus_active`]、[`Self::with_viewport_size`] 与
    /// [`crate::input::camera_controller::OrbitCameraController::update`]。
    pub fn pan(previous: Vec2, current: Vec2) -> Self {
        Self {
            action: OrbitCameraAction::Pan { previous, current },
            ..Self::default()
        }
    }

    /// 构造沿相机当前前向移动的缩放动作。
    ///
    /// # 参数
    ///
    /// - `delta`：滚轮或缩放增量；值会完整保存，但默认轨道控制器只使用其符号，零值不请求位移。
    ///
    /// # 返回值与边界
    ///
    /// 返回默认焦点有效的动作值；构造不移动相机，也不改变视野角或正交投影尺寸。
    /// 步长和距离策略由控制器设置解释，增量的绝对大小不放大默认控制器的一步位移。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_focus_active`] 与 [`crate::core::framework::camera_controller::OrbitCameraSettings`]。
    pub fn zoom(delta: Real) -> Self {
        Self {
            action: OrbitCameraAction::Zoom { delta },
            ..Self::default()
        }
    }

    /// 构造替换轨道持续目标的聚焦动作。
    ///
    /// # 参数
    ///
    /// - `target`：与相机变换采用同一世界坐标空间的新目标点。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回默认焦点有效的动作值，不直接定位相机。
    /// 启用且具有输入焦点的控制器消费后只更新持续目标；后续旋转、平移和缩放使用这个目标。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_focus_active`] 与 [`crate::core::framework::camera_controller::OrbitCameraState`]。
    pub fn focus(target: Vec3) -> Self {
        Self {
            action: OrbitCameraAction::Focus { target },
            ..Self::default()
        }
    }

    /// 补充轨道输入携带的视口尺寸。
    ///
    /// # 参数
    ///
    /// - `viewport_size`：调用方报告的像素尺寸；先经 [`clamp_viewport_size`] 校正零分量。
    ///
    /// # 返回值与约束
    ///
    /// 返回修改尺寸后的输入，动作和焦点保持原值；不修改控制器或视口资源。
    /// 当前轨道控制器不读取该字段，尺寸校正本身不能证明拖拽已经按视口归一化。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::orbit`]、[`Self::pan`] 与 [`Self::with_focus_active`]。
    pub fn with_viewport_size(mut self, viewport_size: UVec2) -> Self {
        self.viewport_size = clamp_viewport_size(viewport_size);
        self
    }

    /// 补充本次动作的输入焦点状态。
    ///
    /// # 参数
    ///
    /// - `focus_active`：调用方报告的焦点状态；本方法不读取或取得窗口焦点。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回修改后的输入，动作和尺寸保持原值；交给控制器后，假值阻止所有动作，包括目标替换。
    /// 构造完成不会立即改变控制器状态。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::focus`] 与 [`crate::input::camera_controller::OrbitCameraController::update`]。
    pub fn with_focus_active(mut self, focus_active: bool) -> Self {
        self.focus_active = focus_active;
        self
    }
}

impl Default for OrbitCameraInput {
    /// 构造没有导航动作的轨道输入基值。
    ///
    /// # 返回值与使用时机
    ///
    /// 返回空动作、一像素尺寸和有效焦点；各动作构造方法在此基础上替换动作字段。
    /// 调用方仍须补充实际焦点与视口尺寸；默认尺寸是基值，不代表读取过视口资源。
    ///
    /// # 生命周期与参考
    ///
    /// 构造不修改控制器的持续目标或相机变换，结果只描述随后要投递的一次动作。
    /// 参见 [`Self::none`]、[`Self::with_focus_active`] 与 [`Self::with_viewport_size`]。
    fn default() -> Self {
        Self {
            action: OrbitCameraAction::None,
            viewport_size: UVec2::ONE,
            focus_active: true,
        }
    }
}
