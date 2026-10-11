//! 拥有动作求值一次调用所需的帧轴索引、消费输入下标、上下文活动位和动作结果槽位。
//!
//! 求值器在绑定循环前准备工作区，按同一编译代际的动作顺序写入结果，再投影成拥有型公开状态。
//! 工作区本身不带锁：独立求值器持内部 Mutex，默认管理器持外层求值器锁并直接借用工作区。
//! 各组件复用已分配容量；替换映射后清空逻辑内容，下一次求值须重新准备。
//!
//! # 生命周期与约束
//!
//! 槽号与输入索引只适用于配对的代际和本次消费切片。保存的索引不拥有调用方切片，也不延长其借用。
//! 公开输出拥有自己的集合和字符串；销毁输出不清理本工作区，工作区随求值器销毁时释放其存储。
//!
//! # 参考
//!
//! [`ActionEvaluationWorkspace`]、[`EvaluatedAction`]、[`super::InputActionEvaluator`]、[`crate::input::InputActionState`]。
use crate::input::{GamepadAxisInput, InputButton, InputFrameSnapshot};

use super::consumed_input_index::ConsumedInputIndex;
use super::frame_axis_index::FrameAxisIndex;
use super::generation::ActionEvaluationGeneration;

/// 保存单个动作当前求值的按下、边沿及聚合数值。
///
/// 结果按代际动作槽位写入，投影后用于公开状态；派生 `Default` 为三个 `false` 与 `0.0`。
/// 值可复制，字段不强制互斥或数值范围；下次准备重置槽位，类型自身不保存历史帧。
///
/// # 参考
///
/// [`ActionEvaluationWorkspace::action`]、[`ActionEvaluationWorkspace::set_action`]、[`crate::input::InputActionState`]。
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct EvaluatedAction {
    /// 绑定循环聚合的本次按下标志；投影为真时导出动作 ID 及数值。它是结果标志，不验证绑定非空或保证存在物理输入；准备时重置。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`super::project_action_state`]。
    pub(super) pressed: bool,
    /// 本次按钮按下或轴零到非零转换聚合的激活边沿；投影仅在 pressed 为真时导出，准备时重置。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`super::project_action_state`]。
    pub(super) just_activated: bool,
    /// 本次按钮释放或轴非零到零转换聚合的停用边沿；pressed 仍为真时投影不导出，可与其他标志同时存储。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`super::project_action_state`]。
    pub(super) just_deactivated: bool,
    /// 绑定循环按绝对值优势聚合的帧内数值，默认零；按钮候选为1，轴候选由绑定解释。此槽位不归一化，公开状态构造时才清理和限幅。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`crate::input::InputActionState::from_sets_and_values`]。
    pub(super) value: f32,
}

/// 复用当前求值的索引、活动上下文位和按动作槽位排列的结果存储。
///
/// 派生默认值为空；准备时重置动作并刷新索引，reset清空逻辑内容但不缩减容量或清零测试增长计数。
/// 工作区不包含锁或代际身份校验。调用方提供独占访问，并确保槽位、映射、帧与消费切片相互配对。
/// 只保存拥有型值及整数位置，不持有输入引用；公开投影结果不借用本工作区。
///
/// # 参考
///
/// [`ActionEvaluationWorkspace::prepare`]、[`ActionEvaluationWorkspace::reset`]、[`EvaluatedAction`]、[`super::InputActionEvaluator`]。
///
/// # 既有说明
///
/// Reusable evaluator-local state. The default action manager serializes evaluator calls.
#[derive(Debug, Default)]
pub(super) struct ActionEvaluationWorkspace {
    /// 拥有当前帧轴值与转换索引，仅在代际含轴绑定时装载。下一次准备刷新或清空内容，清空保留容量，不持有帧借用。
    ///
    /// # 参考
    ///
    /// [`FrameAxisIndex`]、[`ActionEvaluationWorkspace::prepare_frame_axes`]。
    frame_axes: FrameAxisIndex,
    /// 拥有当前消费按钮和轴切片的排序位置，不拥有输入值。查询须配合装载时的切片内容和顺序；后续准备可刷新或复用有效下标。
    ///
    /// # 参考
    ///
    /// [`ConsumedInputIndex`]、[`ActionEvaluationWorkspace::prepare_consumed_inputs`]。
    consumed_inputs: ConsumedInputIndex,
    /// 按本代际上下文槽位保存本次活动标记；未知名不增加槽位，重复标记幂等。空列表的全部启用语义及配置禁用位由外层查询合并。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationGeneration`]、[`super::action_context_is_active`]。
    active_contexts: Vec<bool>,
    /// 按本代际动作声明位置拥有暂存结果；槽号不是动作 ID，也不检查代际身份。准备调整长度并置默认值，reset清空长度而保留容量。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`ActionEvaluationWorkspace::prepare_actions`]。
    actions: Vec<EvaluatedAction>,
    /// 仅测试构建存在的饱和累计容量增长观测次数，不是元素数、字节数或实际分配调用次数。准备函数按组报告，reset不清零。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationWorkspace::record_growth`]、[`ActionEvaluationWorkspace::storage_growth_count()`]。
    #[cfg(test)]
    storage_growth_count: usize,
}

impl ActionEvaluationWorkspace {
    // 每次评估重置动作结果并装入本帧上下文、轴和消费输入；容量在热路径跨帧复用。
    /// 在每次绑定求值前准备同一代际的全部工作区组件。
    ///
    /// 本方法不核对代际身份。消费查询应继续传入相同切片；名称只用于本次查找，不保留借用。空名称列表的全部启用规则由外层处理，AsRef 行为由输入类型决定。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `generation`：与当前动作映射配对的编译代际。
    /// - `frame`：只在索引装载期间借用的帧快照。
    /// - `active_contexts`：本次活动名称列表，逐项调用 `AsRef<str>`。
    /// - `consumed_buttons`：由调用方持有的消费按钮切片。
    /// - `consumed_axes`：由调用方持有的消费轴切片。
    ///
    /// # 返回
    ///
    /// 返回 `()`；依次完成动作重置、上下文活动位、帧轴和消费索引准备，尚未计算或投影动作。
    ///
    /// # 生命周期
    ///
    /// 准备只复制当前帧所需的值和位置；本次绑定查询须继续使用配对代际和消费切片。后续准备可覆盖结果，已分配容量继续复用。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationGeneration`]、[`InputFrameSnapshot`]、[`ActionEvaluationWorkspace::set_action`]、[`ActionEvaluationWorkspace::reset`]。
    pub(super) fn prepare(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
        active_contexts: &[impl AsRef<str>],
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        self.prepare_actions(generation.actions().len());
        self.prepare_contexts(generation, active_contexts);
        self.prepare_frame_axes(generation, frame);
        self.prepare_consumed_inputs(consumed_buttons, consumed_axes);
    }

    /// 借用最近准备的帧轴索引。
    ///
    /// 读取不分配或转移所有权；后续准备/reset可改变内容，查询前应已完成对应帧准备。
    ///
    /// # 参数
    ///
    /// - `self`：借用工作区。
    ///
    /// # 返回
    ///
    /// 受 `self` 借用限制的只读索引引用。
    ///
    /// # 生命周期
    ///
    /// 返回引用随工作区共享借用结束；需要刷新或清空索引时须重新取得工作区的独占访问。
    ///
    /// # 参考
    ///
    /// [`FrameAxisIndex`]、[`ActionEvaluationWorkspace::prepare_frame_axes`]。
    pub(super) fn frame_axes(&self) -> &FrameAxisIndex {
        &self.frame_axes
    }

    /// 借用最近装载的消费输入排序位置。
    ///
    /// 查询仍须提供与装载配对的消费切片；返回值不拥有这些切片，也不延长它们的借用。
    ///
    /// # 参数
    ///
    /// - `self`：借用工作区。
    ///
    /// # 返回
    ///
    /// 受 `self` 借用限制的只读消费索引引用。
    ///
    /// # 生命周期
    ///
    /// 索引引用受工作区借用限制；消费切片由调用方保活并保持与装载时的内容、顺序配对。
    ///
    /// # 参考
    ///
    /// [`ConsumedInputIndex`]、[`ActionEvaluationWorkspace::prepare_consumed_inputs`]。
    pub(super) fn consumed_inputs(&self) -> &ConsumedInputIndex {
        &self.consumed_inputs
    }

    /// 在测试构建读取最近消费索引装载的源元素数。
    ///
    /// 该统计不表示二分查询或匹配次数；仅测试构建提供，不修改统计。
    ///
    /// # 参数
    ///
    /// - `self`：借用工作区。
    ///
    /// # 返回
    ///
    /// 最近装载时两切片长度的饱和和；索引 clear 后为零。
    ///
    /// # 生命周期
    ///
    /// 只复制测试观测值，不改变索引。后续装载或清空会更新被读取的源元素数。
    ///
    /// # 参考
    ///
    /// [`ConsumedInputIndex::source_visit_count`]。
    #[cfg(test)]
    pub(super) fn consumed_input_source_visit_count(&self) -> usize {
        self.consumed_inputs.source_visit_count()
    }

    /// 按本代际动作槽位复制暂存结果。
    ///
    /// 直接索引，越界会 panic；类型不校验槽号是否来自正确代际。
    ///
    /// # 参数
    ///
    /// - `self`：已准备的工作区。
    /// - `slot`：同一代际动作位置，必须小于当前动作长度。
    ///
    /// # 返回
    ///
    /// 独立的 `EvaluatedAction` 副本，不保留工作区借用。
    ///
    /// # 生命周期
    ///
    /// 结果副本不保留工作区借用；后续准备、槽位覆盖或映射替换不会更新该副本。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`ActionEvaluationGeneration`]、[`ActionEvaluationWorkspace::set_action`]。
    pub(super) fn action(&self, slot: usize) -> EvaluatedAction {
        self.actions[slot]
    }

    /// 替换一个动作槽位的聚合结果。
    ///
    /// 调用方先汇总绑定再写入；方法不验证字段一致性或代际，越界会 panic。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用且已准备的工作区。
    /// - `slot`：同一代际的有效动作位置。
    /// - `action`：复制到槽位的聚合结果。
    ///
    /// # 返回
    ///
    /// 返回 `()`；只覆盖目标槽位。
    ///
    /// # 生命周期
    ///
    /// 写入值归目标槽位保存，输入没有被保留为引用；下一次准备或清空会替换、丢弃此逻辑结果。
    ///
    /// # 参考
    ///
    /// [`EvaluatedAction`]、[`ActionEvaluationWorkspace::action`]、[`ActionEvaluationGeneration`]。
    pub(super) fn set_action(&mut self, slot: usize, action: EvaluatedAction) {
        self.actions[slot] = action;
    }

    /// 查询本次准备的上下文活动标记。
    ///
    /// 这里只查工作区标记；外层另合并配置启用位、无上下文动作及空活动列表规则。
    ///
    /// # 参数
    ///
    /// - `self`：借用工作区。
    /// - `slot`：应来自配对代际的上下文槽号。
    ///
    /// # 返回
    ///
    /// 有效槽位返回布尔副本；越界或 reset 后返回 `false`。
    ///
    /// # 生命周期
    ///
    /// 结果是本次准备状态的布尔副本，后续名称标记、准备或清空不会回写已返回值。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationGeneration`]、[`super::action_context_is_active`]。
    pub(super) fn context_is_active(&self, slot: usize) -> bool {
        self.active_contexts.get(slot).copied().unwrap_or(false)
    }

    /// 清空旧映射的逻辑工作区内容。
    ///
    /// 映射替换后调用；容量保留，测试源访问数随索引清空，累计增长计数保留。旧槽位不可继续求值，下一次调用须重新准备。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    ///
    /// # 返回
    ///
    /// 返回 `()`；轴、消费索引、上下文位和动作长度清空。
    ///
    /// # 生命周期
    ///
    /// 旧槽位和输入位置不能继续用于求值；下一次调用重新准备。工作区仍拥有容量，销毁时释放。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationWorkspace::prepare`]、[`super::InputActionEvaluator::set_action_map`]。
    pub(super) fn reset(&mut self) {
        self.frame_axes.clear();
        self.consumed_inputs.clear();
        self.active_contexts.clear();
        self.actions.clear();
    }

    /// 调整动作槽位数并恢复默认帧内结果。
    ///
    /// 调整前按容量是否不足记录增长观测，再委托重置。容量足够时复用存储，方法本身不识别代际或投影输出。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `action_count`：生产路径使用本代际动作数；测试可指定目标长度。
    ///
    /// # 返回
    ///
    /// 返回 `()`；全部保留和新增槽位均为默认值。
    ///
    /// # 生命周期
    ///
    /// 向量保持目标逻辑长度和可复用容量；所有有效结果只属于即将开始的本次求值。
    ///
    /// # 参考
    ///
    /// [`reset_action_storage`]、[`EvaluatedAction`]、[`ActionEvaluationWorkspace::record_growth`]。
    fn prepare_actions(&mut self, action_count: usize) {
        self.record_growth(
            self.actions.len() < action_count && self.actions.capacity() < action_count,
        );
        reset_action_storage(&mut self.actions, action_count);
    }

    /// 按代际槽位刷新本次活动上下文位。
    ///
    /// 先调整长度并全置假，再标记已知名；未知名忽略、重复名幂等。只记录活动位，不修改代际配置启用位；AsRef 实现决定名称转换行为。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `generation`：与调用路径配对的代际。
    /// - `active_contexts`：逐项借用名称用于查找的列表。
    ///
    /// # 返回
    ///
    /// 返回 `()`；位向量长度为代际上下文数，已知名称位置为真。
    ///
    /// # 生命周期
    ///
    /// 名称只在本次查找期间借用，保存的布尔位与传入代际配对；下次准备覆盖本次活动位。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationGeneration::context_slot()`]、[`ActionEvaluationWorkspace::context_is_active`]、[`super::action_context_is_active`]。
    fn prepare_contexts(
        &mut self,
        generation: &ActionEvaluationGeneration,
        active_contexts: &[impl AsRef<str>],
    ) {
        let context_count = generation.context_count();
        self.record_growth(
            self.active_contexts.len() < context_count
                && self.active_contexts.capacity() < context_count,
        );
        self.active_contexts.resize(context_count, false);
        self.active_contexts.fill(false);
        for context in active_contexts {
            if let Some(slot) = generation.context_slot(context.as_ref()) {
                self.active_contexts[slot] = true;
            }
        }
    }

    /// 按代际轴绑定标志刷新或清空帧轴索引。
    ///
    /// 无轴路径不扫描帧轴；装载复制值/转换而不保留帧引用，按前后合计容量比较报告增长。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `generation`：提供编译时轴绑定标志的代际。
    /// - `frame`：有轴绑定时借读的帧快照。
    ///
    /// # 返回
    ///
    /// 返回 `()`；无轴绑定时清空并提前返回，有轴绑定时装载索引。
    ///
    /// # 生命周期
    ///
    /// 工作区拥有复制后的轴值和转换，不保留帧借用；下次准备可以刷新或清空索引内容。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationGeneration::has_axis_bindings`]、[`FrameAxisIndex::load_frame`]、[`ActionEvaluationWorkspace::record_growth`]。
    fn prepare_frame_axes(
        &mut self,
        generation: &ActionEvaluationGeneration,
        frame: &InputFrameSnapshot,
    ) {
        if !generation.has_axis_bindings() {
            self.frame_axes.clear();
            return;
        }
        let capacity_before = self.frame_axes.storage_capacity();
        self.frame_axes.load_frame(frame);
        self.record_growth(self.frame_axes.storage_capacity() > capacity_before);
    }

    /// 为当前消费切片刷新或复用排序下标。
    ///
    /// 装载过程可保留仍有效的排序位置；工作区不取得切片所有权或存储引用。按装载前后合计容量报告增长，后续查询须保持配对。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `consumed_buttons`：调用方只读消费按钮切片。
    /// - `consumed_axes`：调用方只读消费轴切片。
    ///
    /// # 返回
    ///
    /// 返回 `()`；索引可用于配对切片的后续消费查询。
    ///
    /// # 生命周期
    ///
    /// 保存的整数位置不保留切片借用；调用方在后续查询期间维持切片配对。清空或下一次装载可以改变位置。
    ///
    /// # 参考
    ///
    /// [`ConsumedInputIndex::load`]、[`ActionEvaluationWorkspace::consumed_inputs()`]。
    fn prepare_consumed_inputs(
        &mut self,
        consumed_buttons: &[InputButton],
        consumed_axes: &[GamepadAxisInput],
    ) {
        let capacity_before = self.consumed_inputs.storage_capacity();
        self.consumed_inputs.load(consumed_buttons, consumed_axes);
        self.record_growth(self.consumed_inputs.storage_capacity() > capacity_before);
    }

    /// 按调用方提供的增长观测更新测试累计计数。
    ///
    /// 本方法不测量容量或执行分配；一次观测可对应多个内部向量，reset不清零累计值。
    ///
    /// # 参数
    ///
    /// - `self`：独占借用的工作区。
    /// - `grew`：本组准备是否观察或预判需要增长，不表示增长幅度。
    ///
    /// # 返回
    ///
    /// 返回 `()`；测试构建为真时饱和加一，生产构建忽略此参数。
    ///
    /// # 生命周期
    ///
    /// 测试累计值在工作区存续期间饱和增长，普通清空不归零；非测试构建不保存该计数。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationWorkspace`]、[`ActionEvaluationWorkspace::prepare`]。
    fn record_growth(&mut self, grew: bool) {
        #[cfg(test)]
        if grew {
            self.storage_growth_count = self.storage_growth_count.saturating_add(1);
        }
        #[cfg(not(test))]
        let _ = grew;
    }

    /// 在测试构建读取容量增长观测的累计次数。
    ///
    /// 生产构建不提供该方法；reset与逐帧准备不会把累计计数归零，它不是分配调用次数。
    ///
    /// # 参数
    ///
    /// - `self`：借用工作区。
    ///
    /// # 返回
    ///
    /// 自默认初始化或调用方设置以来的 `usize` 副本，不清零。
    ///
    /// # 生命周期
    ///
    /// 读取不改变累计值；返回的整数不借用工作区，也不跟随后续增长变化。
    ///
    /// # 参考
    ///
    /// [`ActionEvaluationWorkspace::record_growth`]、[`super::InputActionEvaluator::workspace_storage_growth_count`]。
    #[cfg(test)]
    pub(super) fn storage_growth_count(&self) -> usize {
        self.storage_growth_count
    }
}

/// 调整结果向量长度并重置仍保留的旧前缀。
///
/// 缩小时丢弃尾部；扩展时 resize 为新槽位提供默认值，仅再次填充调整前保留的前缀。容量足够时不重新分配，清空至零仍保留容量。
///
/// # 参数
///
/// - `actions`：独占借用的动作向量。
/// - `action_count`：目标逻辑长度。
///
/// # 返回
///
/// 返回 `()`；向量长度匹配目标，全部逻辑元素为默认结果。
///
/// # 生命周期
///
/// 输入向量仍由调用方拥有，方法只独占借用。归零不释放容量，缩减丢弃的尾部不再属于逻辑结果。
///
/// # 参考
///
/// [`EvaluatedAction`]、[`ActionEvaluationWorkspace::prepare_actions`]。
fn reset_action_storage(actions: &mut Vec<EvaluatedAction>, action_count: usize) {
    let reset_len = actions.len().min(action_count);
    actions.resize(action_count, EvaluatedAction::default());
    actions[..reset_len].fill(EvaluatedAction::default());
}

/// 在测试构建接入动作槽位重置等价、源码形状与默认忽略的计时夹具。
///
/// 测试子模块可访问父模块私有状态；生产构建不包含该模块。源码守卫依赖文本分隔符，计时样例需显式运行才能产生结果。
///
/// # 参考
///
/// [`EvaluatedAction`]、[`ActionEvaluationWorkspace`]、[`reset_action_storage`]。
#[cfg(test)]
#[path = "tests/workspace.rs"]
mod tests;
