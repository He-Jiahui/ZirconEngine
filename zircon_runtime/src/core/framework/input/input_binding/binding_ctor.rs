//! Constructors and normalization for [`super::InputBinding`].
use super::{InputAxisBinding, InputBinding, InputButton};

/// 提供拥有数据的按钮/轴构造、规范化和空输入检查。
///
/// # 使用时机
/// 配置构造阶段调用；输入求值和服务安装由外部消费者完成。
///
/// # 约束
/// 构造时整理本值，不修复已经直接构造或反序列化的值；名称和设备来源不验证。
///
/// # 生命周期
/// 字符串和列表移入返回值，不保留调用方集合的借用。
///
/// # 参考
/// 参见 [`InputBinding::chord`]、[`InputBinding::buttons_and_axes`]、[`InputBinding::is_empty`]。
impl super::InputBinding {
    /// 用一个完整按钮身份创建无轴绑定。
    ///
    /// # 参数
    /// - `action`：转换为拥有的 String，按原值保存，不验证非空或自动声明动作。
    /// - `button`：按值移入的按钮；Key/KeyCode 和设备按钮按各自完整身份保存。
    ///
    /// # 返回
    /// 返回含唯一按钮、空轴列表的 Self，整理规则与 chord 相同。
    ///
    /// # 使用时机
    /// 配置单按钮命名动作时使用。
    ///
    /// # 约束
    /// 不读取按钮状态，不翻译键名或注册动作。
    ///
    /// # 生命周期
    /// 返回值拥有 action 与按钮元素，未借用调用方状态。
    ///
    /// # 参考
    /// 参见 [`InputBinding::chord`]、[`super::InputButton`]。
    pub fn button(action: impl Into<String>, button: InputButton) -> Self {
        Self::chord(action, [button])
    }

    /// 收集并规范化一条按钮组合绑定，轴列表为空。
    ///
    /// # 参数
    /// - `action`：转换为拥有的 String，按原值保存，不验证非空或自动声明动作。
    /// - `buttons`：消费一次迭代器并收集拥有的按钮；按派生 Ord 排序，再按完整 Eq 去重。
    ///
    /// # 返回
    /// 返回拥有规范化按钮列表和空 axes 的 Self；空迭代器得到可表示的空绑定。
    ///
    /// # 使用时机
    /// 准备按钮组合配置时使用；当前具体求值器随后要求同条按钮全部按住。
    ///
    /// # 约束
    /// 只整理枚举身份，不把 KeyCode 与 Key 视为等价，也不排除空名称或空输入；bind 入口才拒绝两个列表都为空的值。
    ///
    /// # 生命周期
    /// 收集完成后不借用迭代器；String 和含字符串的按钮随返回值拥有。
    ///
    /// # 参考
    /// 参见 [`super::InputButton`]、[`InputBinding::buttons_and_axes`]、[`super::InputActionMap::bind`]。
    pub fn chord(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
    ) -> Self {
        let mut buttons = buttons.into_iter().collect::<Vec<_>>();
        buttons.sort_unstable();
        buttons.dedup();
        Self {
            action: action.into(),
            buttons,
            axes: Vec::new(),
        }
    }

    /// 用一条物理轴来源创建无按钮绑定。
    ///
    /// # 参数
    /// - `action`：转换为拥有的 String，按原值保存，不验证非空或自动声明动作。
    /// - `axis`：按值保存的来源/方向绑定，其标识不在此校验。
    ///
    /// # 返回
    /// 返回包含该唯一轴、空 buttons 的 Self。
    ///
    /// # 使用时机
    /// 配置单个连续标量动作来源时使用。
    ///
    /// # 约束
    /// 不设置缩放或二维分量，不读取样本；方向计算在已有源值时进行。
    ///
    /// # 生命周期
    /// 返回值拥有名称和列表，轴元素按值复制/移动，不借用设备。
    ///
    /// # 参考
    /// 参见 [`InputBinding::axes`]、[`InputAxisBinding`]。
    pub fn axis(action: impl Into<String>, axis: InputAxisBinding) -> Self {
        Self::axes(action, [axis])
    }

    /// 收集多个轴来源创建无按钮绑定。
    ///
    /// # 参数
    /// - `action`：转换为拥有的 String，按原值保存，不验证非空或自动声明动作。
    /// - `axes`：消费一次迭代器的轴绑定集合；按设备、轴、方向排序并去重完整三元身份。
    ///
    /// # 返回
    /// 返回规范化 axes 与空 buttons 的 Self；空集合保留为空绑定。
    ///
    /// # 使用时机
    /// 准备多个候选轴来源时使用。
    ///
    /// # 约束
    /// 相同设备/轴的不同方向仍保留；当前具体求值器选最大绝对值，列表本身不规定二维组合、相加或增益。
    ///
    /// # 生命周期
    /// 返回值拥有收集出的列表和名称，不借用迭代器或采样。
    ///
    /// # 参考
    /// 参见 [`InputBinding::buttons_and_axes`]、[`InputAxisBinding`]。
    pub fn axes(
        action: impl Into<String>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        Self::buttons_and_axes(action, std::iter::empty(), axes)
    }

    /// 构造时统一排序去重，使重复设备输入不会改变求值或序列化后的绑定身份。
    /// 同时收集按钮门控与轴来源，并规范化两份拥有的列表。
    ///
    /// # 参数
    /// - `action`：转换为拥有的 String，按原值保存，不验证非空或自动声明动作。
    /// - `buttons`：先收集并按完整按钮身份排序去重；不同按钮 variant 不互相翻译。
    /// - `axes`：随后收集，按 (gamepad, axis, direction) 排序，只有完整三元身份相等才去重。
    ///
    /// # 返回
    /// 返回拥有规范化列表的 Self；任一列表或两者均可为空，action 原值保留。
    ///
    /// # 使用时机
    /// 配置组合门控/连续输入时使用；当前具体求值器先检查全部按钮，再从未消费的轴选标量。
    ///
    /// # 约束
    /// 不同方向的同源轴保留；不验证设备、动作注册或名称，不设置死区、增益和二维分量。构造整理不是公开字段/Serde 的不变式。
    ///
    /// # 生命周期
    /// 两次收集均取得元素所有权，返回值不借用迭代器；方法不安装映射或保存帧。
    ///
    /// # 参考
    /// 参见 [`InputBinding::chord`]、[`InputAxisBinding`]、[`super::InputActionMap::bind`]。
    pub fn buttons_and_axes(
        action: impl Into<String>,
        buttons: impl IntoIterator<Item = InputButton>,
        axes: impl IntoIterator<Item = InputAxisBinding>,
    ) -> Self {
        let mut binding = Self::chord(action, buttons);
        let mut axes = axes.into_iter().collect::<Vec<_>>();
        axes.sort_unstable_by(|left, right| {
            left.gamepad
                .cmp(&right.gamepad)
                .then(left.axis.cmp(&right.axis))
                .then(left.direction.cmp(&right.direction))
        });
        axes.dedup();
        binding.axes = axes;
        binding
    }

    /// 检查两个物理输入列表是否同时为空。
    ///
    /// # 参数
    /// - `self`：只读借用当前绑定的两个 Vec。
    ///
    /// # 返回
    /// 仅在 buttons 与 axes 都为空时返回 true；action 的空字符串、未知名称和设备有效性不参与判断。
    ///
    /// # 使用时机
    /// 映射 bind 入口使用此检查过滤空绑定，也可独立检查直接构造或反序列化的值。
    ///
    /// # 约束
    /// 不是配置验证器；公开映射字段和反序列化安装路径不一定调用此过滤。
    ///
    /// # 生命周期
    /// 不修改或保留借用，返回 bool 按值拥有。
    ///
    /// # 参考
    /// 参见 [`super::InputActionMap::bind`]、[`InputBinding`]。
    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty() && self.axes.is_empty()
    }
}

