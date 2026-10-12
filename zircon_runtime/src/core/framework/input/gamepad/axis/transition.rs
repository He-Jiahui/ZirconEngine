//! 帧内轴变化端点的拥有式记录，供动作层辨认零与非零转换。
//!
//! # 使用时机、约束与生命周期
//!
//! 常规采样合并起点与终点，不保留完整轨迹；帧清理影响管理器集合，不改变已经复制的快照。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::InputFrameSnapshot::gamepad_axis_transitions`]、[`crate::core::framework::input::GamepadAxisState`]。
use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 记录一根设备轴的变化，供动作求值识别跨零边沿；常规同帧采样合并起点和最终值。
/// 保存当前帧一根设备轴的变化端点，供动作求值识别归零与离零边沿。
///
/// # 记录语义与约束
///
/// 当前默认管理器的常规轴采样按设备和轴合并：保留该帧第一次变化的 previous_value，只把 value 更新为最新接受值，保持首次出现顺序。变化是处理后端点，不保存中间轨迹或原始事件时间。
/// 首次没有当前样本时以零为起点，接受零值但值不变时不生成常规变化。失焦可能改写已有记录终值；断开可追加到零的记录，所以不能保证任何来源的列表中每个控件只有一条。
/// 动作求值按绑定方向解释端点并判断最终零/非零关系；同帧完整轨迹中的每次跨零不一定各有动作边沿。面对公开快照的重复记录，当前求值索引对每个控件保留最后源项。
///
/// # 使用时机与生命周期
///
/// 在读取当前帧快照并求值动作时使用；begin_frame 清空管理器变化集合，同时保留持续轴值。拥有式快照保留复制时的记录，不受后续清理或事件改变。
/// 字段、Copy 与 Serde 不验证两个值的来源、先后或有限性，也不校验连接状态。结构没有设备借用、帧编号或时间戳；其它生产者须自行维持所需的端点契约。
///
/// # 参考
///
/// 参见 [`crate::core::framework::input::InputFrameSnapshot::gamepad_axis_transitions`]、[`crate::core::framework::input::GamepadAxisState`]、[`crate::core::framework::input::InputAxisBinding`]、[`crate::core::framework::input::InputManager::begin_frame()`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisTransition {
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
    /// 本条变化的起点值，供动作层与最终值一起判断归零或离零。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 常规默认管理器记录同帧该控件第一次变化之前的已接受值；无历史以零为起点，后续常规合并不更新它。断开追加记录使用断开前当前值，不能无条件视为全帧起点。
    /// 它是处理后幅值，不是变化量或原始事件。拥有式副本在帧结束后仍保存原值；公开构造不验证来源或有限性。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::value`]、[`crate::core::framework::input::GamepadAxisState`]、[`crate::core::framework::input::InputAxisBinding`]。
    pub previous_value: f32,
    /// 本条变化的最终端点，供动作层判断求值时的零与非零关系。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 常规默认管理器同帧后续采样只更新此终值；失焦或断开可写入归零终值。中间多次跨零不完整保存在此字段。
    /// 它是处理后幅值，配合 previous_value 和绑定方向使用；结构本身不检查与当前样本的一致性或有限性。旧快照副本不自动更新。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::previous_value`]、[`crate::core::framework::input::InputAxisBinding`]、[`crate::core::framework::input::InputFrameSnapshot::gamepad_axis_transitions`]。
    pub value: f32,
}
