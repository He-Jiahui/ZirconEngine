//! 定义兼容查询所需的指针位置、持续按下按钮和纵向滚动量。
//!
//! # 职责与使用时机
//! InputManager::snapshot 返回这三个字段的拥有型值；trait 的帧快照默认适配再从它补出有限状态。本模块只声明查询载荷，不采样设备，也不维护按钮边沿。
//!
//! # 边界与生命周期
//! 公开构造、Default 和 serde 生成值，不读取或修改运行时。返回值可独立保存和修改；去重、排序、采样一致性及帧时效取决于生产者。
//!
//! # 参考
//! 参见 [InputSnapshot](super::InputSnapshot)、[InputButton](super::InputButton)、[InputManager::snapshot()](super::InputManager::snapshot)、[InputManager::frame_snapshot()](super::InputManager::frame_snapshot)。
use serde::{Deserialize, Serialize};

use super::InputButton;

/// 保存指针位置、按下按钮和旧式纵向滚动量的拥有型兼容快照。
///
/// # 使用时机与职责
/// 由基本状态查询返回，或由调用方直接构造、克隆和反序列化；不是管理器的实时引用。
///
/// # 约束
/// 派生 Default 产生零位置、空按钮 Vec 和零滚动。公开字段和派生 Deserialize 不强制按钮去重、顺序、坐标有限性或帧一致性，也不把缺失字段自动补成 Default 值。PartialEq 使用字段比较，浮点 NaN 不构成普通等值。
///
/// # 生命周期
/// 容器和字符串按值归此快照拥有；保留旧值不会阻止管理器更新，旧值也不自动刷新。
///
/// # 参考
/// 参见 [InputManager::snapshot()](super::InputManager::snapshot)、[InputManager::frame_snapshot()](super::InputManager::frame_snapshot)、[InputFrameSnapshot](super::InputFrameSnapshot)、[InputButton](super::InputButton)。
/// 兼容只需要光标、按住按钮和纵向滚动的简化视图；它不携带帧边沿及扩展设备事件。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct InputSnapshot {
    /// 查询时保留的二维指针位置。
    ///
    /// # 使用时机与职责
    /// 由快照生产者在查询时填入；调用方也可直接构造或修改此值。
    ///
    /// # 约束
    /// 默认值为 [0,0]；当前 DefaultInputManager直接保留 CursorMoved 的 x、y。该值不表明指针位于窗口内，也不自行统一坐标空间或检查有限性。
    ///
    /// # 生命周期
    /// 值归快照拥有，不借用管理器；后续提交或切帧不更新已返回值。
    ///
    /// # 参考
    /// 参见 [InputEvent::CursorMoved](super::InputEvent::CursorMoved)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    pub cursor_position: [f32; 2],
    /// 查询时持续按下按钮的拥有型列表。
    ///
    /// # 使用时机与职责
    /// 由快照生产者在查询时填入；调用方也可直接构造或修改此值。
    ///
    /// # 约束
    /// 派生默认值为空；不含本帧边沿。DefaultInputManager按有序集合克隆并去重，但公开 Vec 和反序列化允许其他顺序或重复身份，消费方不能把生产者习惯当作类型验证。
    ///
    /// # 生命周期
    /// 值归快照拥有，不借用管理器；后续提交或切帧不更新已返回值。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[ButtonInputState](super::ButtonInputState)。
    pub pressed_buttons: Vec<InputButton>,
    /// 当前状态保留的兼容纵向滚动标量。
    ///
    /// # 使用时机与职责
    /// 由快照生产者在查询时填入；调用方也可直接构造或修改此值。
    ///
    /// # 约束
    /// 默认值为零；DefaultInputManager对 Line 取纵向值，对 Pixel 按固定比例换算，再在切帧清零。它不保留横向、逐条顺序或原始单位，不宜代替带单位事件。
    ///
    /// # 生命周期
    /// 值归快照拥有，不借用管理器；后续提交或切帧不更新已返回值。
    ///
    /// # 参考
    /// 参见 [MouseWheelEvent](super::MouseWheelEvent)、[InputFrameSnapshot](super::InputFrameSnapshot)。
    pub wheel_accumulator: f32,
}
