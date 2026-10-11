//! 定义设备内一个按钮的拥有式幅值观察值，供帧快照与布尔按压状态并列提供给消费者。
//!
//! # 使用时机与生命周期
//!
//! 当前默认管理器从幅值缓存复制此值；缓存跨帧持续，只有接受新幅值、失焦或对应设备断开等输入处理改变它。
//! 这是读取时已有状态的副本，没有设备借用、连接保证、实时采样或自动更新能力。
//!
//! # 参考
//!
//! 参见 [`GamepadButtonValueState`]、[`crate::core::framework::input::InputFrameSnapshot::gamepad_button_values`] 与 [`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`]。
use serde::{Deserialize, Serialize};

use super::super::device::GamepadId;
use super::GamepadButton;

/// 保留模拟按钮的当前幅值，与布尔按住状态并列供帧快照观察；动作触发依据滞回后的按钮边沿。
///
/// 表示快照读取时某个设备按钮最近一次已接受的处理后幅值，使消费者能观察布尔按住状态之外的模拟强度。
///
/// # 状态与结果边界
///
/// 设备标签与按钮标签共同定位控件；当前默认管理器以该二元组为有序缓存键，按键顺序复制成帧快照列表。
/// 由当前默认管理器产生时，幅值经过默认零到一整形及变化阈值过滤；被忽略的原始采样不替换保存值。因此该值不一定等于最新原始设备采样，也不是帧内变化量。
/// 当前管理器保存幅值为零的已接受项，普通释放不会自动删除该项；是否按住或有按下/释放边沿须另查同一帧的按钮集合。
/// 当前按钮动作求值读取布尔按住及边沿，并把按钮动作强度作为数字量；本观察值不自行触发动作，也不表示宿主 UI导航已处理。
///
/// # 使用时机与生命周期
///
/// `begin_frame` 清除边沿但保留幅值缓存；失焦清空所有按钮幅值，对应设备断开移除其缓存并生成原按住项的释放边沿。先前返回的拥有式快照继续保留原值，后续事件不会更新它。
/// 默认管理器接收按钮事件时不要求设备已在连接集合中；持有此值不能证明设备当前连接、控件存在或该幅值仍是最新状态。
/// 三个字段均公开，`Clone`、`Copy` 与派生 Serde 只复制或构造值，不验证设备、幅值范围或与其它快照字段的一致性；含 `NaN` 时浮点 `PartialEq` 可不自反。
/// 本类型没有设备资源、引用、时间戳或配置代次，也没有 `Default` 实现。trait 默认帧快照的该列表为空，不能将某个具体管理器的投影策略扩展为所有实现的保证。
///
/// # 参考
///
/// 参见 [`crate::core::framework::input::InputFrameSnapshot::gamepad_button_values`]、[`crate::core::framework::input::InputFrameSnapshot::buttons`]、[`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`] 与 [`crate::core::framework::input::InputManager::frame_snapshot()`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonValueState {
    /// 观察值所属设备的运行时标签，与 `button` 共同组成当前幅值缓存的控件键。
    /// 标签是可复制整数标识，不携带连接状态、硬件资源或验证回执；同一按钮类别在不同设备上为不同项。
    ///
    /// # 使用时机与生命周期
    ///
    /// 读取快照时据此定位设备；当前默认管理器允许没有先前连接事件的按钮采样，所以该字段不证明设备在连接列表内。
    /// 设备断开后的新快照移除对应缓存，已有副本仍保留此标签；它不借用设备或管理器。
    ///
    /// # 参考
    ///
    /// 参见 [`crate::core::framework::input::GamepadId`]、[`Self::button`] 与 [`crate::core::framework::input::InputFrameSnapshot::connected_gamepads`]。
    pub gamepad: GamepadId,
    /// 设备内按钮的共享控件标签，与 `gamepad` 共同定位这一项幅值。
    /// 标签可为命名按钮或携带额外编号的 `Other`；它不包含幅值、当前按住布尔或帧内边沿。
    ///
    /// # 使用时机与生命周期
    ///
    /// 观察幅值后使用同一设备/按钮二元组查询相应按钮集合；单独比较标签无法区分不同设备。
    /// 这是拥有式副本，可按枚举全序参与快照键序；公开赋值或Serde 构造不验证硬件控件存在性。
    ///
    /// # 参考
    ///
    /// 参见 [`crate::core::framework::input::GamepadButton`]、[`Self::gamepad`] 与 [`crate::core::framework::input::InputButton::Gamepad`]。
    pub button: GamepadButton,
    /// 保存该控件最近一次被接受的处理后模拟幅值，而非原始采样、变化量或按钮按住布尔。
    /// 当前默认管理器使用默认整形与变化筛选，因此其投影值在零到一范围；筛选掉的小变化或非有限原始值不替换这个保存值。
    /// 普通释放仍可留下幅值为零的缓存项；中间幅值也可能因滞回而保持原按压状态，不能单靠此字段推断边沿。
    ///
    /// # 使用时机与生命周期
    ///
    /// 幅值缓存跨 `begin_frame` 保留，接受后续更新、失焦或设备断开影响后来读取的快照；已有副本不会自动更新。
    /// 结构公开构造和派生 Serde不强制上述管理器来源或范围，也不验证有限性；观察字段本身没有设备借用、时间戳或更新操作。
    ///
    /// # 参考
    ///
    /// 参见 [`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`]、[`crate::core::framework::input::GamepadButtonSettings::transition_for_value()`] 与 [`crate::core::framework::input::InputFrameSnapshot::buttons`]。
    pub value: f32,
}
