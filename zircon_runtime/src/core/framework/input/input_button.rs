//! 定义鼠标、键盘和手柄按钮在输入状态与动作绑定中的结构身份。
//!
//! # 职责与使用时机
//! 9 个分支区分固定鼠标按钮、扩展编号、物理键码、逻辑键名及设备/手柄按钮组合。InputManager 查询、按钮状态集合和动作绑定使用这些值比较身份；本模块不映射平台码或持有设备。
//!
//! # 边界与生命周期
//! String 键名由值拥有；复制或保存身份不会延长设备、窗口或服务的寿命。相等、排序、哈希和 serde 只作用于分支与载荷，不验证连接状态，也不把物理码与逻辑名合并。
//!
//! # 参考
//! 参见 [InputButton](super::InputButton)、[ButtonInputState](super::ButtonInputState)、[InputManager::button_pressed()](super::InputManager::button_pressed)。
use serde::{Deserialize, Serialize};

use super::{GamepadButton, GamepadId};

/// 按分支及完整载荷区分鼠标、物理键、逻辑键和指定手柄按钮的身份。
///
/// # 使用时机与职责
/// 作为按下集合、帧边沿、绑定匹配和脚本查询的键；本类型只表达身份，不执行设备采样或动作。
///
/// # 约束
/// 派生相等、排序和哈希使用分支及载荷。物理码与逻辑名不互为别名，逻辑 String 不折叠大小写、去空白或规范化；Gamepad 以设备 id 与按钮联合区分。MouseOther 的数值也不会映射成固定鼠标分支。公开构造与 serde 不验证码值、连接或有效设备对象；本类型没有 Default。
///
/// # 生命周期
/// Key 分支拥有字符串，其他身份载荷按值保存；身份值和其克隆不拥有任何活设备或窗口。
///
/// # 参考
/// 参见 [ButtonInputState](super::ButtonInputState)、[InputEvent::KeyboardInput](super::InputEvent::KeyboardInput)、[InputEvent::GamepadButton](super::InputEvent::GamepadButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)。
/// 动作绑定使用的按钮身份；键盘物理键码与逻辑键名是不同项，同一宿主事件可同时更新两者。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum InputButton {
    /// 鼠标左按钮的固定身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 不含设备/窗口身份，不自行赋予点击或选择动作。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseLeft,
    /// 鼠标右按钮的固定身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 与 MouseLeft、MouseOther 等分支分别匹配，不自行赋予菜单或相机动作。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseRight,
    /// 鼠标中按钮的固定身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 按钮通知与滚轮旋转事件分别表达，滚动不等于此按钮按下。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseMiddle,
    /// 鼠标后退按钮的固定身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 不自动触发导航；当前动态会话鼠标转换只接收左/右/中，此共享身份仍可由其他提交方使用。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseBack,
    /// 鼠标前进按钮的固定身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 不自动触发导航，也不保证每个宿主入口都接受此身份。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseForward,
    /// 携带提交方定义编号的其他鼠标按钮身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 载荷 .0 为 u16；不验证编号、设备范围或映射成其他固定分支。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    MouseOther(u16),
    /// 携带物理键码数字的按钮身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 载荷 .0 为 u32；码值约定来自提交方，不自动映射为逻辑键名。脚本可以通过数字或固定前缀显式选择此身份。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    KeyCode(u32),
    /// 携带拥有型逻辑键名的按钮身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 载荷 .0 为 String；按完整文字匹配，允许空字符串，不折叠大小写或去空白，区别于 KeyCode。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    Key(String),
    /// 指定手柄上的按钮完整身份。
    ///
    /// # 使用时机与职责
    /// 由提交方构造，供状态和绑定消费方按完整值匹配。
    ///
    /// # 约束
    /// 由 gamepad 与 button 联合区分；不验证设备连接、稳定世代或业务动作。
    ///
    /// # 生命周期
    /// 身份载荷归枚举值拥有；保存该值不拥有或固定设备/窗口生命周期。
    ///
    /// # 参考
    /// 参见 [InputButton](super::InputButton)、[InputManager::button_pressed()](super::InputManager::button_pressed)、[ButtonInputState](super::ButtonInputState)。
    Gamepad {
        /// 手柄设备身份值；与同分支 button 联合决定按钮身份。
        ///
        /// # 约束
        /// 本字段只保存身份，不验证连接，不拥有设备，也不把它转换成业务动作。
        ///
        /// # 生命周期
        /// 载荷按值归枚举拥有，与活设备寿命独立。
        ///
        /// # 参考
        /// 参见 [GamepadId](super::GamepadId)、[InputEvent::GamepadButton](super::InputEvent::GamepadButton)。
        gamepad: GamepadId,
        /// 指定手柄上的按钮值；相同按钮在不同 gamepad 上分别匹配。
        ///
        /// # 约束
        /// 本字段只保存身份，不验证连接，不拥有设备，也不把它转换成业务动作。
        ///
        /// # 生命周期
        /// 载荷按值归枚举拥有，与活设备寿命独立。
        ///
        /// # 参考
        /// 参见 [GamepadButton](super::GamepadButton)、[InputEvent::GamepadButton](super::InputEvent::GamepadButton)。
        button: GamepadButton,
    },
}
