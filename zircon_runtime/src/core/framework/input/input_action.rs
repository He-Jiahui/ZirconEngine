//! 定义动作映射中按名称关联绑定和上下文的逻辑动作声明值。
//!
//! [`InputAction`] 拥有 id 及可选上下文、显示名称；[`super::InputActionMap::add_action`] 以 id 查重。
//! 当前默认编译器按 id 关联绑定、按 context 选择上下文槽，输出按 id 建立状态名称。
//! 构造或改变声明只产生配置值，安装与求值由 [`super::InputActionManager`] 及其实现负责。
//! 值的克隆独立拥有字符串；修改配置后需重新安装，已有编译代际和状态不会自动刷新。
//! 参见 [`InputAction::new`]、[`InputAction::with_context`] 与 [`super::InputBinding`]。
use serde::{Deserialize, Serialize};

/// 动作映射中的稳定逻辑标识；绑定按 `id` 查找，`display_name` 只供界面显示。
///
/// 保存一个约定稳定的逻辑动作名称，以及可选的上下文归属和显示元数据。
///
/// # 名称与值比较
/// [`Self::id`] 是映射注册和绑定匹配的精确字符串键。空值、大小写与空白均原样保存，类型没有命名规范或全局注册验证。
/// [`super::InputActionMap::add_action`] 查重只读取 id；派生 PartialEq/Eq 比较 id、context、display_name 三个字段。
/// 因此注册查重与整个声明值的相等比较各有用途，同名添加不会更新原声明的其他字段。
///
/// # 使用时机与上下文
/// [`Self::new`] 初始没有命名上下文或显示名称；配置可以在注册前用消费式构建方法补充这些字段。
/// 当前默认求值器让 context 为 None 的动作直接参与；Some 名称先结合配置启用位，再结合本次活动列表。
/// 只在动作上引用而未显式声明的上下文默认启用，但仍受非空活动列表选择影响。
/// 显示名称供界面使用，当前默认绑定编译和状态投影读取 id 与 context，不采用显示名称作为查找键。
///
/// # 所有权与生命周期
/// 实例拥有三个字段的字符串值，Clone复制所拥有数据；构造和设置字段不会注册上下文、修改绑定名称或刷新已安装映射。
/// 公开字段允许直接改变声明。重命名时由调用方更新希望继续关联的绑定，并重新安装配置供默认求值器重建代际。
/// 声明不借用输入帧或服务；保存副本不代表动作当前已激活。
///
/// # 参考
/// 参见 [`Self::with_context`]、[`Self::with_display_name`]、[`super::InputActionMap`] 与 [`super::InputBinding::action`]。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputAction {
    /// 动作注册、绑定关联和输出状态查询共同使用的逻辑名称。
    ///
    /// 当前映射和默认编译器按完整字符串精确匹配，不解析点分段、修剪空白或验证非空。
    /// 整个声明的值相等比较还包含其他字段；注册查重只比较本字段。
    ///
    /// # 所有权与生命周期
    /// 字段拥有 String；重命名不会同步改变已有绑定的 action 名称，修改已安装配置需显式重新安装。
    ///
    /// # 参考
    /// 参见 [`super::InputActionMap::add_action`]、[`super::InputActionMap::has_action`] 与 [`super::InputBinding::action`]。
    pub id: String,
    /// 动作所属的可选命名上下文，保存配置名称而非本帧活动状态。
    ///
    /// None 在当前默认求值器中直接参与；Some 名称按精确字符串与显式上下文及本次活动列表关联，Some空字符串仍是命名值。
    /// 仅被动作引用的名称在默认编译时补入为启用；显式首项的 enabled 优先，配置禁用时本次活动列表不能将其启用。
    ///
    /// # 所有权与生命周期
    /// Option拥有其字符串；修改声明不会自动创建显式上下文，也不会刷新已有编译槽号，需重新安装配置。
    ///
    /// # 参考
    /// 参见 [`Self::with_context`]、[`super::InputActionContext`] 与 [`super::InputActionMap::context_enabled`]。
    pub context: Option<String>,
    /// 供界面使用的可选动作显示元数据，不承担当前默认求值路径的名称匹配。
    ///
    /// None 表示本声明未提供显示名称；Some可以保存任意字符串，类型不规定界面的回退显示方式。
    /// 字段参与派生的声明值相等比较，当前默认编译器和状态投影仍按 id 查找与输出。
    ///
    /// # 所有权与生命周期
    /// 字段拥有可选字符串；改变它只更新配置值，不重命名绑定或同步修改已返回的克隆。
    ///
    /// # 参考
    /// 参见 [`Self::with_display_name`]、[`Self::id`] 与 [`super::InputActionMap::add_action`]。
    pub display_name: Option<String>,
}

impl InputAction {
    /// 以给定逻辑名称创建没有上下文和显示名称的动作声明。
    ///
    /// # 参数
    /// - `id`：通过 Into<String> 转为自有名称；输入转换由其实现完成，不验证命名规范或注册状态。
    ///
    /// # 返回
    /// 返回拥有 id 的 [`Self`]，context 与 display_name 均为 None；返回值仅证明配置声明已构造。
    ///
    /// # 调用时机与生命周期
    /// 通常在 [`super::InputActionMap::add_action`] 之前调用。结果不借用传入名称，不生成绑定或动作状态；调用方可继续设置字段后安装映射。
    ///
    /// # 参考
    /// 参见 [`Self::with_context`]、[`Self::with_display_name`] 与 [`super::InputActionMap::add_action`]。
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            context: None,
            display_name: None,
        }
    }

    /// 消费动作声明并设置其命名上下文，供配置构建或归属变更。
    ///
    /// # 参数
    /// - `self`：将被更新的自有声明。
    /// - `context`：转为自有 String 后保存为 Some 的上下文名称，任意字符串原样使用。
    ///
    /// # 返回
    /// 返回 [`Self`]，id 和显示名称继续保留，旧上下文值被替换。
    ///
    /// # 调用时机与生命周期
    /// 构建声明时可链式调用；此方法不添加显式上下文或验证名称。返回值拥有字段，修改已经注册的动作时须把新值写回配置并重新安装，同名 add_action 会保留旧声明。
    ///
    /// # 参考
    /// 参见 [`Self::context`]、[`super::InputActionContext`] 与 [`super::InputActionMap::add_action`]。
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// 消费动作声明并保存一个界面显示名称。
    ///
    /// # 参数
    /// - `self`：要更新元数据的自有声明。
    /// - `display_name`：通过 Into<String> 转为自有显示文本，允许空字符串。
    ///
    /// # 返回
    /// 返回 [`Self`]，id 与上下文继续保留，display_name 为新文本的 Some 值。
    ///
    /// # 调用时机与生命周期
    /// 用于配置构建或元数据编辑；不规定界面回退、渲染或本地化规则。结果拥有新文本且不借用参数，不改绑定查找键或已保存克隆；安装后的配置需显式更新。
    ///
    /// # 参考
    /// 参见 [`Self::display_name`]、[`Self::id`] 与 [`super::InputActionMap::add_action`]。
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}
