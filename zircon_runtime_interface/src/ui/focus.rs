//! 焦点准入、可见性及输入记录的公共值类型，并提供只读的 Tab 候选收集。
//!
//! 这些值保存节点标识和自有集合，不持有保留树的借用；实际焦点变更由 Runtime 的 Surface 维护。
//! 候选收集只读取根节点可达的树结构，不执行导航分组、模态或边界策略。

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::tree::UiTree;

/// Controls whether a focusable node can receive pointer and keyboard focus.
///
/// 描述已声明可聚焦的节点允许哪类输入；是否启用、可见以及可聚焦仍由调用方另外检查。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiFocusMode {
    None,
    Click,
    #[default]
    All,
}

impl UiFocusMode {
    /// 返回该模式是否允许指针聚焦；此检查不读取节点状态。
    pub const fn allows_pointer_focus(self) -> bool {
        !matches!(self, Self::None)
    }

    /// 返回该模式是否允许 Tab 聚焦；只有完整输入模式返回真。
    pub const fn allows_tab_focus(self) -> bool {
        matches!(self, Self::All)
    }
}

/// Identifies why a node became focused so visual policy can distinguish keyboard focus rings.
///
/// 焦点原因只决定默认焦点环意图；创建该值不会切换实际焦点。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiFocusCause {
    Pointer,
    #[default]
    Navigation,
    Programmatic,
    Restore,
}

impl UiFocusCause {
    /// 将原因映射为可见性值：导航显示焦点环，指针、程序切换和恢复均隐藏。
    /// 返回独立值；恢复与程序切换共用程序原因，不保留两者的来源差异。
    pub const fn focus_visible(self) -> UiFocusVisible {
        match self {
            Self::Navigation => UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
            Self::Pointer => UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
            Self::Programmatic | Self::Restore => {
                UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic)
            }
        }
    }
}

/// 焦点环显示或隐藏的原因标签；标签本身不约束可见位的取值。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusVisibleReason {
    #[default]
    Initial,
    KeyboardNavigation,
    PointerInteraction,
    Programmatic,
    DisabledOrHidden,
}

/// 焦点环可见位及其原因的可序列化快照。
/// 两个字段可独立构造，反序列化不会核验它们的业务组合。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusVisible {
    pub visible: bool,
    pub reason: UiFocusVisibleReason,
}

impl UiFocusVisible {
    /// 以调用方给出的原因创建显示焦点环的值；不更新节点或 Surface。
    pub const fn visible(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: true,
            reason,
        }
    }

    /// 以调用方给出的原因创建隐藏焦点环的值；不更新节点或 Surface。
    pub const fn hidden(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: false,
            reason,
        }
    }
}

/// 保存当前、前次和待自动聚焦的节点标识，以及焦点环状态。
/// 这是可独立持有的记录；构造和反序列化都不会查树或执行焦点转移。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputFocus {
    pub focused: Option<UiNodeId>,
    pub previous: Option<UiNodeId>,
    pub pending_autofocus: Option<UiNodeId>,
    pub focus_visible: UiFocusVisible,
}

/// 焦点变化记录中的原因标签；清除、失效和显式切换可由消费者分别标记。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusChangeReason {
    #[default]
    Input,
    Navigation,
    Programmatic,
    Autofocus,
    Clear,
    Disabled,
    Hidden,
    Despawned,
}

/// 一次焦点变化的前后标识、原因与可见性快照。
/// 前后标识都可为空；该值记录结果，不请求切换焦点，也不保证事件已经派发。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusChangeEvent {
    pub previous: Option<UiNodeId>,
    pub current: Option<UiNodeId>,
    pub reason: UiFocusChangeReason,
    pub visible: UiFocusVisible,
}

/// 焦点输入记录的来源种类；只区分输入类别，不携带对应事件负载。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusedInputKind {
    #[default]
    Keyboard,
    Text,
    Ime,
    Navigation,
    Pointer,
    AccessibilityAction,
}

/// 一次面向焦点的输入处理记录，包含自有冒泡路径、处理者和接受结果。
/// 路径中的标识不会借用或延长节点生命周期；构造该记录不校验处理者是否属于路径。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusedInput {
    pub focused: UiNodeId,
    pub kind: UiFocusedInputKind,
    pub route: Vec<UiNodeId>,
    pub handled_by: Option<UiNodeId>,
    pub accepted: bool,
}

/// 节点声明的焦点准入与恢复配置。
///
/// # 约束与生命周期
/// 可聚焦位与输入模式共同决定准入；模式的默认值允许全部输入，但可聚焦位默认关闭。
/// 自动聚焦、关闭恢复和可选焦点环配置只是声明，类型本身不执行这些动作。
/// 该值不借用节点；树和 Surface 的消费者负责检查标识有效性及实际切换时机。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusContract {
    pub focusable: bool,
    #[serde(default)]
    pub mode: UiFocusMode,
    pub autofocus: bool,
    pub restore_on_close: bool,
    pub focus_visible: Option<UiFocusVisible>,
}

impl UiFocusContract {
    /// 同时检查可聚焦位和指针模式；可见性、启用位及模态约束由调用方检查。
    pub const fn allows_pointer_focus(&self) -> bool {
        self.focusable && self.mode.allows_pointer_focus()
    }

    /// 同时检查可聚焦位和 Tab 模式；可见性、启用位及模态约束由调用方检查。
    pub const fn allows_tab_focus(&self) -> bool {
        self.focusable && self.mode.allows_tab_focus()
    }
}

/// Returns the stable Tab traversal order for the reachable retained tree.
///
/// Authored tab indices sort before default pre-order candidates; equal indices retain pre-order.
///
/// 按根节点和子节点的前序顺序收集 Tab 候选，再把显式索引分区放在默认分区之前。
///
/// # 参数
/// - `tree`：本次读取的保留树；仅沿根可达的子节点收集，缺失或已访问的节点跳过。
///
/// # 返回
/// 返回自有节点标识数组；显式索引按数值、再按前序位置排序，默认候选保持前序顺序。
///
/// # 约束与生命周期
/// 准入要求祖先与自身渲染可见、本节点启用且允许 Tab，并且显式索引未禁用。
/// 父节点的启用位不传给后代；导航分组、模态范围及边界策略不在此函数中执行。
/// 树只在调用期间被共享借用，结果不借用树；函数不会修改树、实际焦点或事件路由。
pub fn focus_chain(tree: &UiTree) -> Vec<UiNodeId> {
    let mut default_candidates = Vec::new();
    let mut indexed_candidates = Vec::new();
    let mut visited = HashSet::with_capacity(tree.nodes.len());
    let mut pre_order = 0usize;

    for root in &tree.roots {
        collect_focus_candidates(
            tree,
            *root,
            true,
            &mut visited,
            &mut pre_order,
            &mut default_candidates,
            &mut indexed_candidates,
        );
    }

    finish_focus_chain(default_candidates, indexed_candidates)
}

/// 显式索引分区的内部记录；前序位置用于保持相同索引之间的遍历次序。
struct UiFocusChainCandidate {
    node_id: UiNodeId,
    tab_index: crate::ui::navigation::UiTabIndex,
    pre_order: usize,
}

/// 合并已按准入分成的默认分区与显式索引分区。
///
/// # 参数与返回
/// 两个数组均移交所有权；仅排序显式分区，然后输出显式分区和默认分区的节点标识。
/// 显式分区为空时直接返回默认数组，保留其存储；否则结果拥有新数组，不借用输入。
fn finish_focus_chain(
    mut default_candidates: Vec<UiNodeId>,
    mut indexed_candidates: Vec<UiFocusChainCandidate>,
) -> Vec<UiNodeId> {
    indexed_candidates.sort_by_key(|candidate| (candidate.tab_index.order, candidate.pre_order));
    if indexed_candidates.is_empty() {
        return default_candidates;
    }

    let mut chain = Vec::with_capacity(indexed_candidates.len() + default_candidates.len());
    chain.extend(
        indexed_candidates
            .into_iter()
            .map(|candidate| candidate.node_id),
    );
    chain.append(&mut default_candidates);
    chain
}

/// 从一个节点递归收集候选，只修改调用方传入的访问集合、前序计数和两个输出分区。
///
/// # 约束
/// 各根共享访问集合以阻止重复及环路；缺失节点会终止本次递归。
/// 祖先渲染可见性向下传递，启用位只检查本节点；不合格的节点仍会继续遍历子节点。
/// 每个首次找到的节点推进一次饱和前序计数，候选的排序不依赖访问集合的迭代顺序。
fn collect_focus_candidates(
    tree: &UiTree,
    node_id: UiNodeId,
    ancestors_render_visible: bool,
    visited: &mut HashSet<UiNodeId>,
    pre_order: &mut usize,
    default_candidates: &mut Vec<UiNodeId>,
    indexed_candidates: &mut Vec<UiFocusChainCandidate>,
) {
    if !visited.insert(node_id) {
        return;
    }
    let Some(node) = tree.node(node_id) else {
        return;
    };

    let render_visible = ancestors_render_visible && node.is_render_visible();
    let tab_index = node.navigation.tab_index;
    if render_visible
        && node.state_flags.enabled
        && node.focus.allows_tab_focus()
        && tab_index.map(|index| index.tabbable).unwrap_or(true)
    {
        if let Some(tab_index) = tab_index {
            indexed_candidates.push(UiFocusChainCandidate {
                node_id,
                tab_index,
                pre_order: *pre_order,
            });
        } else {
            default_candidates.push(node_id);
        }
    }
    *pre_order = pre_order.saturating_add(1);

    for child in &node.children {
        collect_focus_candidates(
            tree,
            *child,
            render_visible,
            visited,
            pre_order,
            default_candidates,
            indexed_candidates,
        );
    }
}

#[cfg(test)]
#[path = "tests/focus_performance_tests.rs"]
mod performance_tests;

#[cfg(test)]
#[path = "focus/tests/focus_tests.rs"]
mod focus_tests;
