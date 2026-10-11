//! 把模拟按钮原始采样整形为幅值，并按相对于前次已接受幅值的变化量决定是否接收更新。
//!
//! # 使用时机与生命周期
//!
//! 当前默认管理器先调用接收方法，成功后保存幅值并判定按钮状态；被过滤的采样不会替换历史幅值或更新布尔按压集合。
//! 设置本身不保存历史；调用方提供的 `previous_value` 应与返回值处于同一缩放尺度。直接缩放方法有独立的输入准入边界。
//!
//! # 参考
//!
//! 参见 [`GamepadButtonAxisSettings`]、[`GamepadButtonAxisSettings::process_value()`]、[`GamepadButtonAxisSettings::scaled_value()`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
use serde::{Deserialize, Serialize};

use super::super::value_scaling::linear_remapping;

/// 默认低端边界为钳制后的原始幅值 `0.05`；输入不高于该值时缩放结果为零。
/// 边界是原始零到一采样尺度，不是按钮释放阈值。
///
/// # 使用时机与生命周期
///
/// 由默认设置复制读取；常量本身没有设备状态、历史采样或帧窗口。
///
/// # 参考
///
/// 参见 [`GAMEPAD_BUTTON_AXIS_HIGH`] 与 [`GamepadButtonAxisSettings::scaled_value()`]。
pub const GAMEPAD_BUTTON_AXIS_LOW: f32 = 0.05;
/// 默认高端边界为钳制后的原始幅值 `0.95`；输入不低于该值时缩放结果为一。
/// 默认低/高边界之间线性映射，端点比较包含相等。
///
/// # 使用时机与生命周期
///
/// 由默认设置复制读取；常量本身没有设备状态、历史采样或帧窗口。
///
/// # 参考
///
/// 参见 [`GAMEPAD_BUTTON_AXIS_LOW`] 与 [`GamepadButtonAxisSettings::scaled_value()`]。
pub const GAMEPAD_BUTTON_AXIS_HIGH: f32 = 0.95;
/// 默认更新阈值为缩放幅值的绝对变化量 `0.01`；达到相等边界也接受。
/// 它相对于前次已接受幅值比较，单位不是时间、百分比、事件数或原始采样变化量。
///
/// # 使用时机与生命周期
///
/// 由默认设置复制读取；常量本身没有设备状态、历史采样或帧窗口。
///
/// # 参考
///
/// 参见 [`GamepadButtonAxisSettings::process_value()`] 与 [`GamepadButtonAxisSettings::change_threshold`]。
pub const GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

/// 将模拟扳机值映射到稳定的零到一范围，再交给按钮滞回判定和动作值求值。
/// 自定义低高边界应递增；构造器不验证配置。
///
/// 保存模拟按钮幅值整形与更新筛选的三个参数，使调用方能把原始采样和前次已接受幅值分开处理。
///
/// # 配置与结果边界
///
/// 使用有限且满足 `0 <= low < high <= 1` 的边界可得到通常的零到一映射；更新阈值通常取有限正值并按缩放幅值解释。
/// [`Self::new()`]、公开字段和派生 Serde 不强制这些条件。边界为 `NaN`、无穷或其它异常组合时，实际浮点分支与线性算式可能产生非有限结果；仅原始采样在接收方法入口受有限性检查。
/// 对有限钳制值，有限但相等或逆序的低/高边界按先低端、后高端的优先级形成阶跃，方法不重排边界。变化阈值为零或负数时可接受相同有限幅值；阈值为 `NaN` 时有历史的比较不成立。
/// 当前管理器使用默认设置；结果作为幅值状态供快照观察，并驱动后续布尔滞回。当前按钮动作求值读取布尔按压/边沿，未直接把此幅值作为按钮动作强度。
///
/// # 使用时机与生命周期
///
/// 设置是拥有式可复制参数值，不保存前次采样、设备借用或可变历史；接收后是否存入缓存及何时清理由调用者负责。
/// `Clone`、`Copy` 不改变配置含义，`PartialEq` 按浮点字段比较；派生 Serde 不经过构造器或跨字段验证，具体格式是否支持非有限数值由格式决定。
/// 直接调用缩放方法会绕过接收方法的原始输入有限性与变化过滤；两种方法的返回结果不能相互当作同一准入承诺。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`Self::process_value()`]、[`Self::scaled_value()`]、[`crate::core::framework::input::GamepadButtonValueState`] 与 [`crate::core::framework::input::GamepadButtonSettings`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonAxisSettings {
    /// 钳制后原始采样的低端零输出边界，比较 `value <= low` 包含相等并先于高端分支。
    /// 通常与较高的有限 `high` 一起置于零到一范围；相等或逆序边界不会自动重排，低端条件成立时优先返回零。
    ///
    /// # 使用时机与生命周期
    ///
    /// 这是可公开写入和反序列化的参数副本；修改不会安装管理器设置或改变已有幅值缓存。
    /// 字段不受有限性检查，异常值会按浮点分支和线性算式影响输出。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::high`]、[`Self::new()`] 与 [`Self::scaled_value()`]。
    pub low: f32,
    /// 钳制后原始采样的高端一输出边界，低端条件未成立时检查 `value >= high`，包含相等。
    /// 通常使用比 `low` 高的有限域内值；这是原始采样尺度边界，与滞回按下阈值的缩放尺度不同。
    ///
    /// # 使用时机与生命周期
    ///
    /// 公开字段和派生 Serde 可直接提供任意 `f32`，没有验证、设备借用或缓存更新。
    /// 逆序或非有限配置遵循实际比较和映射结果，不保证通常的线性零到一输出。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::low`]、[`Self::scaled_value()`] 与 [`crate::core::framework::input::GamepadButtonSettings::press_threshold`]。
    pub high: f32,
    /// 已有历史时接受更新所需的缩放幅值绝对差下界，比较包含相等；`previous_value` 必须与新结果使用同一尺度。
    /// 正阈值过滤小幅变化，零或负阈值接受相同有限幅值；`NaN` 阈值使已有历史的有序比较为假，正无穷阈值通常拒绝有限差值。
    ///
    /// # 使用时机与生命周期
    ///
    /// 首次无历史的有限原始采样不检查此阈值；配置值本身不保存历史，也不度量帧数或时间。
    /// 字段可经公开写入或 Serde 绕过构造建议；调用方保存最后接受值后，后续小变化以该保存值为基线。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::process_value()`]、[`Self::new()`] 与 [`GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD`]。
    pub change_threshold: f32,
}

/// 提供普通模拟按钮的默认低端、高端与幅值变化筛选参数。
///
/// # 使用时机与生命周期
///
/// 每次构造新的参数值，不查询原始设备或幅值历史；当前默认管理器逐事件使用该实现。
///
/// # 参考
///
/// 参见 [`GamepadButtonAxisSettings::default()`]、[`GAMEPAD_BUTTON_AXIS_LOW`]、[`GAMEPAD_BUTTON_AXIS_HIGH`] 与 [`GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD`]。
impl Default for GamepadButtonAxisSettings {
    /// 创建低端 `0.05`、高端 `0.95`、缩放幅值变化阈值 `0.01` 的模拟按钮参数。
    ///
    /// # 返回
    ///
    /// 返回字段取三个当前默认常量的拥有式设置，没有错误返回或设备状态变化。
    ///
    /// # 使用时机
    ///
    /// 在普通采样接收或直接缩放前调用；返回值没有历史或借用，不能据此认为任何采样已被接收。
    ///
    /// # 约束
    ///
    /// 默认参数有限且低高有序；这不为经公开字段或反序列化构造的其它设置提供验证。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`GAMEPAD_BUTTON_AXIS_LOW`]、[`GAMEPAD_BUTTON_AXIS_HIGH`]、[`GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD`] 与 [`Self::process_value()`]。
    fn default() -> Self {
        Self {
            low: GAMEPAD_BUTTON_AXIS_LOW,
            high: GAMEPAD_BUTTON_AXIS_HIGH,
            change_threshold: GAMEPAD_BUTTON_AXIS_CHANGE_THRESHOLD,
        }
    }
}

/// 提供原始参数构造、历史更新准入与直接幅值整形，供消费者按所需的处理层级调用。
///
/// # 使用时机与生命周期
///
/// 所有方法按值使用参数，不持有设备或历史；调用方负责保存已接受值，并在取消或断开时清理实际状态。
/// 接收方法检查原始输入有限性；直接缩放不进行该准入和变化过滤。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`Self::process_value()`] 与 [`Self::scaled_value()`]。
impl GamepadButtonAxisSettings {
    /// 精确保存自定义模拟按钮整形边界和更新筛选阈值，支持常量求值。
    ///
    /// # 参数
    ///
    /// `low`、`high` 是钳制到零到一之后的原始采样边界；通常使用有限且满足 `0 <= low < high <= 1` 的值以得到常规映射。
    /// `change_threshold` 是新结果与前次已接受缩放幅值之间的绝对差下界，通常取有限正值；不是原始采样差或时间预算。
    /// 上述条件是调用方的配置建议；本方法不验证、排序、钳制或修正任何实参。
    ///
    /// # 返回
    ///
    /// 返回保留三个原始实参的拥有式设置值，没有错误返回。
    ///
    /// # 使用时机
    ///
    /// 在显式使用自定义设置的方法前构造；此调用不改变默认管理器配置，不分配设备资源或保存历史。
    /// 公开字段和派生 Serde 可构造同样未经验证的值，后续返回边界须按实际字段解释。
    ///
    /// # 约束
    ///
    /// 调用方选择整形边界与变化阈值；输入准入和结果有限性不能由构造成功推得。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::process_value()`]、[`Self::scaled_value()`]、[`Self::low`]、[`Self::high`] 与 [`Self::change_threshold`]。
    pub const fn new(low: f32, high: f32, change_threshold: f32) -> Self {
        Self {
            low,
            high,
            change_threshold,
        }
    }

    /// 无效输入或变化未达到阈值时返回 None；管理器据此保留上次幅值和按钮状态。
    ///
    /// 对原始模拟按钮采样执行有限性准入和幅值变化筛选，向调用者返回可以保存的新缩放值。
    ///
    /// # 参数
    ///
    /// `self` 是按值使用的整形设置；`raw_value` 是本次原始采样，只有有限值能进入缩放，有限越界采样仍可被钳制。
    /// `previous_value` 是前次已接受的缩放幅值；`None` 表示调用者没有历史，不能用未缩放原始值或被过滤采样替代该基线。
    ///
    /// # 返回
    ///
    /// 原始采样为 `NaN` 或正/负无穷时直接返回 `None`。其余先缩放；无历史时直接返回 `Some(value)`，已有历史时仅在绝对差不小于变化阈值时返回 `Some(value)`。
    /// 返回 `None` 表示本方法不建议替换幅值；方法没有写入或删除任何历史状态。当前默认管理器据此保留幅值与布尔按钮状态，但仍把原始事件送入录制器和帧事件队列。
    /// 本方法不检查配置、历史值或缩放结果的有限性；异常配置在无历史时仍可返回 `Some(NaN)`。历史为 `NaN` 或阈值为 `NaN` 时，有序变化比较为假；历史为无穷时差值仍按浮点规则比较。
    ///
    /// # 使用时机
    ///
    /// 在按钮滞回判定之前调用，并由调用者仅在接受时保存返回值；当前管理器的基线跨 `begin_frame` 持续，失焦或设备断开会清理对应缓存。
    /// 小变化相对于最后接受值累积，而不是与每次相邻采样比较；宿主按压标志改变本身不能绕过该幅值准入。
    /// 设置、实参和返回值均按值使用，不存在设备借用、异步处理或自动缓存更新；与直接缩放的准入约束不同。
    ///
    /// # 约束
    ///
    /// 历史基线须使用相同缩放尺度的最后接受值；原始输入有限不保证异常配置产生有限结果。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::scaled_value()`]、[`Self::change_threshold`]、[`crate::core::framework::input::GamepadButtonSettings::transition_for_value()`] 与 [`crate::core::framework::input::GamepadButtonValueState`]。
    pub fn process_value(self, raw_value: f32, previous_value: Option<f32>) -> Option<f32> {
        if !raw_value.is_finite() {
            return None;
        }
        let value = self.scaled_value(raw_value);
        if previous_value
            // 历史基线是最后一次接受的缩放值；被过滤的小变化不会替换它，后续变化仍据此累积。
            // 无历史时跳过比较，故 raw 的有限性检查不能替代配置和缩放结果的约束。
            .map(|previous| (value - previous).abs() >= self.change_threshold)
            .unwrap_or(true)
        {
            Some(value)
        } else {
            None
        }
    }

    /// 直接钳制和整形原始模拟按钮幅值，用于需要映射结果而不需要更新筛选的调用。
    ///
    /// # 参数
    ///
    /// `self` 是低/高边界参数副本；`raw_value` 是待映射原始值，首先钳制到零到一。
    /// 本方法不执行有限性准入；正/负无穷在此可钳制为端点，`NaN` 仍为 `NaN`。边界也不受验证。
    ///
    /// # 返回
    ///
    /// 钳制值不高于低边界时返回零，否则不低于高边界时返回一，中间值按低/高边界线性映射。
    /// 低端条件优先且端点包含相等；对有限钳制值，有限的相等或逆序边界按此优先级产生阶跃，不重新排序。`NaN` 输入可使两个端点比较都为假而进入中段。
    /// 默认参数或有限且满足 `0 <= low < high <= 1` 的通常配置对有限输入产生零到一幅值；异常配置或`NaN` 输入不保证有限结果，线性算式后没有再次钳制或验证。
    /// 该返回值没有相对于历史的变化判断；无穷输入在这里可得到端点，在 `process_value` 入口则会被拒绝。
    ///
    /// # 使用时机
    ///
    /// 需要单纯映射时调用；设置与结果按值传递，不更新缓存、布尔按压集合或帧边沿。
    /// 需要接受策略时显式使用 `process_value` 并保存其 `Some` 结果，不能把本方法返回值视为一次已接受更新。
    ///
    /// # 约束
    ///
    /// 直接映射绕过采样有限性和历史变化筛选；不能把返回值当作一次已接受更新。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::process_value()`]、[`Self::low`]、[`Self::high`] 与 [`Self::new()`]。
    pub fn scaled_value(self, raw_value: f32) -> f32 {
        let value = raw_value.clamp(0.0, 1.0);
        if value <= self.low {
            0.0
        } else if value >= self.high {
            1.0
        } else {
            linear_remapping(value, self.low, self.high, 0.0, 1.0)
        }
    }
}
