//! 设备与轴的复合消费键，限定本次动作求值跳过哪个物理输入。
//!
//! # 使用时机、约束与生命周期
//!
//! 键不保存消费状态、方向或幅值；帧间消费由调用方再次传入，持有键不持有设备。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::InputAxisBinding`]、[`crate::core::framework::input::InputActionManager::evaluate_actions_with_consumed_input()`]。
use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadAxis;

/// 用设备与轴的组合标识一次可消费的模拟输入，避免上层消费一根轴时屏蔽其他设备的同名轴。
/// 把设备编号和轴标签合为一根物理输入的身份键，供动作层明确消费某个设备的某根轴。
///
/// # 身份与约束
///
/// 键不包含采样幅值、方向筛选或时间戳。相等、全序和哈希比较两个字段，排序先设备再轴；消费它不会同时屏蔽其它设备的同名轴。
/// 构造、公开字段和 Serde 只保存标签，不验证连接、控件存在性或当前帧是否包含样本。方向转换是动作绑定的责任。
///
/// # 使用时机与生命周期
///
/// 在上层路由后把键放入本次动作求值的 consumed_axes 切片；求值器据此跳过匹配输入。键本身不记录消费，也不自动延续到下一帧。
/// 这是 Copy 的拥有式标识，没有设备或帧借用；生命周期由保存它的集合负责，断开后旧键仍可被比较。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`crate::core::framework::input::InputAxisBinding`]、[`crate::core::framework::input::InputActionManager::evaluate_actions_with_consumed_input()`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GamepadAxisInput {
    /// 指定消费键所属设备，与 axis 一起确定物理输入身份。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 使用与帧样本及绑定一致的运行时设备标签；不携带连接证明或硬件资源。
    /// 按值保存，可跨帧持有；需要在后续求值继续消费时必须由调用方再次传入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::axis`]、[`crate::core::framework::input::GamepadId`]。
    pub gamepad: GamepadId,
    /// 指定设备内被消费的轴标签，独立于动作绑定的方向和当前幅值。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与 gamepad 一起匹配当前样本及变化记录；只传轴类别无法区分多台设备。
    /// 拥有式标签支持命名轴及 Other 扩展轴；公开赋值不验证设备能力或建立消费状态。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::gamepad`]、[`crate::core::framework::input::GamepadAxis`]、[`crate::core::framework::input::InputAxisBinding`]。
    pub axis: GamepadAxis,
}

impl GamepadAxisInput {
    /// 构造用于本次动作求值匹配的设备轴消费键，支持常量求值。
    ///
    /// # 参数
    ///
    /// `gamepad` 是设备运行时标签；`axis` 是该设备内的轴类别或扩展编号。两个参数按值保存，不读取设备或快照。
    ///
    /// # 返回
    ///
    /// 返回精确保留两个参数的拥有式身份键，没有错误返回或输入验证。
    ///
    /// # 使用时机
    ///
    /// 在准备动作求值的已消费轴列表时调用；调用本方法不会登记消费或修改绑定。
    ///
    /// # 约束
    ///
    /// 调用者保证标签与目标输入一致；构造成功不能证明设备连接或轴存在。
    ///
    /// # 所有权与生命周期
    ///
    /// 结果没有资源或借用。消费只对调用方将结果传入的那次求值生效，跨帧保存的键不会自动生效。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::gamepad`]、[`Self::axis`]、[`crate::core::framework::input::InputActionManager::evaluate_actions_with_consumed_input()`]。
    pub const fn new(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self { gamepad, axis }
    }
}
