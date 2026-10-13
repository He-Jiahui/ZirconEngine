//! 声明命名动作、上下文与物理绑定的自有配置，并提供注册、删除绑定和借用查询入口。
//!
//! [`InputActionMap`] 保存可序列化的值；[`super::InputActionManager`] 接收映射，由实现负责求值视图一致性。
//! 启用 `native-host` 时提供的当前具体求值器 `InputActionEvaluator` 在构造或替换映射时编译代际，服务默认实现返回映射克隆供调用方编辑。
//! 编辑完成后需显式安装新映射；独立克隆和此前返回的状态不会被同步改写。
//! 上下文优先级维护配置次序，默认求值还结合启用位和本次活动列表。名称均按精确字符串比较。
//! 公开向量仍可直接构造；反序列化会过滤无按钮且无轴来源的空绑定。
//! 参见 [`InputActionMap::add_context`]、[`InputActionMap::bind`] 与 [`InputActionMap::bindings_for_action`]。
use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use super::{InputAction, InputActionContext, InputBinding};

/// 声明动作、上下文和物理绑定的可序列化配置；求值器会编译它，修改后须重新设置映射。
/// 公共字段允许外部构造；新增上下文通过 `add_context` 维护优先级顺序，同名添加保留原项。
///
/// 拥有一份动作声明、命名上下文和物理绑定配置，供动作服务安装与求值器编译。
///
/// # 构造与名称身份
/// [`Self::new`] 和派生 `Default` 都创建三个空向量。动作与上下文各自以 id 精确匹配，大小写、空白与空字符串原样参与比较。
/// [`Self::add_action`] 与 [`Self::add_context`] 只在对应向量没有同名项时加入；同名添加保留原内容。
/// 上下文按优先级降序、id 升序排列；重复上下文的提前返回不修复原有未排序列表，也不更新启用位。
/// 公开字段允许直接构造未整理值；反序列化保留动作/上下文次序，仅过滤无来源的空绑定。缺失 contexts 使用默认值。
///
/// # 顺序与求值边界
/// 动作与绑定保留存储顺序，绑定可重复，也可引用尚未声明的动作。
/// 当前默认编译器按动作声明顺序建立记录，并为每个 id 保留对应绑定的源顺序；未声明动作的绑定不进入动作范围。
/// 外部直接配置的重复动作声明仍各有编译记录，可重复使用同一组绑定；配置值自身没有全局唯一性验证。
/// 未声明上下文的启用查询默认返回 true；默认求值器还检查本次活动列表，None 上下文的动作直接参与。
/// 优先级只用于本配置的排序，不赋予默认求值器排他输入消费语义。
///
/// # 所有权与生命周期
/// 实例拥有向量及其中的字符串、声明和绑定值；Clone复制配置数据，不共享可变配置视图。
/// 映射不保存物理帧状态。当前默认求值器拥有安装后的映射和编译代际；替换时重建代际并清空工作区的逻辑内容。
/// 工作区保留可复用容量，已返回的映射与动作状态继续拥有自己的数据；对编辑副本的修改需显式重新安装。
/// 上述编译和互斥锁策略属于当前默认实现，服务接口本身只提供读取、替换与求值入口。
///
/// # 参考
/// 参见 [`super::InputAction`]、[`super::InputActionContext`]、[`super::InputBinding`]、[`super::InputActionManager`] 与 [`super::InputActionManager::set_action_map`]。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "RawInputActionMap")]
pub struct InputActionMap {
    /// 拥有显式声明的命名上下文及其优先级和启用位。
    ///
    /// [`Self::add_context`] 对新名称维护优先级降序、id 升序；同名添加保留首项并提前返回。
    /// 公开构造和反序列化允许未排序或同名项；[`Self::context_enabled`] 从存储顺序中读取首个匹配项。
    /// 字段的 serde(default) 使缺失 contexts 时采用空向量，空向量仍允许未声明名称按默认启用规则查询。
    ///
    /// # 所有权与生命周期
    /// 向量拥有 [`super::InputActionContext`] 值；修改配置不会刷新已编译代际，安装新映射后才进入后续默认求值。
    ///
    /// # 参考
    /// 参见 [`Self::add_context`]、[`Self::has_context`] 与 [`Self::context_enabled`]。
    #[serde(default)]
    pub contexts: Vec<InputActionContext>,
    /// 按声明顺序拥有逻辑动作，绑定依动作 id 匹配。
    ///
    /// [`Self::add_action`] 追加尚未出现的 id；直接构造或反序列化可保留重复声明，且不自动同步绑定名称。
    /// 当前默认编译器逐条保留声明及其上下文槽位；没有命名上下文的动作不受活动列表限制。
    ///
    /// # 所有权与生命周期
    /// 向量拥有 [`super::InputAction`] 及其字符串；重命名或改变上下文后需重新安装配置以重建默认求值视图。
    ///
    /// # 参考
    /// 参见 [`Self::add_action`]、[`Self::has_action`] 与 [`super::InputAction::with_context`]。
    pub actions: Vec<InputAction>,
    /// 按存储顺序拥有动作名称到物理按钮及轴的绑定条目。
    ///
    /// [`Self::bind`] 仅排除按钮和轴都为空的条目，其余条目原样追加，允许重复或尚未声明的动作名称。
    /// 直接写入仍可绕过该过滤；反序列化会丢弃空条目，求值器也会忽略直接构造的空绑定。
    /// 当前默认编译器保留每个动作匹配条目的源顺序；等幅值合并保留先到的值，因此顺序可影响模拟输出的符号。
    ///
    /// # 所有权与生命周期
    /// 向量拥有 [`super::InputBinding`] 值，查询迭代器借用它；修改后需重新安装映射，先前状态不被改写。
    ///
    /// # 参考
    /// 参见 [`Self::bind`]、[`Self::clear_bindings`]、[`Self::bindings_for_action`] 与 [`super::InputActionManager`]。
    pub bindings: Vec<InputBinding>,
}

#[derive(Deserialize)]
struct RawInputActionMap {
    #[serde(default)]
    contexts: Vec<InputActionContext>,
    actions: Vec<InputAction>,
    bindings: Vec<InputBinding>,
}

impl From<RawInputActionMap> for InputActionMap {
    fn from(raw: RawInputActionMap) -> Self {
        Self {
            contexts: raw.contexts,
            actions: raw.actions,
            bindings: raw
                .bindings
                .into_iter()
                .filter(|binding| !binding.is_empty())
                .collect(),
        }
    }
}

impl InputActionMap {
    /// 创建空的动作、上下文和绑定配置。
    ///
    /// # 返回
    /// 返回拥有三个空向量的 [`Self`]，等同派生 Default；没有配置校验结果。
    ///
    /// # 调用时机与生命周期
    /// 用于启动配置或增量构建之前。构造只创建自有配置值，不注册服务或编译求值代际。空表的未声明上下文查询仍默认启用。
    ///
    /// # 参考
    /// 参见 [`Self::add_action`]、[`Self::add_context`] 与 [`Self::context_enabled`]。
    pub fn new() -> Self {
        Self::default()
    }

    /// 消费映射并按名称注册一个动作，供链式构建配置。
    ///
    /// # 参数
    /// - `self`：要继续构建的映射，所有权进入调用。
    /// - `action`：自有动作值；id 与已有声明精确匹配时保留旧项。
    ///
    /// # 返回
    /// 返回拥有原配置及实际注册结果的 [`Self`]；同名时返回原有内容。
    ///
    /// # 调用时机与生命周期
    /// 委托 [`Self::add_action`]，不更新同名声明的上下文或显示名。传入的新值在同名时被丢弃；结果不借用来源，也不刷新已安装视图。
    ///
    /// # 参考
    /// 参见 [`Self::add_action`] 与 [`super::InputAction`]。
    pub fn with_action(mut self, action: InputAction) -> Self {
        self.add_action(action);
        self
    }

    /// 消费映射并注册一个命名上下文，供链式构建配置。
    ///
    /// # 参数
    /// - `self`：将接收上下文的自有映射。
    /// - `context`：包含名称、优先级和启用位的自有值。
    ///
    /// # 返回
    /// 返回 [`Self`]；新名称按 [`Self::add_context`] 规则插入，同名保留旧值。
    ///
    /// # 调用时机与生命周期
    /// 构建配置时使用。新名称可能触发已有上下文排序；同名路径提前返回，不修复未排序状态。返回值拥有配置，后续改变需显式安装。
    ///
    /// # 参考
    /// 参见 [`Self::add_context`] 与 [`super::InputActionContext`]。
    pub fn with_context(mut self, context: InputActionContext) -> Self {
        self.add_context(context);
        self
    }

    /// 消费映射并追加一个有物理来源的绑定，供链式构建配置。
    ///
    /// # 参数
    /// - `self`：将接收绑定的自有映射。
    /// - `binding`：拥有动作名称、按钮和轴的条目，空来源条目不加入。
    ///
    /// # 返回
    /// 返回 [`Self`]；委托 [`Self::bind`] 后保留所有已有绑定及实际追加结果。
    ///
    /// # 调用时机与生命周期
    /// 构建或重绑定配置时使用。不会校验动作是否声明、去重条目或额外规范化来源；返回配置不借用输入，尚需安装才能改变默认求值。
    ///
    /// # 参考
    /// 参见 [`Self::bind`] 与 [`super::InputBinding::is_empty`]。
    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bind(binding);
        self
    }

    /// 按优先级和标识稳定插入；读取外部构造的未排序上下文时先修正顺序。
    ///
    /// 在尚无同名声明时，按优先级降序、id 升序插入一个上下文。
    ///
    /// # 参数
    /// - `self`：独占借用的配置，可包含外部直接构造的未排序上下文。
    /// - `context`：待加入的自有值；名称用于精确查重，优先级用于排序。
    ///
    /// # 返回
    /// 返回同一映射的 `&mut Self`，可继续构建。已有名称时所有向量保持原样，新值被丢弃。
    ///
    /// # 调用时机与生命周期
    /// 先查重，再检查排序；仅新名称路径会稳定排序已有项并寻找插入点。比较不读取 enabled；不清除已存在的重复项。可变借用与返回引用绑定 self，新增字符串由映射拥有，未安装配置时不改变求值器。
    ///
    /// # 参考
    /// 参见 [`Self::has_context`]、[`super::InputActionContext`]、[`contexts_are_sorted`] 与 [`context_order`]。
    pub fn add_context(&mut self, context: InputActionContext) -> &mut Self {
        if self.has_context(&context.id) {
            return self;
        }
        if !contexts_are_sorted(&self.contexts) {
            self.contexts.sort_by(context_order);
        }
        let insertion_index = self
            .contexts
            .partition_point(|candidate| context_order(candidate, &context).is_lt());
        self.contexts.insert(insertion_index, context);
        self
    }

    /// 将尚未出现的动作 id 追加到声明列表，保留声明顺序。
    ///
    /// # 参数
    /// - `self`：接收声明的可变映射。
    /// - `action`：自有动作值；查重只读取 id，不比较上下文或显示名。
    ///
    /// # 返回
    /// 返回同一映射的 `&mut Self`。同名时保留旧声明，并丢弃传入值。
    ///
    /// # 调用时机与生命周期
    /// 创建或编辑配置时使用；空字符串也按普通名称比较。不会创建上下文或同步绑定，也不会修复外部已存在的重复动作。结果引用借用 self，新增值由向量拥有，默认求值视图需重新安装。
    ///
    /// # 参考
    /// 参见 [`Self::has_action`]、[`Self::with_action`] 与 [`super::InputAction`]。
    pub fn add_action(&mut self, action: InputAction) -> &mut Self {
        if !self.has_action(&action.id) {
            self.actions.push(action);
        }
        self
    }

    /// 把至少包含按钮或轴的条目追加到绑定列表。
    ///
    /// # 参数
    /// - `self`：接收条目的独占可变映射。
    /// - `binding`：自有绑定；两种来源都为空时被丢弃，否则原样保存。
    ///
    /// # 返回
    /// 返回同一映射的 `&mut Self`，不报告是否追加，也不返回名称验证结果。
    ///
    /// # 调用时机与生命周期
    /// 构建或重绑定时使用；保留存储顺序，允许重复绑定和未知动作名。公开字段可绕过空值过滤；反序列化单独过滤无按钮且无轴来源的绑定。返回引用借用 self，条目所有权由映射保存；默认求值器须显式安装新表。
    ///
    /// # 参考
    /// 参见 [`super::InputBinding::is_empty`]、[`Self::with_binding`] 与 [`Self::bindings_for_action`]。
    pub fn bind(&mut self, binding: InputBinding) -> &mut Self {
        if !binding.is_empty() {
            self.bindings.push(binding);
        }
        self
    }

    /// 移除所有动作名称精确匹配的绑定，保留其他条目的相对顺序。
    ///
    /// # 参数
    /// - `self`：要编辑绑定列表的可变映射。
    /// - `action`：借助 AsRef<str> 暂借的动作名；不要求已有动作声明。
    ///
    /// # 返回
    /// 返回同一映射的 `&mut Self`。没有匹配项时配置保持原样；动作和上下文声明继续保留。
    ///
    /// # 调用时机与生命周期
    /// 用于重新绑定前清除旧来源。匹配条目随移除释放所拥有的值，查询名只在调用中借用；返回引用绑定 self。该操作只改配置，默认求值器需再接收完整新表，既有状态值仍保留各自数据。
    ///
    /// # 参考
    /// 参见 [`Self::bind`]、[`Self::bindings_for_action`] 与 [`super::InputActionManager::set_action_map`]。
    pub fn clear_bindings(&mut self, action: impl AsRef<str>) -> &mut Self {
        let action = action.as_ref();
        self.bindings.retain(|binding| binding.action != action);
        self
    }

    /// 查询声明列表是否包含精确匹配的动作 id。
    ///
    /// # 参数
    /// - `self`：只读借用的配置。
    /// - `action`：AsRef<str> 提供的暂借名称，允许任意字符串。
    ///
    /// # 返回
    /// 至少一个声明匹配时返回 true，否则 false；结果不证明存在绑定或当前输入已激活动作。
    ///
    /// # 调用时机与生命周期
    /// 注册查重或配置查询时使用。扫描 actions 而不修改状态、复制查询名或分配声明；只读借用在调用结束后释放，返回自有布尔值。
    ///
    /// # 参考
    /// 参见 [`Self::add_action`] 与 [`super::InputAction::id`]。
    pub fn has_action(&self, action: impl AsRef<str>) -> bool {
        let action = action.as_ref();
        self.actions.iter().any(|candidate| candidate.id == action)
    }

    /// 查询显式上下文列表是否包含精确匹配的 id。
    ///
    /// # 参数
    /// - `self`：只读借用的配置。
    /// - `context`：由 AsRef<str> 暂借的上下文名称。
    ///
    /// # 返回
    /// 存在任一匹配声明时返回 true，即使它被禁用；没有显式声明返回 false。
    ///
    /// # 调用时机与生命周期
    /// 用于 [`Self::add_context`] 查重或调用方查询。结果只表示声明存在，启用规则需另查 [`Self::context_enabled`]。不修改配置或保留名称借用，返回自有布尔值。
    ///
    /// # 参考
    /// 参见 [`Self::add_context`]、[`Self::context_enabled`] 与 [`super::InputActionContext::id`]。
    pub fn has_context(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .any(|candidate| candidate.id == context)
    }

    /// 未显式声明的上下文默认可用，使只在动作上命名的旧配置仍能求值。
    ///
    /// 读取首个同名显式上下文的启用位，未声明名称默认返回启用。
    ///
    /// # 参数
    /// - `self`：只读借用的配置，可包含外部构造的重复名称。
    /// - `context`：由 AsRef<str> 暂借的精确上下文名称。
    ///
    /// # 返回
    /// 首个匹配项返回其 enabled；不存在则返回 true。返回 true 不能证明名称已经声明或本次活动列表包含它。
    ///
    /// # 调用时机与生命周期
    /// 支持只在动作上命名上下文的配置。方法不读取 priority 或输入帧；当前默认编译器采用同样的首项与隐式启用规则，再独立检查本次活动位。查询不修改或注册名称，返回值无借用。
    ///
    /// # 参考
    /// 参见 [`Self::has_context`]、[`super::InputAction::context`] 与 [`super::InputActionContext::enabled`]。
    pub fn context_enabled(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .find(|candidate| candidate.id == context)
            .map(|candidate| candidate.enabled)
            .unwrap_or(true)
    }

    /// 借用查询指定动作名称的全部绑定，按当前存储顺序惰性产生引用。
    ///
    /// # 参数
    /// - `self`：借用至返回迭代器使用结束的映射。
    /// - `action`：具有同一 'a 生命周期的查询名称切片，直接由过滤闭包持有引用。
    ///
    /// # 返回
    /// 返回 Item 为 `&'a InputBinding` 的迭代器；重复条目继续返回，不要求动作声明存在，也不跳过公开字段中的空条目。
    ///
    /// # 调用时机与生命周期
    /// 调用方在读取配置时使用。迭代器不复制名称或绑定，同时受映射和 action 的借用限制；持续使用迭代器时须维持两者有效，结束使用后借用可释放。查询不改配置或求值状态。
    ///
    /// # 参考
    /// 参见 [`Self::bindings`]、[`Self::bind`] 与 [`Self::clear_bindings`]。
    pub fn bindings_for_action<'a>(
        &'a self,
        action: &'a str,
    ) -> impl Iterator<Item = &'a InputBinding> + 'a {
        self.bindings
            .iter()
            .filter(move |binding| binding.action == action)
    }
}

/// 只读检查上下文列表是否已符合统一比较规则，供增量插入避免重复全排序。
///
/// # 参数
/// - `contexts`：调用期间借用的上下文切片，元素可以同名或同优先级。
///
/// # 返回
/// 相邻项均不逆序时返回 true；空或单项切片也为 true。比较相等允许通过，不证明名称唯一、配置启用或字段有效。
///
/// # 调用时机与生命周期
/// 由新名称插入路径在 partition_point 之前调用。仅检查次序，不修复列表；所有元素借用只在调用中存在，返回自有布尔值。
///
/// # 参考
/// 参见 [`context_order`] 与 [`InputActionMap::add_context`]。
fn contexts_are_sorted(contexts: &[InputActionContext]) -> bool {
    contexts
        .windows(2)
        .all(|pair| !context_order(&pair[0], &pair[1]).is_gt())
}

/// 为上下文配置提供优先级降序、名称升序的统一排序比较。
///
/// # 参数
/// - `left`：调用期间只读借用的左侧上下文。
/// - `right`：调用期间只读借用的右侧上下文。
///
/// # 返回
/// 返回 [`Ordering`]；较高 priority 排在前面，同优先级按 id 升序，同键返回 Equal。enabled 不参与比较。
///
/// # 调用时机与生命周期
/// 供完整排序、相邻检查和插入边界共用，保证这些操作解释同一种次序；测试基线亦复用本比较器。函数不改变值、激活动作或持有输入借用。
///
/// # 参考
/// 参见 [`contexts_are_sorted`]、[`InputActionMap::add_context`] 与 [`super::InputActionContext::priority`]。
fn context_order(left: &InputActionContext, right: &InputActionContext) -> Ordering {
    right
        .priority
        .cmp(&left.priority)
        .then(left.id.cmp(&right.id))
}

/// 仅在测试构建中挂载上下文插入回归、源码形态守卫和默认忽略的计时夹具。
///
/// 测试可以访问 [`InputActionMap`]、[`contexts_are_sorted`] 与 [`context_order`] 的私有实现。
/// 挂载不调用测试；计时夹具需显式请求被忽略用例，普通测试构建不会自动执行它。
/// 测试模块与生产配置值各有生命周期，静态源码守卫只检查指定字串。
/// 参见 [`InputActionMap::add_context`]。
#[cfg(test)]
#[path = "tests/input_action_map.rs"]
mod tests;
