//! 双电机原始强度与显式归一化规则。
//!
//! # 职责、使用时机与生命周期
//! 构造保留原值；clamped 独立处理两通道。该文件不持有电机、效果句柄或播放状态。
//!
//! # 参考
//! [`GamepadRumbleIntensity`]、[`crate::core::framework::input::InputEvent`]。
use serde::{Deserialize, Serialize};

/// 运行时的双电机强度值；出站 ABI 转换前须调用 `clamped`，不能直接把未校验浮点值交给宿主。
///
/// 保存希望交给宿主的强、弱电机原始强度，并提供显式归一化操作。
///
/// # 使用时机
///
/// 为 Add 请求填写强度时使用 new、公开字段或命名常量；出站传给宿主前使用 clamped 得到归一化副本。
///
/// # 约束
///
/// 归一化域为每电机 [0,1]。该类型的公开字段、new 及 serde 还原都保存原值，允许越界、NaN 和正负无穷；clamped 才把有限值截到区间、把全部非有限值改为零。当前声明没有 Default 实现，常量也不表示硬件支持或已生效。
///
/// # 所有权与生命周期
///
/// 两个 f32 按值拥有，可 Copy、Clone；不借用电机或持有播放状态。归一化不会修改调用方已有副本。
///
/// # 参考
///
/// [`crate::core::framework::input::GamepadRumbleIntensity::new()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]、[`crate::core::framework::input::GamepadRumbleRequest::Add`]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadRumbleIntensity {
    /// 希望用于强电机的原始 f32 强度，归一化后的 0 表示零幅值、1 表示本通道最大请求幅值。
    ///
    /// # 约束
    ///
    /// 公开字段可以直接写入任意浮点值。clamped 独立处理此通道：有限值截到 [0,1]，NaN 与正负无穷都变为 0；它不与另一通道共享预算。
    ///
    /// # 所有权与生命周期
    ///
    /// 按值拥有，与设备执行进度无关；最大请求幅值不确认宿主或设备具备该电机能力。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleIntensity::new()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    pub strong_motor: f32,
    /// 希望用于弱电机的原始 f32 强度，归一化后的 0 表示零幅值、1 表示本通道最大请求幅值。
    ///
    /// # 约束
    ///
    /// 公开字段可以直接写入任意浮点值。clamped 独立处理此通道：有限值截到 [0,1]，NaN 与正负无穷都变为 0；它不与另一通道共享预算。
    ///
    /// # 所有权与生命周期
    ///
    /// 按值拥有，与设备执行进度无关；最大请求幅值不确认宿主或设备具备该电机能力。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleIntensity::new()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    pub weak_motor: f32,
}

impl GamepadRumbleIntensity {
    /// 同时请求强、弱两通道的最大归一化幅值。
    ///
    /// # 使用时机
    ///
    /// 作为 Add 请求的明确强度值使用，时长和设备目标由请求另行提供。
    ///
    /// # 约束
    ///
    /// 常量字段固定为 strong_motor=1.0、weak_motor=1.0，均已在 [0,1]；使用常量只选择请求参数，没有执行或能力确认。
    ///
    /// # 所有权与生命周期
    ///
    /// 可复制纯值，未绑定设备或效果生命周期。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::add()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    pub const MAX: Self = Self {
        strong_motor: 1.0,
        weak_motor: 1.0,
    };
    /// 请求强电机最大幅值、弱电机零幅值。
    ///
    /// # 使用时机
    ///
    /// 作为 Add 请求的明确强度值使用，时长和设备目标由请求另行提供。
    ///
    /// # 约束
    ///
    /// 常量字段固定为 strong_motor=1.0、weak_motor=0.0，均已在 [0,1]；使用常量只选择请求参数，没有执行或能力确认。
    ///
    /// # 所有权与生命周期
    ///
    /// 可复制纯值，未绑定设备或效果生命周期。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::add()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    pub const STRONG_MAX: Self = Self {
        strong_motor: 1.0,
        weak_motor: 0.0,
    };
    /// 请求弱电机最大幅值、强电机零幅值。
    ///
    /// # 使用时机
    ///
    /// 作为 Add 请求的明确强度值使用，时长和设备目标由请求另行提供。
    ///
    /// # 约束
    ///
    /// 常量字段固定为 strong_motor=0.0、weak_motor=1.0，均已在 [0,1]；使用常量只选择请求参数，没有执行或能力确认。
    ///
    /// # 所有权与生命周期
    ///
    /// 可复制纯值，未绑定设备或效果生命周期。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleRequest::add()`]、[`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
    pub const WEAK_MAX: Self = Self {
        strong_motor: 0.0,
        weak_motor: 1.0,
    };

    /// 按值保存两路原始浮点强度，便于构造请求参数或常量。
    ///
    /// # 使用时机
    ///
    /// 已选择两电机请求值时调用；在对外输出前仍需用 clamped 取得归一化副本。
    ///
    /// # 参数
    ///
    /// - `strong_motor`：强电机原始 f32，任何有限或非有限值均被原样保留。
    /// - `weak_motor`：弱电机原始 f32，与强电机独立保存，没有总幅值预算。
    ///
    /// # 返回
    ///
    /// 拥有型 Self 值，两字段与输入相同。构造不截断、不拒绝异常浮点，也没有硬件或排队结果。
    ///
    /// # 约束
    ///
    /// 构造保持原始数值；只有 clamped 显式建立每通道有限的 [0,1] 输出范围。
    ///
    /// # 所有权与生命周期
    ///
    /// 参数均按值复制，不借用调用者存储或设备；可在常量上下文使用。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]、[`crate::core::framework::input::GamepadRumbleRequest::add()`]
    pub const fn new(strong_motor: f32, weak_motor: f32) -> Self {
        Self {
            strong_motor,
            weak_motor,
        }
    }

    /// 为两电机各自生成可交给宿主的归一化强度副本。
    ///
    /// # 使用时机
    ///
    /// 从公开字段、new 或反序列化得到原始值后，在出站设备输出边界调用；当前运行时的 Add ABI 转换已使用此方法。
    ///
    /// # 参数
    ///
    /// - `self`：按值接收可 Copy 的原始强度，两个字段独立处理。
    ///
    /// # 返回
    ///
    /// 新的 Self：每个有限输入截到 [0,1]；NaN、正无穷和负无穷均返回该通道的 0.0。没有错误类型或设备执行结果。
    ///
    /// # 约束
    ///
    /// 非有限值被静音而不是把正无穷取成最大幅值。两通道之间没有归一化总和或优先级关系。
    ///
    /// # 所有权与生命周期
    ///
    /// 不借用或原地修改原值，不访问设备；调用方保留的其他副本仍含自己的原始字段。
    ///
    /// # 参考
    ///
    /// [`crate::core::framework::input::GamepadRumbleIntensity::new()`]、[`crate::core::framework::input::GamepadRumbleRequest::Add`]
    pub fn clamped(self) -> Self {
        Self {
            strong_motor: clamp_motor_intensity(self.strong_motor),
            weak_motor: clamp_motor_intensity(self.weak_motor),
        }
    }
}

/// 把单路原始电机强度转为有限的归一化幅值，供双电机归一化共用。
///
/// # 使用时机
///
/// 由 GamepadRumbleIntensity::clamped 对每个通道独立调用。
///
/// # 参数
///
/// - `value`：任意 f32 原始强度，允许有限越界值、NaN 和正负无穷。
///
/// # 返回
///
/// 有限输入截到 [0,1]；所有非有限输入返回 0.0。固定有效上下界，不产生错误返回。
///
/// # 约束
///
/// 先检查 is_finite，再对有限值执行 clamp；正无穷也走零值分支。该操作只归一化数据。
///
/// # 所有权与生命周期
///
/// 按值接收并返回 f32，无借用、设备访问或延迟工作。
///
/// # 参考
///
/// [`crate::core::framework::input::GamepadRumbleIntensity::clamped()`]
fn clamp_motor_intensity(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
