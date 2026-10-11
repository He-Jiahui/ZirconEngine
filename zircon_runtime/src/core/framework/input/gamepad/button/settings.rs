//! 提供模拟按钮的双阈值比较与候选状态转换，供输入管理器在接受幅值变化后决定按压或释放。
//!
//! # 使用时机与生命周期
//!
//! 设置是可复制的值，当前按住状态由每次调用提供；本模块不缓存前次幅值或按钮状态。
//! 默认管理器对已接受的缩放幅值使用默认阈值，并结合宿主按压标志更新布尔集合；自定义设置的构造不会改变该管理器配置。
//!
//! # 参考
//!
//! 参见 [`GamepadButtonSettings`]、[`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`] 与 [`crate::core::framework::input::ButtonInputState`]。
use serde::{Deserialize, Serialize};

/// 默认未按住按钮的按下阈值为归一化幅值 `0.75`，比较在相等时也成立。
/// 默认管理器先缩放并过滤原始值，再使用该幅值尺度；这不是原始设备采样的直接阈值。
///
/// # 使用时机与生命周期
///
/// 由默认设置读取；常量没有设备状态，与较低的默认释放阈值共同形成滞回区间。
///
/// # 参考
///
/// 参见 [`GAMEPAD_BUTTON_RELEASE_THRESHOLD`]、[`GamepadButtonSettings::default()`] 与 [`GamepadButtonSettings::is_pressed()`]。
pub const GAMEPAD_BUTTON_PRESS_THRESHOLD: f32 = 0.75;
/// 默认已按住按钮的释放阈值为归一化幅值 `0.65`，比较在相等时也成立。
/// 它低于默认按下阈值，使位于二者之间的已接受幅值保持调用方给定的当前按压状态。
///
/// # 使用时机与生命周期
///
/// 由默认设置读取；这是无状态常量，具体释放与帧边沿仍由管理器更新。
///
/// # 参考
///
/// 参见 [`GAMEPAD_BUTTON_PRESS_THRESHOLD`]、[`GamepadButtonSettings::default()`] 与 [`GamepadButtonSettings::is_released()`]。
pub const GAMEPAD_BUTTON_RELEASE_THRESHOLD: f32 = 0.65;

/// 按下与释放采用分离阈值，防止模拟按钮在阈值附近反复抖动。
/// 自定义配置应让释放阈值低于按下阈值；构造器不验证配置。
///
/// 保存把已接受的模拟幅值解释为按钮状态时使用的按下和释放阈值；当前状态由调用者维护。
///
/// # 配置与约束
///
/// 通常使用有限且位于零到一幅值尺度内的阈值，并让释放阈值严格低于按下阈值，从而得到稳定滞回区间。
/// [`Self::new()`]、公开字段和派生 Serde 均不检查有限性、范围或二者顺序。比较方法不钳制输入，也不修正自定义配置。
/// 若释放阈值大于或等于按下阈值，同一幅值可能同时满足独立比较；转换方法只看当前状态对应的一侧，重复按候选更新后可交替按下和释放。
/// `NaN` 输入使两个比较都为假；某个阈值是 `NaN` 时，该阈值的比较为假。无穷值仍按普通 `f32` 比较处理。
///
/// # 使用时机与生命周期
///
/// 在一次幅值更新被接收后按值调用；`Clone`、`Copy` 保存独立参数值，`PartialEq` 按浮点字段比较，含 `NaN` 的值可能与自身不等。
/// 派生 Serde 不调用构造器或执行跨字段检查，具体格式能否表示非有限值由格式决定；本类型没有可变历史、设备借用或事件派发能力。
/// 当前默认管理器每次使用默认设置，自定义构造值只影响显式使用该值的调用者；不要把局部比较结果视为已修改管理器状态。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`Self::transition_for_value()`]、[`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`] 与 [`crate::core::framework::input::ButtonInputState`]。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GamepadButtonSettings {
    /// 调用方当前未按住时用于候选按下的幅值下界，`value >= press_threshold` 包含相等边界。
    /// 默认尺度是缩放后的零到一幅值；为稳定滞回通常取有限值并高于释放阈值。
    ///
    /// # 使用时机与生命周期
    ///
    /// 这是可公开改写或经 Serde 读取的参数副本，没有自动范围/顺序验证，也不会同步安装到输入管理器。
    /// 字段为 `NaN` 时对应按下比较为假；无穷或逆序配置按实际浮点比较解释。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_pressed()`]、[`Self::transition_for_value()`] 与 [`Self::release_threshold`]。
    pub press_threshold: f32,
    /// 调用方当前已按住时用于候选释放的幅值上界，`value <= release_threshold` 包含相等边界。
    /// 默认尺度是缩放后的零到一幅值；有限且严格低于按下阈值时，两阈值之间保留原按压状态。
    ///
    /// # 使用时机与生命周期
    ///
    /// 公开字段和派生 Serde 可直接提供任意 `f32`，不执行验证或重排，也没有设备状态借用。
    /// 字段为 `NaN` 时对应释放比较为假；改变此参数值只影响后来显式使用该副本的调用。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_released()`]、[`Self::transition_for_value()`] 与 [`Self::press_threshold`]。
    pub release_threshold: f32,
}

/// 为按钮幅值判定提供默认双阈值参数，使普通调用共享同一滞回区间。
///
/// # 使用时机与生命周期
///
/// 生成新的参数值，不查询或修改管理器及设备状态；此实现的固定阈值不代表所有自定义配置都有效。
///
/// # 参考
///
/// 参见 [`GamepadButtonSettings::default()`]、[`GAMEPAD_BUTTON_PRESS_THRESHOLD`] 与 [`GAMEPAD_BUTTON_RELEASE_THRESHOLD`]。
impl Default for GamepadButtonSettings {
    /// 创建按下阈值 `0.75`、释放阈值 `0.65` 的按钮判定参数。
    ///
    /// # 返回
    ///
    /// 返回拥有式设置值，字段取当前两个默认常量；没有错误返回或设备状态变化。
    ///
    /// # 使用时机
    ///
    /// 在普通归一化幅值比较前调用；返回值可复制和独立保留，不借用管理器，也不开始设备处理。
    ///
    /// # 约束
    ///
    /// 固定阈值适用于当前默认零到一幅值尺度；创建参数不证明设备或任何采样已经被处理。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`GAMEPAD_BUTTON_PRESS_THRESHOLD`]、[`GAMEPAD_BUTTON_RELEASE_THRESHOLD`] 与 [`Self::transition_for_value()`]。
    fn default() -> Self {
        Self {
            press_threshold: GAMEPAD_BUTTON_PRESS_THRESHOLD,
            release_threshold: GAMEPAD_BUTTON_RELEASE_THRESHOLD,
        }
    }
}

/// 集中参数构造、独立阈值比较和依赖调用方当前状态的候选转换。
///
/// # 使用时机与生命周期
///
/// 方法不保存历史或借用设备；`self` 按值传入，因设置可复制，调用方可继续持有原参数。
/// 实际接受幅值、更新按压集合及产生帧边沿由使用这些结果的管理器负责。
///
/// # 参考
///
/// 参见 [`Self::new()`]、[`Self::is_pressed()`]、[`Self::is_released()`] 与 [`Self::transition_for_value()`]。
impl GamepadButtonSettings {
    /// 按调用方给定的双阈值构造参数值，支持常量求值和自定义幅值解释。
    ///
    /// # 参数
    ///
    /// `press_threshold` 是未按住时的按下下界，`release_threshold` 是已按住时的释放上界，均按调用者所用幅值尺度解释。
    /// 为得到稳定的零到一幅值滞回，通常选择有限的域内值并使释放阈值严格较低；本方法不强制这些建议，也不检查非有限或逆序配置。
    ///
    /// # 返回
    ///
    /// 返回精确保存两个实参的拥有式设置值；没有错误返回、钳制、排序或验证。
    ///
    /// # 使用时机
    ///
    /// 在显式比较或自定义处理前创建；构造不会安装到默认管理器，也不触发按压、释放或设备操作。
    /// 返回值没有借用或按钮历史，调用者须在每次转换时另外提供当前按压状态。
    ///
    /// # 约束
    ///
    /// 稳定滞回由调用方选择有限且释放阈值严格较低的值；此入口不强制配置建议。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::transition_for_value()`]、[`Self::press_threshold`] 与 [`Self::release_threshold`]。
    pub const fn new(press_threshold: f32, release_threshold: f32) -> Self {
        Self {
            press_threshold,
            release_threshold,
        }
    }

    /// 检查幅值是否达到按下下界，供未按住按钮的候选转换使用。
    ///
    /// # 参数
    ///
    /// `self` 是按值传入的阈值副本；`value` 是待解释幅值，当前默认管理器使用已接受的缩放值。
    /// 本方法允许直接比较任意 `f32`，不执行原始采样过滤、钳制或配置验证。
    ///
    /// # 返回
    ///
    /// 返回 `value >= press_threshold` 的布尔结果，相等返回 `true`；输入或该阈值为 `NaN` 时返回 `false`。
    /// 无穷值参与普通浮点比较。结果不含当前状态，因此不证明按钮此前释放、已经按下或本帧生成了边沿。
    ///
    /// # 使用时机
    ///
    /// 独立查询某个阈值条件时调用；需要滞回时另用状态转换方法。调用不存储幅值或改变设备状态，返回值不保留借用。
    ///
    /// # 约束
    ///
    /// 单一比较不带当前状态；需要滞回时使用状态转换接口，并由调用者应用候选。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::press_threshold`]、[`Self::is_released()`] 与 [`Self::transition_for_value()`]。
    pub fn is_pressed(self, value: f32) -> bool {
        value >= self.press_threshold
    }

    /// 检查幅值是否降至释放上界，供已按住按钮的候选释放判定使用。
    ///
    /// # 参数
    ///
    /// `self` 是按值传入的阈值副本；`value` 是待解释幅值，当前默认管理器使用已接受的缩放值。
    /// 本方法不筛除非有限值、不钳制输入，也不验证与按下阈值的关系。
    ///
    /// # 返回
    ///
    /// 返回 `value <= release_threshold` 的布尔结果，相等返回 `true`；输入或该阈值为 `NaN` 时返回 `false`。
    /// 无穷值按普通浮点比较解释。结果不表示此前确实按住、已经释放或已生成帧内释放边沿。
    ///
    /// # 使用时机
    ///
    /// 在候选转换或管理器结合宿主释放标志时查询；调用没有幅值历史或状态修改，布尔返回值独立于设置生命周期。
    ///
    /// # 约束
    ///
    /// 单一比较不建立此前按住或新释放边沿；配置和输入按实际浮点规则解释。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::release_threshold`]、[`Self::is_pressed()`] 与 [`Self::transition_for_value()`]。
    pub fn is_released(self, value: f32) -> bool {
        value <= self.release_threshold
    }

    /// 根据调用方提供的当前按压状态，选择一侧阈值判断本次幅值是否建议改变状态。
    ///
    /// # 参数
    ///
    /// `self` 是双阈值参数副本；`value` 是本次待解释幅值，当前默认管理器在缩放及变化过滤后调用。
    /// `currently_pressed` 必须表达调用者想延续的当前按住状态；本方法不自行查询或记住它。
    ///
    /// # 返回
    ///
    /// 当前已按住且幅值不高于释放阈值时返回 `Some(false)`；当前未按住且幅值不低于按下阈值时返回 `Some(true)`。
    /// 其余返回 `None`，表示没有候选转换，调用者可保持原状态；它不等同于最终按住状态，也不证明已执行状态更新或派发事件。
    /// `NaN` 输入没有候选转换。阈值相等或逆序时，重叠区内结果随给定当前状态切换，方法不重排或修复配置。
    ///
    /// # 使用时机
    ///
    /// 在接受幅值更新之后、修改按压集合之前调用。方法只返回建议，没有设备借用、可变历史或帧边沿副作用。
    /// 当前默认管理器还结合宿主按压标志及独立释放条件；幅值被过滤时它不执行此转换。因此本函数结果不能覆盖整个输入服务的最终状态契约。
    ///
    /// # 约束
    ///
    /// 调用者须提供想延续的真实当前状态；返回候选与实际管理器结合宿主标志的更新规则分开解释。
    ///
    /// # 所有权与生命周期
    ///
    /// 设置与结果都是拥有式值；调用不持有设备借用或启动异步工作，状态保存及清理由实际消费者负责。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::is_pressed()`]、[`Self::is_released()`]、[`crate::core::framework::input::GamepadButtonAxisSettings::process_value()`] 与 [`crate::core::framework::input::ButtonInputState::press()`]。
    pub fn transition_for_value(self, value: f32, currently_pressed: bool) -> Option<bool> {
        if currently_pressed {
            self.is_released(value).then_some(false)
        } else {
            self.is_pressed(value).then_some(true)
        }
    }
}
