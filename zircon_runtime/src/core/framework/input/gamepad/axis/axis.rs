//! 设备内轴标签，连接宿主解释和输入绑定的控件身份。
//!
//! # 使用时机、约束与生命周期
//!
//! 标签只拥有可复制值，硬件存在、方向及原始数值由宿主决定；实际定位须结合设备编号。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputEvent::GamepadAxis`]。
use serde::{Deserialize, Serialize};

/// 定义设备内模拟轴的共享身份，使设备事件、帧样本、动作绑定与消费索引使用同一标签。
///
/// # 身份与约束
///
/// 本枚举只标识控件类别；实际定位一根轴还须结合设备编号。命名变体不保证硬件存在、正方向或原始取值范围。
/// 派生相等、全序与哈希比较标签；全序按声明顺序并在 Other 中比较载荷，适用于有序缓存和绑定键，不是外部 ABI 数字代码。Serde 按派生枚举表示序列化，不校验设备能力。
///
/// # 使用时机与生命周期
///
/// 在解释宿主轴事件、构造绑定和匹配快照时使用。标签是可复制拥有式值，不包含采样、连接状态或设备借用；断开不会使已经保存的标签失效，但不能据它证明设备仍可用。
/// 当前宿主适配器映射命名轴，ABI解释器把其它轴代码收纳到 Other；扩展编号的来源及碰撞边界由适配器负责。
///
/// # 参考
///
/// 参见 [`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputEvent::GamepadAxis`]、[`crate::core::framework::input::InputAxisBinding`]、[`crate::core::framework::input::GamepadAxisState`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GamepadAxis {
    /// 标识左摇杆的横向轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::LeftStickX`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    LeftStickX,
    /// 标识左摇杆的纵向轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::LeftStickY`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    LeftStickY,
    /// 标识宿主映射的左侧 Z 轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::LeftZ`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    LeftZ,
    /// 标识右摇杆的横向轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::RightStickX`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    RightStickX,
    /// 标识右摇杆的纵向轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::RightStickY`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    RightStickY,
    /// 标识宿主映射的右侧 Z 轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::RightZ`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    RightZ,
    /// 标识方向键的横向模拟轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::DPadX`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    DPadX,
    /// 标识方向键的纵向模拟轴，供同类事件与绑定匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 与设备编号组合使用，正负方向和是否提供该轴由宿主决定；轴值整形不由此标签执行。
    /// 标签可复制并保存在帧或绑定中，没有设备借用；同名轴在不同设备上是不同输入。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::DPadY`]、[`crate::core::framework::input::GamepadAxisInput`]、[`crate::core::framework::input::InputAxisBinding`]。
    DPadY,
    /// 保存命名轴之外的设备内轴编号，使扩展轴仍能参与事件、绑定和消费匹配。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// `u16` 载荷是标签的一部分，不是采样或设备编号；本类型不验证它的来源、唯一性或硬件能力。
    /// 当前 ABI 解释器将未识别的 `u32` 轴代码用整数转换截为低 16 位，不同高位代码可能变成同一标签；宿主 Unknown 也进入此变体。
    /// 与设备编号组合后按载荷参与相等、排序及哈希。拥有副本不会保留设备连接，也不会获得硬件枚举或反向 ABI 转换能力。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::Other`]、[`crate::core::framework::input::GamepadAxisInput`]。
    Other(u16),
}
