//! 定义命名动作绑定的拥有数据表示、构造规范化与单轴方向投影。
//!
//! # 使用时机
//! 先建立绑定值，再由动作映射收纳并由具体消费者安装；单独构造值不会注册动作或设备。
//!
//! # 约束
//! 构造器只整理本值。公开字段与派生反序列化允许绕过列表排序去重；按钮组合和轴聚合策略由消费者实现。
//! Core 中这些值可独立存在；当前运行时的具体求值器 `InputActionEvaluator` 是已确认消费者，具体策略不扩写为动作服务 trait 的统一保证。该求值器所在的 `crate::input` 模块仅在启用 `native-host` 时提供。
//!
//! # 生命周期
//! 字符串与列表由绑定拥有，轴来源/方向是可复制值；本模块不保存帧借用或运行期工作区。
//!
//! # 参考
//! 参见 [`InputBinding`]、[`InputAxisBinding`]、[`InputAxisDirection`]。
use serde::{Deserialize, Serialize};

use super::{GamepadAxis, GamepadId, InputButton};

/// 选择整根轴或单侧轴参与动作；单侧值折为非负幅值，便于同一动作绑定不同方向。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 选择一个物理轴标量的完整有符号值或某一侧的非负幅值。
///
/// # 使用时机
/// 由轴绑定保存，在调用方向投影时使用。
///
/// # 约束
/// 默认 variant 为 Full；派生顺序按当前声明排列，用于构造器对轴三元身份排序，不表示状态转移。
/// 没有死区、增益或二维输出分量；类型可复制，派生 Serde 只恢复方向选择。
///
/// # 生命周期
/// 方向值无借用，不拥有设备或输入帧。
///
/// # 参考
/// 参见 [`InputAxisDirection::value`]、[`InputAxisBinding`]。
pub enum InputAxisDirection {
    #[default]
    /// 保留有限源值的正负号，并把幅度限定在单位闭区间。
    ///
    /// # 约束
    /// 此项是默认方向；非有限源值归零，两个符号的零统一为正零。
    ///
    /// # 生命周期
    /// 仅表示投影策略，实际数值在每次调用时由调用方提供。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection::value`]。
    Full,
    /// 只保留源值的正侧幅值，负侧归零。
    ///
    /// # 约束
    /// 结果位于 [0, 1]；非有限输入归零，不是一个独立轴或增益参数。
    ///
    /// # 生命周期
    /// 方向值可复制，不读取设备状态。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection::value`]。
    Positive,
    /// 把源值的负侧映射为非负幅值，正侧归零。
    ///
    /// # 约束
    /// 先对有限源值夹取，再取负侧幅值，结果位于 [0, 1]；不会保留负号。
    ///
    /// # 生命周期
    /// 方向值不保存上次采样或边沿状态。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection::value`]。
    Negative,
}

/// 提供无状态的标量方向投影。
///
/// # 使用时机
/// 轴绑定或独立调用者已有本次源值时使用。
///
/// # 约束
/// 仅对提供的数值处理，不决定其设备身份、上游缩放或动作消费规则。
///
/// # 生命周期
/// 调用不保留状态，返回值不借用输入。
///
/// # 参考
/// 参见 [`InputAxisDirection::value`]、[`normalized_axis_value`]。
impl InputAxisDirection {
    /// 夹取单轴源值并按当前方向返回标量。
    ///
    /// # 参数
    /// - `self`：按值提供本次方向；类型可复制，不变更已有方向值。
    /// - `source`：调用方提供的轴标量；有限值夹取到 [-1, 1]，NaN 和正负无穷归零。
    ///
    /// # 返回
    /// Full 返回 [-1, 1] 内的有符号值；Positive 与 Negative 返回各自侧的 [0, 1] 幅值。所有零结果为正零。
    ///
    /// # 使用时机
    /// 收到当前值或转换前后值后投影；是否已应用死区和活区由采样生产者决定。
    ///
    /// # 约束
    /// 没有额外死区、增益、二维合成或设备查询。
    ///
    /// # 生命周期
    /// 纯数值计算，不保留帧或跨次历史，返回 f32 按值拥有。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection`]、[`normalized_axis_value`]、[`InputAxisBinding::value`]。
    pub fn value(self, source: f32) -> f32 {
        let value = if source.is_finite() {
            source.clamp(-1.0, 1.0)
        } else {
            0.0
        };

        match self {
            Self::Full => normalized_axis_value(value),
            Self::Positive => normalized_axis_value(value.max(0.0)),
            Self::Negative => normalized_axis_value((-value).max(0.0)),
        }
    }
}

/// 指定单台手柄的一根轴作为动作来源；求值先按设备和轴匹配帧样本，再按 direction 保留整轴值或折为单侧幅值。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 把物理轴来源的设备、轴标识和方向选择组合成可复制绑定值。
///
/// # 使用时机
/// 用于命名动作的轴来源列表；消费者先按设备与轴查找源标量，再调用 value。
///
/// # 约束
/// 字段公开，可直接构造或修改；缺省反序列化 direction 为 Full。构造器不分配设备号，也不验证连接状态或 Other 的编号。
/// 没有缩放系数或目标二维分量；派生 Eq 比较完整三元组。
///
/// # 生命周期
/// 按值保存三个可复制标识/策略，不拥有设备句柄、采样缓冲区或帧引用。
///
/// # 参考
/// 参见 [`super::GamepadId`]、[`super::GamepadAxis`]、[`InputAxisDirection`]、[`InputAxisBinding::value`]。
pub struct InputAxisBinding {
    /// 确定读取轴样本的设备身份。
    ///
    /// # 约束
    /// 与 axis 共同匹配物理来源；公开字段可任意赋值，本层不保证编号分配、全局唯一或设备在线。
    ///
    /// # 生命周期
    /// 仅保存可复制标识，不取得设备所有权。
    ///
    /// # 参考
    /// 参见 [`super::GamepadId`]、[`InputAxisBinding::value`]。
    pub gamepad: GamepadId,
    /// 确定该设备上要读取的物理轴分量。
    ///
    /// # 约束
    /// LeftStickX/Y 等名称区分物理标量；Other(u16) 按现有枚举表示保存，不能解释为目标动作的二维分量或平台编号翻译。
    ///
    /// # 生命周期
    /// 可复制枚举值，实际源采样由调用方提供。
    ///
    /// # 参考
    /// 参见 [`super::GamepadAxis`]、[`InputAxisBinding::value`]。
    pub axis: GamepadAxis,
    #[serde(default)]
    /// 指定匹配到的物理轴标量采用哪一侧投影。
    ///
    /// # 约束
    /// 省略 Serde 字段时使用 Full；相同设备和轴但方向不同的绑定是不同三元身份，构造器保留它们。
    ///
    /// # 生命周期
    /// 可复制策略，不携带转换历史或设备资源。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection`]、[`InputAxisDirection::value`]。
    pub direction: InputAxisDirection,
}

/// 提供轴来源的显式方向构造与数值投影入口。
///
/// # 使用时机
/// 配置绑定或对已匹配源值求值时使用。
///
/// # 约束
/// 三个构造器只保存传入标识；value 只读取 direction，来源匹配由调用方完成。
///
/// # 生命周期
/// 构造返回拥有可复制字段的值，方法不建立设备或帧借用。
///
/// # 参考
/// 参见 [`InputAxisBinding::new`]、[`InputAxisBinding::positive`]、[`InputAxisBinding::negative`]、[`InputAxisBinding::value`]。
impl InputAxisBinding {
    /// 创建保留完整有符号轴值的来源绑定。
    ///
    /// # 参数
    /// - `gamepad`：按值复制要匹配的设备标识，不在此分配或验证。
    /// - `axis`：按值复制该设备的物理轴标识，包含 Other 表示。
    ///
    /// # 返回
    /// 返回 direction 为 Full 的 Self；其它两个字段与输入相同。
    ///
    /// # 使用时机
    /// 准备轴绑定列表时使用；const 构造也只建立值。
    ///
    /// # 约束
    /// 不查询设备、读取轴值或验证来源；具体幅值规则由方向 value 实现。
    ///
    /// # 生命周期
    /// 返回值不借用参数，未取得设备资源。
    ///
    /// # 参考
    /// 参见 [`InputAxisBinding`]、[`InputAxisDirection::value`]。
    pub const fn new(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Full,
        }
    }

    /// 创建只保留正侧幅值的来源绑定。
    ///
    /// # 参数
    /// - `gamepad`：按值复制要匹配的设备标识，不在此分配或验证。
    /// - `axis`：按值复制该设备的物理轴标识，包含 Other 表示。
    ///
    /// # 返回
    /// 返回 direction 为 Positive 的 Self；其它两个字段与输入相同。
    ///
    /// # 使用时机
    /// 准备轴绑定列表时使用；const 构造也只建立值。
    ///
    /// # 约束
    /// 不查询设备、读取轴值或验证来源；具体幅值规则由方向 value 实现。
    ///
    /// # 生命周期
    /// 返回值不借用参数，未取得设备资源。
    ///
    /// # 参考
    /// 参见 [`InputAxisBinding`]、[`InputAxisDirection::value`]。
    pub const fn positive(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Positive,
        }
    }

    /// 创建把负侧映射为非负幅值的来源绑定。
    ///
    /// # 参数
    /// - `gamepad`：按值复制要匹配的设备标识，不在此分配或验证。
    /// - `axis`：按值复制该设备的物理轴标识，包含 Other 表示。
    ///
    /// # 返回
    /// 返回 direction 为 Negative 的 Self；其它两个字段与输入相同。
    ///
    /// # 使用时机
    /// 准备轴绑定列表时使用；const 构造也只建立值。
    ///
    /// # 约束
    /// 不查询设备、读取轴值或验证来源；具体幅值规则由方向 value 实现。
    ///
    /// # 生命周期
    /// 返回值不借用参数，未取得设备资源。
    ///
    /// # 参考
    /// 参见 [`InputAxisBinding`]、[`InputAxisDirection::value`]。
    pub const fn negative(gamepad: GamepadId, axis: GamepadAxis) -> Self {
        Self {
            gamepad,
            axis,
            direction: InputAxisDirection::Negative,
        }
    }

    /// 把调用方提供的单个源标量按绑定方向投影。
    ///
    /// # 参数
    /// - `self`：按值提供绑定；仅使用 direction，gamepad 与 axis 不参与此方法的计算。
    /// - `source`：应当已经按本绑定的设备/轴匹配的数值；本方法不会检查该对应关系。
    ///
    /// # 返回
    /// 返回方向 value 的 f32 结果：有限值先夹取，非有限值归零，零符号统一；Negative 返回负侧的非负幅值。
    ///
    /// # 使用时机
    /// 具体求值器查得当前状态或转换前后值之后使用。
    ///
    /// # 约束
    /// 不重新应用上游死区或活区缩放，没有 gain 或二维输出。
    ///
    /// # 生命周期
    /// 不保留状态或输入引用，返回标量按值拥有。
    ///
    /// # 参考
    /// 参见 [`InputAxisDirection::value`]、[`InputAxisBinding`]。
    pub fn value(self, source: f32) -> f32 {
        self.direction.value(source)
    }
}

/// 将一个命名动作关联到按钮和模拟轴；按钮作为组合键，含轴时还要求轴产生有效值。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 保存一个动作名称及该条绑定的按钮集合和轴来源列表。
///
/// # 使用时机
/// 构造配置值，再交给映射或消费者安装；动作声明与名称关联在映射和具体求值器中处理。
///
/// # 约束
/// 本类型没有目标二维分量、增益或上下文字段。构造器只对按钮完整身份和轴三元身份排序去重；公开字段、直接构造及派生 Serde 可绕过此整理。
/// action 必须由反序列化输入提供；buttons 缺省为空，axes 缺省为空且序列化时省略空列表。空名称不被拒绝。
/// 当前运行时的具体求值器对同条按钮采用全按住条件；有轴时按钮门控轴值，轴按最大绝对值选标量，同动作多条绑定是候选路径。这些是具体实现行为，不能当作 trait 额外保证。
/// 两个输入列表可同时为空；bind 入口会拒绝这种值，公开映射字段或反序列化可绕过该过滤。
///
/// # 生命周期
/// 拥有 action 字符串和两个 Vec；克隆形成拥有数据的副本，返回值不借用构造器的迭代器。
///
/// # 参考
/// 参见 [`InputBinding::buttons_and_axes`]、[`InputBinding::is_empty`]、[`super::InputActionMap::bind`]、[`super::InputActionManager`]。
pub struct InputBinding {
    /// 给出此绑定关联的精确动作名称。
    ///
    /// # 约束
    /// 原样拥有字符串，不自动注册动作、补齐上下文或检查名称非空；当前代际按已声明动作的名称寻找候选绑定。
    ///
    /// # 生命周期
    /// 随绑定拥有，修改已返回配置副本须重新安装才影响具体服务。
    ///
    /// # 参考
    /// 参见 [`super::InputActionMap`]、[`InputBinding::is_empty`]。
    pub action: String,
    #[serde(default)]
    /// 保存该条绑定要求的完整按钮身份列表。
    ///
    /// # 约束
    /// 构造器按枚举 Ord 排序并按完整 Eq 去重；KeyCode 与 Key 字符串仍是不同身份。公开字段和 Serde 不自动整理。
    /// 当前具体求值器要求这些按钮全部按住，任一按钮已消费会跳过整条绑定；这不是字段类型自行执行的匹配。
    ///
    /// # 生命周期
    /// Vec 拥有元素；Key variant 的字符串也随元素拥有，列表可为空。
    ///
    /// # 参考
    /// 参见 [`super::InputButton`]、[`InputBinding::chord`]。
    pub buttons: Vec<InputButton>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// 保存该条绑定的物理轴来源和方向选择。
    ///
    /// # 约束
    /// 构造器按设备、轴、方向排序，只有完整三元组重复才去重；同物理轴的不同方向保留。缺省为空，空列表在序列化时省略。
    /// 当前具体求值器选择最大绝对值标量，不把列表当作二维向量或相加；轴消费按物理设备/轴过滤，同来源各方向都会受影响。
    ///
    /// # 生命周期
    /// Vec 拥有可复制绑定值，不持有实时采样；公开写入和 Serde 可保留任意顺序和重复项。
    ///
    /// # 参考
    /// 参见 [`InputAxisBinding`]、[`InputBinding::axes`]、[`InputBinding::buttons_and_axes`]。
    pub axes: Vec<InputAxisBinding>,
}

#[path = "input_binding/binding_ctor.rs"]
mod binding_ctor;

/// 把数值零统一成正零，其余数值原样返回。
///
/// # 参数
/// - `value`：待规范化的 f32；现有调用方已先处理非有限值并完成单位区间夹取。
///
/// # 返回
/// value 与零相等时返回正零，因此 -0.0 也变为 +0.0；其它值原样返回，本助手自身不会消除 NaN 或无穷。
///
/// # 使用时机
/// 方向 value 的三个分支完成投影后调用。
///
/// # 约束
/// 只规范零的符号；有限性和范围保证来自上层调用路径，不能单独依赖此私有助手夹取或验证。
///
/// # 生命周期
/// 纯按值转换，无持久状态或资源。
///
/// # 参考
/// 参见 [`InputAxisDirection::value`]。
fn normalized_axis_value(value: f32) -> f32 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

/// 在测试构建中挂载绑定规范化对照、源码守卫和默认忽略的计时样例。
///
/// # 使用时机
/// 仅测试构建包含路径指定的私有子模块，常规生产模块不含其函数。
///
/// # 约束
/// 固定排序比较不覆盖所有绑定求值；源码守卫读取真实文本，分隔符匹配边界须独立核查，计时样例不提供未运行的性能结论。
///
/// # 生命周期
/// 测试局部值只在对应测试作用域存续；不建立生产注册或设备生命周期。
///
/// # 参考
/// 参见 [`InputBinding::buttons_and_axes`]、[`InputAxisDirection::value`]。
#[cfg(test)]
#[path = "tests/input_binding.rs"]
mod tests;
