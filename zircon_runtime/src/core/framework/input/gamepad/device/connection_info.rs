//! 手柄连接状态观察及可选识别信息。
//!
//! # 职责、使用时机与生命周期
//! 通知拥有名称与标量；默认输入管理器应用通知，保存旧通知不会更新设备状态或保活连接。
//!
//! # 参考
//! [`GamepadConnectionInfo`]、[`crate::core::framework::input::InputEvent`]。
use std::num::NonZeroU64;

use serde::{Deserialize, Serialize};

use super::GamepadId;

/// 设备连接生命周期消息；断开时输入管理器释放该设备的按钮和轴，不将它误作窗口失焦。
///
/// 拥有一条手柄连接状态通知及其可选识别信息，供宿主把设备枚举或断连观察提交给输入管理器。
///
/// # 使用时机
///
/// 通过 InputEvent::GamepadConnection 发送；当前原生宿主在第一次轮询公布已连接设备，随后对连接与断连变化发出通知。
///
/// # 约束
///
/// connected 是这条消息报告的状态，不是读取时的实时查询。默认输入管理器为 gamepad 保留当前非零 generation 与开闭状态；旧代通知被拒绝，高代通知先退役旧输入，断连释放该编号按钮并移除模拟状态。其他设备及窗口焦点不因这条消息失去连接。名称和硬件 ID 不保存在默认帧快照的连接集合中。
///
/// # 所有权与生命周期
///
/// 通知拥有 Option<String>；Clone 为名称创建拥有型副本。它可随事件队列或原始记录存留，不能保证读取时设备仍在线，也不保活设备。公开字段及 serde 还原都不验证设备身份。
///
/// # 参考
///
/// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::InputEvent::GamepadConnection`]、[`crate::core::framework::input::InputManager`]、[`crate::core::framework::input::InputFrameSnapshot::connected_gamepads`]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadConnectionInfo {
    /// 这条通知描述的设备编号，用于配对同一宿主的按钮、轴和震动目标。
    ///
    /// # 约束
    ///
    /// 默认输入管理器按完整编号更新连接集合并清理断连设备状态；数值本身不附带连接检查。
    ///
    /// # 所有权与生命周期
    ///
    /// 按值拥有可复制编号；长期保存不延长设备连接。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::InputEvent::GamepadConnection`]
    pub gamepad: GamepadId,
    /// Producer-captured connection incarnation. Closed generations cannot reopen.
    /// Missing/zero generations are rejected by deserialization; routing IDs remain unchanged.
    pub generation: NonZeroU64,
    /// 报告本次通知是连接（true）还是断开（false）。
    ///
    /// # 使用时机
    ///
    /// 由宿主枚举或连接事件填写；默认输入管理器据此插入或移除 gamepad。
    ///
    /// # 约束
    ///
    /// 这是生产端捕获的一次观察。默认管理器仅接受已观察且仍打开的同代按钮/轴样本；断连留下关闭代际，后续同代连接也不能重新打开。只有更高代连接可建立新输入周期。
    ///
    /// # 所有权与生命周期
    ///
    /// 布尔值按值随通知保存；读取旧通知不会查询或订阅设备的新状态。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::InputEvent::GamepadConnection`]、[`crate::core::framework::input::InputFrameSnapshot::connected_gamepads`]
    pub connected: bool,
    /// 可选的、由通知拥有的设备显示名称。
    ///
    /// # 约束
    ///
    /// None 表示此次未提供名称；公开值允许 Some(空字符串)。V2 ABI 在代际头之后借用名称，空名称转为 None，非法 UTF-8 拒绝，合法非空名称复制成 String。默认连接快照不存储该名称。
    ///
    /// # 所有权与生命周期
    ///
    /// 字符串随通知、克隆事件或记录存活，不借用宿主发送时的临时名称字节。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadConnectionInfo`]、[`crate::core::framework::input::InputEvent::GamepadConnection`]
    pub name: Option<String>,
    /// 本次观察可用的可选厂商识别号，单位为原始 16 位硬件编号。
    ///
    /// # 约束
    ///
    /// 此共享类型允许任意 u16，包括 Some(0)。V2 桥接保留原有设备标量约定，以 0 表示缺失，接收端还把超出 u16 的值转为 None；不要据它单独识别一台设备。
    ///
    /// # 所有权与生命周期
    ///
    /// 拥有的元数据值，不参与设备保活或连接集合键。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadConnectionInfo`]
    pub vendor_id: Option<u16>,
    /// 本次观察可用的可选产品识别号，与 vendor_id 一同描述设备型号信息。
    ///
    /// # 约束
    ///
    /// 共享类型保留任意 u16；V2 桥接保留原有设备标量约定，将 0 或超出 u16 的值读为 None。它不替代 gamepad，也没有设备世代或跨会话唯一性约束。
    ///
    /// # 所有权与生命周期
    ///
    /// 元数据随通知按值存留，不证明读取时仍有此设备。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadId`]、[`crate::core::framework::input::GamepadConnectionInfo`]
    pub product_id: Option<u16>,
}
