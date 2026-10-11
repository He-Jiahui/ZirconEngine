//! 定义设备内按钮标签的共享枚举，供宿主事件归一化、按钮状态键和动作绑定使用。
//!
//! # 使用时机与生命周期
//!
//! 把按钮标签与设备编号组合后才能定位一个运行时控件；宿主协议码通过显式匹配转换，不使用本枚举的内存布局。
//! 标签是拥有式可复制值；持有或反序列化一个标签不连接设备，也不生成按钮事件、按住状态或帧边沿。
//!
//! # 参考
//!
//! 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::InputEvent::GamepadButton`]。
use serde::{Deserialize, Serialize};

/// 表示一个手柄设备内的按钮控件类别，供输入事件、按压集合和动作绑定共享相同的标签身份。
///
/// # 身份与排序
///
/// 命名变体区分面板按钮、肩部或扳机、菜单、摇杆按键和方向键；同名标签在不同设备上是不同控件，须与 [`crate::core::framework::input::GamepadId`] 组合。
/// [`Self::Other`] 的 `u16` 载荷保留未映射的标签值；它与命名变体不同，`Other(1)` 不会等同于 `South`。
/// 派生相等、哈希和全序使用变体身份及载荷；不同变体按声明顺序排序，`Other` 的同类载荷按数值排序。运行时以 `(设备, 按钮)` 为有序映射键，快照输出沿该顺序。
/// 本枚举没有指定跨语言表示；声明顺序不充当 ABI 编码。当前宿主桥接经版本化协议常量显式映射，未知 `u32` 码转换到 `Other` 时只保留低 16 位。
///
/// # 使用时机与生命周期
///
/// 在事件解释或构造动作按钮键时按值传递；`Clone`、`Copy` 和派生 Serde 产生独立标签值，不保留设备或管理器借用。
/// 序列化格式由所用 Serde 格式决定；读取标签不验证对应设备是否连接或按钮是否存在。没有该枚举的 `Default` 实现。
/// 按住、按下及释放边沿由输入管理器维护；标签自身没有时间戳、幅值或可变历史。
///
/// # 参考
///
/// 参见 [`Self::Other`]、[`crate::core::framework::input::InputButton::Gamepad`]、[`crate::core::framework::input::InputEvent::GamepadButton`] 与 [`crate::core::framework::input::GamepadButtonValueState`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GamepadButton {
    /// 标识面板南侧按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于确认或激活。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    South,
    /// 标识面板东侧按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于取消。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    East,
    /// 标识面板北侧按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前宿主转换保留此独立按钮标签；具体动作由绑定表指定。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    North,
    /// 标识面板西侧按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前宿主转换保留此独立按钮标签；具体动作由绑定表指定。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    West,
    /// 标识左侧第一肩部或扳机按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与第二左侧标签 `LeftTrigger2` 分开，宿主映射保留二者身份。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    LeftTrigger,
    /// 标识左侧第二肩部或扳机按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与第一左侧标签 `LeftTrigger` 分开；具体设备的模拟幅值由对应事件提供。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    LeftTrigger2,
    /// 标识右侧第一肩部或扳机按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与第二右侧标签 `RightTrigger2` 分开，宿主映射保留二者身份。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    RightTrigger,
    /// 标识右侧第二肩部或扳机按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与第一右侧标签 `RightTrigger` 分开；具体设备的模拟幅值由对应事件提供。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    RightTrigger2,
    /// 标识选择按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与 `Start` 和 `Mode` 分开；菜单或游戏用途由消费者的动作配置决定。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    Select,
    /// 标识开始按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 与 `Select` 和 `Mode` 分开；标签本身不会启动会话或切换菜单。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    Start,
    /// 标识模式按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 保留宿主模式按钮身份；应用模式或系统行为不由这个数据变体执行。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    Mode,
    /// 标识左摇杆按键，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 表示左摇杆的按键标签；摇杆连续轴另有 `GamepadAxis` 输入。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    LeftThumb,
    /// 标识右摇杆按键，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 表示右摇杆的按键标签；摇杆连续轴另有 `GamepadAxis` 输入。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    RightThumb,
    /// 标识方向键上方按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于向上导航。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    DPadUp,
    /// 标识方向键下方按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于向下导航。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    DPadDown,
    /// 标识方向键左方按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于向左导航。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    DPadLeft,
    /// 标识方向键右方按钮，用于把这个控件归入同一设备的按钮状态或动作绑定。
    /// 当前动态宿主导航路由把对应协议码用于向右导航。
    ///
    /// # 使用时机与生命周期
    ///
    /// 宿主事件归一化后将此标签与设备编号组合；运行时按住及边沿仍由已接受幅值和按钮状态判定。
    /// 这是可复制的拥有式标签，没有设备借用、按压历史或连接保证；排序与序列化遵循整个枚举的契约。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::InputButton::Gamepad`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
    DPadRight,
    /// 保留未归入命名变体的按钮标签，供事件转发和自定义按钮绑定继续区分这些控件。
    ///
    /// # 载荷与约束
    ///
    /// 唯一载荷是调用方拥有的 `u16` 标签值；所有位模式都可构造，本类型没有范围或硬件存在性验证。
    /// 当前动态协议转换把 C、Z、UNKNOWN 以及其它未映射 `u32` 码转换到该变体，载荷为 `button as u16`；超出 `u16` 的高位被丢弃，不同原始码可能得到相同标签。
    /// 载荷不是设备编号，也不是相同数字的命名按钮；身份仍须与设备编号组合。应用自行构造的 `Other` 值应与实际宿主转换保持同一解释。
    ///
    /// # 使用时机与生命周期
    ///
    /// 在未知或额外按钮事件及其绑定中使用；复制、排序和 Serde 只保存标签值，不维持连接活性或产生按钮状态。
    /// `Other` 排在所有命名变体之后，同类值再按载荷排序；拥有的标签可在快照之后继续保留，但不证明设备仍存在。
    ///
    /// # 参考
    ///
    /// 参见 [`GamepadButton`]、[`crate::core::framework::input::GamepadId`] 与 [`crate::core::framework::input::InputButton::Gamepad`]。
    Other(u16),
}
