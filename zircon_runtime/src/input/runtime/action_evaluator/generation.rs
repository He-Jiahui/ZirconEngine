//! 为动作映射建立供求值器复用的配置查找代际。
//!
//! # 责任与入口
//! [`ActionEvaluationGeneration::from_action_map`] 编译上下文槽位、动作记录和源绑定下标。
//! [`super::InputActionEvaluator`] 在创建或替换映射时构建代际；帧路径读取它，另由工作区保存本帧输入和动作结果。
//!
//! # 配对约束与生命周期
//! 代际拥有上下文名称和整数索引，不借用源映射的字符串，也不保存输入帧。
//! 其中动作位置、绑定下标和范围只适用于生成它的同一 [`InputActionMap`]；读取路径不核验这种配对。
//! 映射内容改变后应重新构建；求值器替换映射时同时替换代际并重置工作区。
//!
//! # 参考
//! 参见 [`CompiledAction`]、[`ActionEvaluationGeneration`] 与 [`super::InputActionEvaluator::set_action_map`]。
use std::collections::BTreeMap;

use crate::input::InputActionMap;

/// 保存一个动作的源位置、上下文槽位和代际内的绑定范围。
///
/// # 使用约束与生命周期
/// `action_index` 指向同一 [`InputActionMap`] 的动作；范围指向所属代际的扁平绑定下标向量。
/// 值可 `Copy`，但复制不会复制源动作或绑定，也不会解除与原代际和映射的配对约束。
/// 派生默认值仅把索引与范围置零、上下文置为 `None`，不能证明记录属于某份已编译映射。
///
/// # 参考
/// 参见 [`ActionEvaluationGeneration`] 与 [`Self::binding_indices`]。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct CompiledAction {
    /// 该动作在源映射 `actions` 中的零基位置，构建后随代际固定。
    /// 输出投影按此位置直接索引同一映射；它不表示动作 id，也不校验映射是否已经替换。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`] 与 [`ActionEvaluationGeneration::actions()`]。
    pub(super) action_index: usize,
    /// 所属上下文在本代际并行表中的槽位；没有命名上下文的动作保存 `None`。
    /// `Some` 槽位随代际固定，求值时结合配置启用位与本帧活动位；`None` 不参与上下文过滤。
    ///
    /// # 参考
    /// 参见 [`ActionEvaluationGeneration::context_slot()`] 与 [`ActionEvaluationGeneration::context_enabled()`]。
    pub(super) context_slot: Option<usize>,
    /// 本动作在代际 `binding_indices` 向量中的起始偏移（包含），构建后固定。
    /// 偏移的单位是扁平向量元素，元素本身才是源 `bindings` 的下标；须与同记录的终点及所属代际配对。
    ///
    /// # 参考
    /// 参见 [`Self::binding_indices`] 与 [`ActionEvaluationGeneration`]。
    binding_start: usize,
    /// 本动作在代际扁平向量中的终止偏移（不包含），与起点组成半开范围。
    /// 生产构建保持起点不大于终点且终点不超过所属向量长度；该整数不能移用于另一代际。
    ///
    /// # 参考
    /// 参见 [`Self::binding_indices`] 与 [`ActionEvaluationGeneration`]。
    binding_end: usize,
}

impl CompiledAction {
    /// 借用本动作在所属代际中保存的源绑定下标。
    ///
    /// # 参数
    /// `generation` 必须是生成此记录的同一 [`ActionEvaluationGeneration`]，并继续与原动作映射配对。
    ///
    /// # 返回与失败
    /// 返回保存范围对应的只读切片；每个元素可索引同一 [`InputActionMap`] 的 `bindings`。
    /// 方法直接切片，不核对代际身份、范围或源映射；非法范围会 panic，另一代际的合法范围也可能返回错误数据。
    ///
    /// # 生命周期
    /// 返回借用的 `'a` 来自 `generation`，不来自记录自身；读取不修改或分配代际存储。
    ///
    /// # 参考
    /// 参见 [`ActionEvaluationGeneration::from_action_map`] 与 [`Self`]。
    pub(super) fn binding_indices<'a>(
        &self,
        generation: &'a ActionEvaluationGeneration,
    ) -> &'a [usize] {
        &generation.binding_indices[self.binding_start..self.binding_end]
    }
}

/// Immutable map-change-time lookup data for one persisted action map.
/// 拥有一次动作映射编译所得的动作、绑定和上下文查找数据。
///
/// # 入口与约束
/// [`Self::from_action_map`] 保留动作声明顺序、建立每个动作的绑定范围，并把上下文名称映射到并行启用位。
/// 动作与绑定索引只适用于原映射；公开配置字段可绕过辅助方法，编译不会验证重复动作或拒绝未知动作绑定。
///
/// # 生命周期
/// 求值器在创建和替换映射时构建它，帧路径只读；本帧活动上下文和动作结果另存工作区。
/// `Clone` 复制所拥有的集合，`Default` 产生空集合与 `false` 轴标志；两者均不自动编译或安装映射。
///
/// # 参考
/// 参见 [`CompiledAction`]、[`InputActionMap`] 与 [`super::InputActionEvaluator::set_action_map`]。
#[derive(Clone, Debug, Default)]
pub(super) struct ActionEvaluationGeneration {
    /// 按源 `actions` 声明顺序拥有编译记录，构建后只读。
    /// 向量位置也是工作区动作槽位；重复公开动作 id 仍各有记录，可重复使用同一组源绑定。
    ///
    /// # 参考
    /// 参见 [`Self::actions()`]、[`CompiledAction`] 与 [`InputActionMap`]。
    actions: Vec<CompiledAction>,
    /// 拥有按动作拼接的源 `bindings` 下标；构建后由每个动作的半开范围借用。
    /// 每组下标保留源绑定顺序；未知动作的绑定不进入动作范围，重复动作 id 可使同一组下标重复拼接。
    /// 元素须与原映射配对，向量长度不必等于源绑定条目数。
    ///
    /// # 参考
    /// 参见 [`CompiledAction::binding_indices`] 与 [`Self::from_action_map`]。
    binding_indices: Vec<usize>,
    /// 拥有上下文名称到并行启用位槽号的映射，构建后固定。
    /// 先按源显式上下文顺序分配，再补入动作引用的名称；同名首次出现决定槽位及启用位，树的键排序不决定槽号。
    ///
    /// # 参考
    /// 参见 [`insert_context_slot`] 与 [`Self::context_slot()`]。
    context_slots: BTreeMap<String, usize>,
    /// 按本代际上下文槽位拥有配置启用位；首次显式声明的值优先，只有动作引用的名称默认启用。
    /// 构建后不会随本帧活动列表改变；求值还需结合工作区记录的活动位。
    ///
    /// # 参考
    /// 参见 [`Self::context_enabled()`] 与 [`Self::context_slots`]。
    context_enabled: Vec<bool>,
    /// 源 `bindings` 是否有任意非空 `axes`，在构建时计算并固定。
    /// 统计包括未知动作或禁用上下文的绑定；此位只门控工作区轴索引准备，不表示当前输入或动作激活。
    ///
    /// # 参考
    /// 参见 [`Self::has_axis_bindings()`] 与 [`InputActionMap`]。
    has_axis_bindings: bool,
    /// 仅测试构建拥有的源绑定原始条目数，随代际构建固定。
    /// 计数包括未知动作绑定，不等同扁平范围长度、帧访问次数或命中次数；生产构建不含该字段。
    ///
    /// # 参考
    /// 参见 [`Self::candidate_count`] 与 [`Self::from_action_map`]。
    #[cfg(test)]
    source_binding_count: usize,
}

impl ActionEvaluationGeneration {
    // 仅在动作表创建或替换时构建查找代次；临时绑定索引借用名称，生成结果保持动作声明顺序并供帧路径只读。
    /// 把完整动作映射编译为可供帧求值复用的配置查找代际。
    ///
    /// # 参数
    /// `action_map` 提供上下文、动作和绑定；调用期间只读借用，动作与绑定的位置按原向量解释。
    ///
    /// # 返回与约束
    /// 返回拥有上下文键、动作记录及绑定下标的 [`Self`]，不返回校验结果。
    /// 显式同名上下文首次出现决定槽位与启用位；仅在动作中命名的上下文随后以 `true` 补入。
    /// 动作保持声明顺序，各动作按 id 取得保持源顺序的绑定；重复动作 id 会重复拼接同一组下标。
    /// 未知动作绑定不进入任何动作范围，但仍计入测试源条目数，并可使轴绑定标志为 `true`。
    /// 函数不去重动作、拒绝空绑定或核验外部直接构造的配置。
    ///
    /// # 使用时机与生命周期
    /// 求值器在创建或替换映射时调用，后续读取须继续配合同一映射；配置变更后应重新编译。
    /// 临时 `BTreeMap<&str, Vec<usize>>` 只在构建期间借用绑定名称；返回代际拥有上下文 `String` 和整数索引，不保留该借用。
    ///
    /// # 参考
    /// 参见 [`InputActionMap`]、[`CompiledAction`]、[`insert_context_slot`] 与 [`super::InputActionEvaluator::set_action_map`]。
    pub(super) fn from_action_map(action_map: &InputActionMap) -> Self {
        let mut context_slots = BTreeMap::new();
        let mut context_enabled = Vec::new();
        for context in &action_map.contexts {
            insert_context_slot(
                &mut context_slots,
                &mut context_enabled,
                &context.id,
                context.enabled,
            );
        }
        for action in &action_map.actions {
            if let Some(context) = action.context.as_deref() {
                insert_context_slot(&mut context_slots, &mut context_enabled, context, true);
            }
        }

        let mut bindings_by_action = BTreeMap::<&str, Vec<usize>>::new();
        for (index, binding) in action_map.bindings.iter().enumerate() {
            bindings_by_action
                .entry(binding.action.as_str())
                .or_default()
                .push(index);
        }

        let mut actions = Vec::with_capacity(action_map.actions.len());
        let mut binding_indices = Vec::with_capacity(action_map.bindings.len());
        for (action_index, action) in action_map.actions.iter().enumerate() {
            let binding_start = binding_indices.len();
            if let Some(indices) = bindings_by_action.get(action.id.as_str()) {
                binding_indices.extend(indices.iter().copied());
            }
            let binding_end = binding_indices.len();
            actions.push(CompiledAction {
                action_index,
                context_slot: action
                    .context
                    .as_deref()
                    .and_then(|context| context_slots.get(context).copied()),
                binding_start,
                binding_end,
            });
        }

        Self {
            actions,
            binding_indices,
            context_slots,
            context_enabled,
            has_axis_bindings: action_map
                .bindings
                .iter()
                .any(|binding| !binding.axes.is_empty()),
            #[cfg(test)]
            source_binding_count: action_map.bindings.len(),
        }
    }

    /// 读取按源动作声明顺序排列的编译记录。
    ///
    /// # 返回与生命周期
    /// 返回受 `self` 借用限制的只读切片，位置对应工作区动作槽位。
    /// 记录与范围仍须配合原代际和同一映射；复制记录不会转移或验证这种配对。
    /// 读取不修改代际。
    ///
    /// # 参考
    /// 参见 [`CompiledAction`] 与 [`CompiledAction::binding_indices`]。
    pub(super) fn actions(&self) -> &[CompiledAction] {
        &self.actions
    }

    /// 报告编译时源映射中是否存在轴绑定，供工作区选择轴索引准备路径。
    ///
    /// # 返回与生命周期
    /// 返回配置布尔值的副本；任意源绑定的 `axes` 非空即为 `true`，包括未匹配动作的绑定。
    /// 结果不表示本帧有轴样本或已激活动作；映射替换后由新代际重新反映配置。
    ///
    /// # 参考
    /// 参见 [`Self::from_action_map`] 与 [`InputActionMap`]。
    pub(super) fn has_axis_bindings(&self) -> bool {
        self.has_axis_bindings
    }

    /// 取得当前代际的上下文槽位数量，供工作区准备本帧活动位。
    ///
    /// # 返回与生命周期
    /// 返回并行启用位向量的长度；经本构建路径产生的代际中，它也等于唯一上下文名称数。
    /// 计数包括动作补入的名称，不等同源显式上下文条目数；映射替换后槽位及数量可改变。
    ///
    /// # 参考
    /// 参见 [`Self::context_slot()`] 与 [`insert_context_slot`]。
    pub(super) fn context_count(&self) -> usize {
        self.context_enabled.len()
    }

    /// 按名称查询当前代际中的上下文槽位。
    ///
    /// # 参数
    /// `context` 是用于查找的上下文 id，可为任意字符串切片。
    ///
    /// # 返回与生命周期
    /// 已知名称返回 `Some` 槽号，未知名称返回 `None`；查询不补入名称或修改代际。
    /// 槽号按值返回，不借用名称，但只能用于本代际；替换映射后须重新查询。
    ///
    /// # 参考
    /// 参见 [`Self::context_count`] 与 [`Self::context_enabled()`]。
    pub(super) fn context_slot(&self, context: &str) -> Option<usize> {
        self.context_slots.get(context).copied()
    }

    /// 读取上下文槽位的配置启用位，供求值时结合本帧活动位。
    ///
    /// # 参数
    /// `slot` 应来自当前代际的 [`Self::context_slot()`]。
    ///
    /// # 返回与生命周期
    /// 有效槽位返回保存的布尔值；越界时返回 `true`，不能据此判断上下文存在。
    /// 返回副本且不修改代际，也不读取调用方本帧活动列表；映射变更后应使用新代际。
    ///
    /// # 参考
    /// 参见 [`Self::context_slot()`] 与 [`Self::from_action_map`]。
    pub(super) fn context_enabled(&self, slot: usize) -> bool {
        self.context_enabled.get(slot).copied().unwrap_or(true)
    }

    /// 在测试构建中读取编译时源映射的绑定条目数。
    ///
    /// # 返回与生命周期
    /// 返回原 `bindings.len()`，包含未知动作绑定；不表示扁平范围总长、当前帧访问、激活或输出数量。
    /// 此计数随代际固定，生产构建没有该方法与对应字段。
    ///
    /// # 参考
    /// 参见 [`Self::from_action_map`] 与 [`super::InputActionEvaluator::indexed_binding_candidate_count`]。
    #[cfg(test)]
    pub(super) fn candidate_count(&self) -> usize {
        self.source_binding_count
    }
}

/// 为尚未出现的上下文名称分配一个并行槽位。
///
/// # 参数
/// `context_slots` 接收拥有的名称与槽号，`context_enabled` 接收同槽号的启用位；两者须来自同一次构建并保持一致。
/// `context` 是待插入名称；`enabled` 仅在名称首次插入时写入。
///
/// # 返回与约束
/// 返回 `()`。名称已存在时不改变任何表；否则以启用位向量的当前长度分配槽号，再追加状态。
/// 函数不校验调用方已提供的槽号与向量是否一致；先处理显式声明可保留其首次启用位。
///
/// # 生命周期
/// 首次插入会复制名称并由表拥有；调用结束后不保留 `context` 借用。
/// 生产构建与测试基线共用此函数，因此相关比较不是两套独立上下文插入实现。
///
/// # 参考
/// 参见 [`ActionEvaluationGeneration::from_action_map`] 与 [`ActionEvaluationGeneration`]。
fn insert_context_slot(
    context_slots: &mut BTreeMap<String, usize>,
    context_enabled: &mut Vec<bool>,
    context: &str,
    enabled: bool,
) {
    if context_slots.contains_key(context) {
        return;
    }
    let slot = context_enabled.len();
    context_slots.insert(context.to_owned(), slot);
    context_enabled.push(enabled);
}

/// 仅在测试构建中接入代际比较、源码守卫及默认忽略的计时样例。
///
/// # 生命周期
/// 测试可访问本模块私有数据，生产构建不包含它；测试结果不构成未运行的性能结论。
///
/// # 参考
/// 参见 [`ActionEvaluationGeneration`] 与 [`InputActionMap`]。
#[cfg(test)]
#[path = "tests/generation.rs"]
mod tests;
