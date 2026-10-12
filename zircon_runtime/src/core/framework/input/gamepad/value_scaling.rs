//! 手柄整形共享的私有线性映射计算，供调用方在中段分支复用。
//!
//! # 使用时机、约束与生命周期
//!
//! 不负责输入验证、钳制或状态存储；分母、参数和结果约束由实际调用路径负责。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::GamepadAxisSettings::scaled_value()`]、[`crate::core::framework::input::GamepadButtonAxisSettings::scaled_value()`]。
/// 按源区间的相对位置计算目标区间值，为手柄轴和模拟按钮的中段整形共用线性算式。
///
/// # 参数
///
/// `value` 是待映射值；`old_start`、`old_end` 定义源区间端点，`new_start`、`new_end` 定义目标区间端点，五个幅值均按值使用。
/// 调用者若要求有限线性结果，应保证输入与端点有限、源区间跨度非零，并考虑浮点溢出；区间可以递减，但不自动交换端点。
///
/// # 返回
///
/// 返回保持相对位置的浮点映射结果。源值超出区间时外推，不钳制目标范围；相等源端点或非有限参数按浮点算式产生 NaN、无穷等结果，没有错误返回。
///
/// # 使用时机
///
/// 只在调用方已经完成对应死区、饱和和分支准入后用于中段；不是通用输入验证入口。
///
/// # 约束
///
/// 函数不验证分母或结果，不能据返回类型推断有限性。轴设置的外层条件会避开部分有限相等边界，不能把本算式的潜在零分母推广为所有该配置的实际调用路径。
///
/// # 所有权与生命周期
///
/// 纯值计算，没有设备、历史或其它借用，不改变缓存或派发工作。
///
/// # 参考
///
/// 参见 [`crate::core::framework::input::GamepadAxisSettings::scaled_value()`]、[`crate::core::framework::input::GamepadButtonAxisSettings::scaled_value()`]。
pub(super) fn linear_remapping(
    value: f32,
    old_start: f32,
    old_end: f32,
    new_start: f32,
    new_end: f32,
) -> f32 {
    ((value - old_start) / (old_end - old_start)) * (new_end - new_start) + new_start
}
