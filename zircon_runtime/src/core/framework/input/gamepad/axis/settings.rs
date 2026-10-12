//! 轴采样整形与变化筛选参数，区分直接映射和带历史的更新准入。
//!
//! # 使用时机、约束与生命周期
//!
//! 公开构造不验证配置；采样有限性、配置有效性和结果有限性是不同边界。设置不保存历史或改变默认管理器。
//! 本模块只组织值协议或纯计算，设备连接、缓存、帧推进及服务生命周期由实际消费者负责。
//!
//! # 参考
//!
//! 参见 [`crate::core::framework::input::GamepadAxisState`]、[`crate::core::framework::input::InputEvent::GamepadAxis`]。
use serde::{Deserialize, Serialize};

use super::super::value_scaling::linear_remapping;

/// 默认死区的负侧闭区间端点，数值为 -0.05。
///
/// # 使用时机与约束
///
/// 由默认设置复制到对应字段；是无单位幅值参数，不是时间或设备枚举。自定义设置可选择其它值，不受本常量强制。
/// 常量自身不保存采样或历史；是否接收变化由处理方法及调用方持有的最后接受值决定。
///
/// # 参考
///
/// 参见 [`GamepadAxisSettings::deadzone_lowerbound`]、[`GamepadAxisSettings::default()`]、[`GamepadAxisSettings::process_value()`]。
pub const GAMEPAD_AXIS_DEADZONE_LOWER: f32 = -0.05;
/// 默认死区的正侧闭区间端点，数值为 0.05。
///
/// # 使用时机与约束
///
/// 由默认设置复制到对应字段；是无单位幅值参数，不是时间或设备枚举。自定义设置可选择其它值，不受本常量强制。
/// 常量自身不保存采样或历史；是否接收变化由处理方法及调用方持有的最后接受值决定。
///
/// # 参考
///
/// 参见 [`GamepadAxisSettings::deadzone_upperbound`]、[`GamepadAxisSettings::default()`]、[`GamepadAxisSettings::process_value()`]。
pub const GAMEPAD_AXIS_DEADZONE_UPPER: f32 = 0.05;
/// 默认活区负侧饱和边界，数值为 -1.0。
///
/// # 使用时机与约束
///
/// 由默认设置复制到对应字段；是无单位幅值参数，不是时间或设备枚举。自定义设置可选择其它值，不受本常量强制。
/// 常量自身不保存采样或历史；是否接收变化由处理方法及调用方持有的最后接受值决定。
///
/// # 参考
///
/// 参见 [`GamepadAxisSettings::livezone_lowerbound`]、[`GamepadAxisSettings::default()`]、[`GamepadAxisSettings::process_value()`]。
pub const GAMEPAD_AXIS_LIVEZONE_LOWER: f32 = -1.0;
/// 默认活区正侧饱和边界，数值为 1.0。
///
/// # 使用时机与约束
///
/// 由默认设置复制到对应字段；是无单位幅值参数，不是时间或设备枚举。自定义设置可选择其它值，不受本常量强制。
/// 常量自身不保存采样或历史；是否接收变化由处理方法及调用方持有的最后接受值决定。
///
/// # 参考
///
/// 参见 [`GamepadAxisSettings::livezone_upperbound`]、[`GamepadAxisSettings::default()`]、[`GamepadAxisSettings::process_value()`]。
pub const GAMEPAD_AXIS_LIVEZONE_UPPER: f32 = 1.0;
/// 默认更新筛选所需的缩放值绝对变化下界，数值为 0.01。
///
/// # 使用时机与约束
///
/// 由默认设置复制到对应字段；是无单位幅值参数，不是时间或设备枚举。自定义设置可选择其它值，不受本常量强制。
/// 常量自身不保存采样或历史；是否接收变化由处理方法及调用方持有的最后接受值决定。
///
/// # 参考
///
/// 参见 [`GamepadAxisSettings::change_threshold`]、[`GamepadAxisSettings::default()`]、[`GamepadAxisSettings::process_value()`]。
pub const GAMEPAD_AXIS_CHANGE_THRESHOLD: f32 = 0.01;

/// 在设备原始轴值进入帧快照前应用死区、活区和变化阈值，稳定动作映射的模拟值。
/// 自定义边界应保持活区下界、死区下界、死区上界、活区上界依次递增；构造器不验证配置。
/// 携带原始轴采样的死区、活区与更新筛选策略，使调用方得到稳定的模拟轴状态。
///
/// # 处理职责
///
/// process_value 内部先调用 scaled_value 映射采样，再按最后接受的缩放值检查变化阈值；调用方只保存 Some 返回值。直接 scaled_value 仅映射，不执行采样有限性准入或历史筛选。
///
/// # 配置与约束
///
/// 通常使用有限且满足 -1 <= livezone_lowerbound < deadzone_lowerbound <= 0 <= deadzone_upperbound < livezone_upperbound <= 1 的边界，并选择有限正 change_threshold。死区是包含端点的闭区间。
/// new、公开字段和派生 Serde 不验证、排序或修正配置。异常配置、历史或结果均按实际浮点比较处理；raw 有限不能证明映射结果有限，process_value 的 Some 也不是验证回执。
///
/// # 使用时机与生命周期
///
/// 在接收原始事件并写入当前值之前使用。当前默认管理器每次采用默认设置；构造一个自定义值不会改变该管理器的配置。
/// 设置是 Copy 的拥有式参数，不持有设备、缓存或上一值。历史跨帧保存及失焦、断开清理由调用方负责；含 NaN 的设置使用浮点 PartialEq，比较可不自反。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`Self::default()`]、[`Self::process_value()`]、[`Self::scaled_value()`]、[`crate::core::framework::input::GamepadAxisState`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadAxisSettings {
    /// 正侧活区饱和边界；达到此边界的有效值映射到正端点。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 通常选择大于 deadzone_upperbound 且不超过 1 的有限值。固定 -1 到 1 的钳制先执行，死区和零值条件优先。
    /// 在处理采样时使用；字段公开且构造及 Serde 不验证它。字段不保存历史，修改配置后历史尺度是否仍可比较由调用方负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::deadzone_upperbound`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub livezone_upperbound: f32,
    /// 死区的正侧闭区间端点；与负侧端点之间的值归零。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 通常选择不小于 0 且小于 livezone_upperbound 的有限值；正侧中段从此处向活区端点映射。
    /// 在处理采样时使用；字段公开且构造及 Serde 不验证它。字段不保存历史，修改配置后历史尺度是否仍可比较由调用方负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::deadzone_lowerbound`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub deadzone_upperbound: f32,
    /// 死区的负侧闭区间端点；与正侧端点一起界定归零区间。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 通常选择不大于 0 且大于 livezone_lowerbound 的有限值；负侧中段从活区端点向此处映射。
    /// 在处理采样时使用；字段公开且构造及 Serde 不验证它。字段不保存历史，修改配置后历史尺度是否仍可比较由调用方负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::deadzone_upperbound`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub deadzone_lowerbound: f32,
    /// 负侧活区饱和边界；达到或低于此边界的有效值映射到负端点。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 通常选择小于 deadzone_lowerbound 且不低于 -1 的有限值。死区优先，异常逆序配置不会被自动纠正。
    /// 在处理采样时使用；字段公开且构造及 Serde 不验证它。字段不保存历史，修改配置后历史尺度是否仍可比较由调用方负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::deadzone_lowerbound`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub livezone_lowerbound: f32,
    /// 新缩放值与最后接受的缩放值之差所需的绝对下界。
    ///
    /// # 使用时机、约束与生命周期
    ///
    /// 通常选择有限正值；比较包含相等。没有历史时跳过此比较；NaN 阈值在已有历史时使有序比较为假，非正阈值可接受零差。
    /// 在处理采样时使用；字段公开且构造及 Serde 不验证它。字段不保存历史，修改配置后历史尺度是否仍可比较由调用方负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::deadzone_lowerbound`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub change_threshold: f32,
}

impl Default for GamepadAxisSettings {
    /// 构造当前默认输入管理器所使用的轴整形参数组。
    ///
    /// # 返回
    ///
    /// 返回死区 [-0.05, 0.05]、活区端点 -1 与 1、变化阈值 0.01 的独立设置值；不返回验证错误。
    ///
    /// # 使用时机
    ///
    /// 需要仓库默认轴策略时调用；默认管理器处理每次轴事件时使用同样的参数。
    ///
    /// # 约束
    ///
    /// 默认参数彼此有效，但该函数不校验设备输入或外部历史，也不能约束之后对公开字段的修改。
    ///
    /// # 所有权与生命周期
    ///
    /// 每次返回拥有式副本，没有设备或状态借用，不清空历史或改变已有缓存。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::process_value()`]、[`Self::new()`]、[`Self::change_threshold`]。
    fn default() -> Self {
        Self {
            livezone_upperbound: GAMEPAD_AXIS_LIVEZONE_UPPER,
            deadzone_upperbound: GAMEPAD_AXIS_DEADZONE_UPPER,
            deadzone_lowerbound: GAMEPAD_AXIS_DEADZONE_LOWER,
            livezone_lowerbound: GAMEPAD_AXIS_LIVEZONE_LOWER,
            change_threshold: GAMEPAD_AXIS_CHANGE_THRESHOLD,
        }
    }
}

impl GamepadAxisSettings {
    /// 精确保存自定义轴整形边界与变化阈值，支持常量构造。
    ///
    /// # 参数
    ///
    /// `livezone_lowerbound`、`livezone_upperbound` 是负、正侧饱和边界；`deadzone_lowerbound`、`deadzone_upperbound` 是归零闭区间端点。
    /// `change_threshold` 是处理后幅值相对最后接受幅值的绝对变化下界，单位不是时间或原始采样差。
    ///
    /// # 返回
    ///
    /// 返回逐项保存实参的拥有式设置值，没有错误返回，不验证、排序或钳制实参。
    ///
    /// # 使用时机
    ///
    /// 在明确需要自定义 process_value 或 scaled_value 策略时调用；不会更新默认管理器的设置。
    ///
    /// # 约束
    ///
    /// 通常采用有限、有序且以零为中心的边界和有限正阈值，具体关系见类型契约。构造成功不保证后续结果有限。
    ///
    /// # 所有权与生命周期
    ///
    /// 实参与结果按值使用；设备状态、配置切换和历史基线由调用方管理，方法不启动工作或持有借用。
    ///
    /// # 参考
    ///
    /// 参见 [`Self`]、[`Self::process_value()`]、[`Self::scaled_value()`]。
    pub const fn new(
        livezone_lowerbound: f32,
        deadzone_lowerbound: f32,
        deadzone_upperbound: f32,
        livezone_upperbound: f32,
        change_threshold: f32,
    ) -> Self {
        Self {
            livezone_upperbound,
            deadzone_upperbound,
            deadzone_lowerbound,
            livezone_lowerbound,
            change_threshold,
        }
    }

    /// 返回 `None` 表示非有限输入或未跨过变化阈值，调用方应沿用上次已发布的值。
    /// 决定本次原始轴采样是否成为新的已接受缩放值，供调用方更新当前值缓存。
    ///
    /// # 参数
    ///
    /// `self` 是本次使用的设置副本。`raw_value` 是原始设备幅值，必须有限才能进入映射；有限越界值仍会钳制。
    /// `previous_value` 是同一控件、同一缩放尺度的最后接受值；None 表示没有历史，被过滤的采样不应替换这一基线。
    ///
    /// # 返回
    ///
    /// 原始采样为 NaN 或正负无穷时返回 None。否则先映射，无历史时直接返回 Some(value)，有历史时只在绝对差不小于 change_threshold 时返回 Some(value)。
    /// None 不删除或修改缓存；当前默认管理器保留已有值，但仍把原始事件送入事件队列与录制器。配置、历史和映射结果的有限性不受检查：异常配置无历史时可返回 Some(NaN)，NaN 历史或阈值使变化比较为假。
    ///
    /// # 使用时机
    ///
    /// 在写入轴缓存与生成变化记录前调用，仅在接受时保存结果。当前默认管理器的历史跨 begin_frame 持续，失焦或断开会清理相应缓存。
    ///
    /// # 约束
    ///
    /// 比较的是处理后值的绝对差，包含相等；小变化相对最后接受值累计。无历史跳过阈值，不能把 Some 当作输入或配置已经验证的证明。
    ///
    /// # 所有权与生命周期
    ///
    /// 方法只使用值副本，不持有、写入或清理历史。首次零值可被接受并存为当前样本；是否生成变化记录由调用方另行比较决定。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::scaled_value()`]、[`Self::change_threshold`]、[`crate::core::framework::input::GamepadAxisState`]、[`crate::core::framework::input::GamepadAxisTransition`]。
    pub fn process_value(self, raw_value: f32, previous_value: Option<f32>) -> Option<f32> {
        if !raw_value.is_finite() {
            return None;
        }
        let value = self.scaled_value(raw_value);
        // 比较基线是最后接受的缩放值；被过滤的小变化不替换它，无历史时直接跳过比较。
        if previous_value
            .map(|previous| (value - previous).abs() >= self.change_threshold)
            .unwrap_or(true)
        {
            Some(value)
        } else {
            None
        }
    }

    /// 直接将原始轴幅值映射为带死区和饱和端点的模拟值，不决定是否发布更新。
    ///
    /// # 参数
    ///
    /// `self` 提供未验证的边界；`raw_value` 首先固定钳制到 -1 到 1，再按配置执行死区与活区处理。
    /// 本入口不检查有限性：无穷可钳制为端点，NaN 留到浮点条件判断。
    ///
    /// # 返回
    ///
    /// 闭区间死区映射为零，正负活区端点分别映射为 1 与 -1；两侧中段分别线性映射到零和相应端点。零值判断优先，其后正端点、负端点、正侧插值、负侧插值，最后回退为零。
    /// 默认或通常有效配置对有限输入产生 -1 到 1 结果。本方法没有最后结果校验或再次钳制；异常配置可能产生非有限值。NaN 原始输入使有序条件为假而回退零，和 process_value 直接拒绝 NaN 的准入不同。
    ///
    /// # 使用时机
    ///
    /// 只需要映射结果时调用；需要基于历史筛选更新时使用 process_value，它会在内部调用本方法。
    ///
    /// # 约束
    ///
    /// 配置不自动排序或验证。有限相等的死区/活区边界会先命中死区或饱和条件，不能仅凭相等就断言一定进入零分母插值。映射结果不表示变化已被接受。
    ///
    /// # 所有权与生命周期
    ///
    /// 只产生拥有式数值，不修改设备、当前缓存或帧变化集合，不保存上一值。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::process_value()`]、[`Self::new()`]、[`Self::deadzone_lowerbound`]、[`Self::deadzone_upperbound`]。
    pub fn scaled_value(self, raw_value: f32) -> f32 {
        // 固定采样范围先于配置死区和活区处理；配置不会扩张有限原始采样钳制后的 -1 到 1 范围。
        let value = self.clamped_value(raw_value.clamp(-1.0, 1.0));
        if value == 0.0 {
            0.0
        } else if value >= self.livezone_upperbound {
            1.0
        } else if value <= self.livezone_lowerbound {
            -1.0
        } else if value >= self.deadzone_upperbound {
            linear_remapping(
                value,
                self.deadzone_upperbound,
                self.livezone_upperbound,
                0.0,
                1.0,
            )
        } else if value <= self.deadzone_lowerbound {
            linear_remapping(
                value,
                self.livezone_lowerbound,
                self.deadzone_lowerbound,
                -1.0,
                0.0,
            )
        } else {
            0.0
        }
    }

    /// 在已限定采样范围的值上应用配置死区与饱和端点，供后续分段缩放使用。
    ///
    /// # 参数
    ///
    /// `self` 提供死区与活区边界；`value` 是 scaled_value 已固定钳制到 -1 到 1 的中间值，也可能仍为 NaN。
    ///
    /// # 返回
    ///
    /// 先检查包含两端的死区，命中时返回零；否则依次检查正活区和负活区，返回 1 或 -1；其余值原样返回。
    /// 返回端点由后续 scaled_value 条件继续处理；该辅助方法不执行线性映射，也不验证结果或历史。
    ///
    /// # 使用时机
    ///
    /// 仅在 scaled_value 固定钳制后调用，不能替代该公共入口或 process_value 的原始有限性检查。
    ///
    /// # 约束
    ///
    /// 死区条件优先于活区饱和，异常重叠或逆序配置按此顺序处理。NaN 的有序比较为假，在本方法保留到后续分支。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置和结果按值传递；不拥有设备、缓存或其它借用，不产生事件或帧记录。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::scaled_value()`]、[`Self::process_value()`]、[`Self::deadzone_lowerbound`]、[`Self::livezone_upperbound`]。
    fn clamped_value(self, value: f32) -> f32 {
        if self.deadzone_lowerbound <= value && value <= self.deadzone_upperbound {
            0.0
        } else if value >= self.livezone_upperbound {
            1.0
        } else if value <= self.livezone_lowerbound {
            -1.0
        } else {
            value
        }
    }
}
