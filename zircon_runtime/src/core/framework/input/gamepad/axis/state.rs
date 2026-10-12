//! 当前已接受轴值的拥有式观察记录，供帧快照保留持续状态。
//!
//! # 使用时机、约束与生命周期
//!
//! 默认管理器的当前值跨帧保持，失焦或断开清理缓存；已经复制的观察值不随事件更新，也不能证明连接。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::InputFrameSnapshot::gamepad_axes`]、[`crate::core::framework::input::GamepadAxisTransition`]。
use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 帧快照中的当前轴样本；默认输入管理器在写入前完成死区映射和阈值过滤。
/// 保存快照读取时某台设备某根轴的最近已接受值，供动作层观察持续模拟状态。
///
/// # 状态与约束
///
/// 设备与轴组成控件键，value 是缓存的处理后幅值，不是原始采样或帧内变化量。当前默认管理器只在默认整形及变化筛选接受后替换它，也保存零值项；忽略的小变化不会使缓存更新。
/// 结构字段公开，派生 Copy、Serde 不验证来源、幅值范围或设备能力。默认管理器的值范围不能推广为所有公开构造值；含 NaN 时浮点 PartialEq 可不自反。
///
/// # 使用时机与生命周期
///
/// 默认管理器从有序缓存复制到帧快照，按设备和轴键序投影；begin_frame 保留当前值，失焦清空，设备断开移除对应设备的项。事件写入不要求设备先有连接记录。
/// 副本没有管理器或设备借用，不随之后事件自动更新；存在某项不能证明当前仍连接或样本仍最新。trait 的默认 frame_snapshot 不提供这些扩展样本。
///
/// # 参考
///
/// 参见 [`crate::core::framework::input::InputFrameSnapshot::gamepad_axes`]、[`crate::core::framework::input::GamepadAxisSettings::process_value()`]、[`crate::core::framework::input::GamepadAxisTransition`]、[`crate::core::framework::input::InputManager::frame_snapshot()`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisState {
    /// 标识观察值所属设备，与 axis 共同定位一根物理轴。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 读取样本或变化时与绑定、消费键使用同一设备标签；不携带连接证明或硬件资源。
    /// 这是拥有式副本，断开后的旧快照仍保留该标签。公开赋值或 Serde 不验证设备是否存在。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::axis`]、[`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadAxisInput`]。
    pub gamepad: GamepadId,
    /// 标识设备内的轴类别或扩展编号，使同一轴的样本与变化能够匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 必须与 gamepad 联合匹配；单独使用轴标签会混淆不同设备。标签不包括当前数值、方向绑定或时间。
    /// 按值保存在快照中，没有设备借用；字段公开，硬件能力由调用方或宿主确认。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::gamepad`]、[`crate::core::framework::input::GamepadAxis`]、[`crate::core::framework::input::InputAxisBinding`]。
    pub axis: GamepadAxis,
    /// 该控件最近一次被接受的处理后轴幅值。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 当前默认管理器用默认设置输出 -1 到 1 值，筛选后的小变化及非有限原始值保留旧值；无历史的零值也可成为缓存项。
    /// 跨 begin_frame 持续，失焦或断开影响后续读取；旧快照副本不更新。公开构造不强制上述来源、范围或有限性。
    ///
    /// # 参考
    ///
    /// 参见 [`crate::core::framework::input::GamepadAxisSettings::process_value()`]、[`crate::core::framework::input::InputFrameSnapshot::gamepad_axes`]、[`crate::core::framework::input::GamepadAxisTransition`]。
    pub value: f32,
}
