//! 定义动作配置中的命名作用域及其排序优先级、配置启用位。
//!
//! [`InputActionContext`] 是自有配置值，由 [`super::InputActionMap::add_context`] 按名称注册并维护次序。
//! 当前默认编译器保存首个显式同名项的 enabled，帧求值再结合调用方活动列表；priority不决定动作排他消费。
//! 构造或设置字段不注册名称、重排已有映射或刷新编译代际；配置变更须显式安装。
//! 参见 [`InputActionContext::new`]、[`InputActionContext::disabled`] 与 [`super::InputAction::context`]。
use serde::{Deserialize, Serialize};

/// 为一组动作提供可启停的命名作用域；优先级只维护配置顺序，求值激活还取决于 `enabled` 和调用方上下文。
///
/// 为逻辑动作提供一个可排序、可配置启停的命名作用域声明。
///
/// # 名称与默认配置
/// [`Self::id`] 是精确字符串键，与动作的可选上下文名称匹配。构造不校验空值、大小写、命名格式或全局注册。
/// [`Self::new`] 采用 priority为0、enabled为true；[`Self::disabled`] 采用同样优先级但enabled为false。
/// 映射注册以id查重，同名添加保留旧项；整个声明的派生相等比较还包含priority和enabled。
///
/// # 排序与激活边界
/// [`super::InputActionMap::add_context`] 按priority降序、id升序维护配置次序；enabled不进入比较。
/// 当前默认编译器读取显式同名首项的enabled；只被动作引用的名称随后以true补入。
/// 默认帧求值对命名动作结合配置enabled和本次活动列表：空列表表示全部启用名称，非空列表仅选择已知活动名称。
/// 配置禁用仍不能被活动列表启用；动作context为None时不参与该过滤。priority不授予当前默认求值器输入抢占或排他消费规则。
///
/// # 所有权与生命周期
/// 实例拥有id字符串和两个标量，Clone复制所拥有的值。设置字段只改变返回的配置值，不更新已注册项、映射次序或编译代际。
/// 公开字段允许直接改写；改变映射中的priority后由调用方维护预期次序，启用位及名称改变后须重新安装配置供默认求值器编译。
/// 声明不保存本帧输入、服务句柄或临时活动状态，保存副本不能证明当前有动作激活。
///
/// # 参考
/// 参见 [`Self::with_priority`]、[`Self::with_enabled`]、[`super::InputActionMap::context_enabled`] 与 [`super::InputActionManager`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionContext {
    /// 上下文注册与动作归属匹配所用的自有名称，也作为同优先级排序的升序键。
    ///
    /// 精确字符串比较保留大小写、空白和空字符串；同名注册不比较其他字段。
    /// 名称不表示本帧已激活，只有动作引用的名称也可在默认编译时作为隐式启用上下文出现。
    ///
    /// # 所有权与生命周期
    /// 字段拥有String；重命名后不会同步改动作context或已有编译槽号，调用方需更新关联配置并重新安装。
    ///
    /// # 参考
    /// 参见 [`super::InputAction::context`]、[`super::InputActionMap::has_context`] 与 [`super::InputActionMap::add_context`]。
    pub id: String,
    /// 用于配置上下文次序的有符号整数，数值较大者排在前面。
    ///
    /// 0是两个构造入口的默认值，负值与全部i32值都可保存；比较直接调用cmp，不以相减计算键。
    /// 同优先级再按id升序，enabled不参与排序。当前默认编译及帧求值不据此决定排他输入消费。
    ///
    /// # 所有权与生命周期
    /// 标量随声明复制。改变独立值不会重排映射中已保存的项；改变公开映射内容后由调用方保持预期次序并安装配置。
    ///
    /// # 参考
    /// 参见 [`Self::with_priority`] 与 [`super::InputActionMap::add_context`]。
    pub priority: i32,
    /// 上下文的配置启用位，与调用方每次求值提供的活动名称列表分别保存。
    ///
    /// 当前默认编译保存显式同名首项；命名动作须配置启用且本次选中，空活动列表视为选中全部启用上下文。
    /// false不会因本次活动列表包含名称而变true；位不影响配置优先级或id排序。
    ///
    /// # 所有权与生命周期
    /// 布尔值属于声明配置，修改映射后需重新安装才能改变当前默认编译代际；已有动作状态继续拥有原数据。
    ///
    /// # 参考
    /// 参见 [`Self::disabled`]、[`Self::with_enabled`]、[`super::InputActionMap::context_enabled`] 与 [`super::InputActionManager`]。
    pub enabled: bool,
}

impl InputActionContext {
    /// 创建默认启用、优先级为0的命名上下文声明。
    ///
    /// # 参数
    /// - `id`：通过Into<String>转换的自有名称，转换后原样保存，不验证格式或是否已注册。
    ///
    /// # 返回
    /// 返回拥有id、priority=0、enabled=true的 [`Self`]；结果只是一份配置值。
    ///
    /// # 调用时机与生命周期
    /// 在注册映射前构造，可继续设置优先级或启用位。结果不借用原名称，不注册服务或激活动作；加入同名映射时旧项仍保留。
    ///
    /// # 参考
    /// 参见 [`Self::with_priority`]、[`Self::with_enabled`] 与 [`super::InputActionMap::add_context`]。
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            priority: 0,
            enabled: true,
        }
    }

    /// 创建默认禁用、优先级为0的命名上下文声明。
    ///
    /// # 参数
    /// - `id`：转为自有String的名称，允许任意字符串；构造不做注册查重。
    ///
    /// # 返回
    /// 返回 [`Self`]，priority为0、enabled为false，名称采用转换后的值。
    ///
    /// # 调用时机与生命周期
    /// 用于配置禁用声明，之后可链式设置优先级或启用位。函数不关闭已有服务或取消运行中的工作；结果不借用名称，安装配置后当前默认求值才读取禁用位。
    ///
    /// # 参考
    /// 参见 [`Self::new`]、[`Self::with_enabled`] 与 [`super::InputActionMap::context_enabled`]。
    pub fn disabled(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            priority: 0,
            enabled: false,
        }
    }

    /// 消费上下文声明并替换其配置排序优先级。
    ///
    /// # 参数
    /// - `self`：要更新的自有声明。
    /// - `priority`：任意i32值，较大值在统一配置次序中排在前面。
    ///
    /// # 返回
    /// 返回拥有新priority的 [`Self`]，id与enabled继续保持。
    ///
    /// # 调用时机与生命周期
    /// 适合在注册前构建；不会重排任何已保存的映射列表。修改已注册声明需写回配置并维护次序，同名add_context不能覆盖旧项；返回值不保留借用，也不激活动作。
    ///
    /// # 参考
    /// 参见 [`Self::priority`] 与 [`super::InputActionMap::add_context`]。
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// 消费上下文声明并设置其配置启用位。
    ///
    /// # 参数
    /// - `self`：要更新启用配置的自有声明。
    /// - `enabled`：新的布尔位，true允许默认求值再按本次活动列表选择，false保持配置禁用。
    ///
    /// # 返回
    /// 返回拥有新enabled的 [`Self`]，id与priority继续保持。
    ///
    /// # 调用时机与生命周期
    /// 构建或编辑声明时使用；不会变更本次活动列表、重排配置或直接更新已编译代际。结果拥有字段且不保留借用，安装后的配置要显式替换；同名注册保留旧项。
    ///
    /// # 参考
    /// 参见 [`Self::enabled`]、[`Self::disabled`] 与 [`super::InputActionMap::context_enabled`]。
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}
