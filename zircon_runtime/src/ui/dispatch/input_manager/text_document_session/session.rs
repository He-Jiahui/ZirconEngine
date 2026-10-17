use std::{borrow::Cow, collections::BTreeMap, ops::Range, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    surface::UiEditableTextState,
    text::{UiTextByteSelection, UiTextDocumentKey, UiTextEditKind, UiTextEditSource},
    tree::UiTree,
};

use crate::{
    text::document::{
        PreparedTextDocumentStoreEdit, TextDocumentStore, TextDocumentStoreEditCommit,
    },
    ui::{surface::UiSurfaceSessionIdentityHandle, text::CommittedTextEditIntent},
};

use super::{
    binding::{UiTextDocumentBinding, UiTextDocumentBindingKey},
    error::UiTextDocumentSessionError,
    history::{
        UiTextDocumentHistory, UiTextHistoryCommit, UiTextHistoryDirection, UiTextHistoryEntry,
        MVP_TEXT_HISTORY_MAX_DELTA_BYTES,
    },
    limits::mvp_text_document_store_limits,
};

/// 持有当前树的有界文档存储、owner 绑定、差量历史与最近同步错误。
///
/// # 使用时机
/// 输入管理器在分派前调用 owner 同步，编辑路径再按 Surface 源 epoch 查找文档。
/// 文档 revision 属于存储版本；源 epoch 属于 Surface，同一数值不表示两者可以互换。
///
/// # 生命周期
/// 切换树或 Surface 会话身份会关闭旧绑定；节点脱离树时只关闭该 owner。
/// 同名树的不同 Surface 依赖会话身份区分；直接按树同步源不能替代该身份检查。
/// 正文由存储拥有，历史另持有差量；身份查询返回的键不延长文档版本有效期。
///
/// # 参考
/// [`Self::synchronize_owners`]、[`Self::prepare_edit`] 与 [`Self::finish_edit`]。
pub(in crate::ui) struct UiTextDocumentSession {
    active_surface: Option<UiSurfaceSessionIdentityHandle>,
    active_tree: Option<UiTreeId>,
    store: TextDocumentStore,
    bindings: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentBinding>,
    histories: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentHistory>,
    synchronization_errors: BTreeMap<UiTextDocumentBindingKey, UiTextDocumentSessionError>,
    observed_layout_order_generation: Option<u64>,
    observed_node_count: usize,
}

impl Default for UiTextDocumentSession {
    /// 创建尚未激活树/Surface 的空会话，并为文档存储装配初始有限预算。
    fn default() -> Self {
        Self {
            active_surface: None,
            active_tree: None,
            store: TextDocumentStore::with_limits(mvp_text_document_store_limits()),
            bindings: BTreeMap::new(),
            histories: BTreeMap::new(),
            synchronization_errors: BTreeMap::new(),
            observed_layout_order_generation: None,
            observed_node_count: 0,
        }
    }
}

impl UiTextDocumentSession {
    /// 查询 owner 在指定源 epoch 下已同步的文档身份，不打开新源。
    /// `tree_id`、`node_id` 共同定位 owner；返回复制的文档键，之后仍可能因重绑或编辑失效。
    /// epoch 不匹配或绑定不存在时返回已记录的同步错误，否则返回未同步错误。
    pub(in crate::ui) fn document_key(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
    ) -> Result<UiTextDocumentKey, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        Ok(UiTextDocumentKey {
            document_id: binding.document_id,
            revision: binding.revision,
        })
    }

    /// 在输入约束预检中查询排除替换范围后仍保留的字素数。
    ///
    /// # 参数与返回
    /// `tree_id`、`node_id`、`source_epoch` 须命中已同步绑定；`range` 是该源的 UTF-8 字节范围。
    /// 成功返回保留字素数，绑定、版本、范围或快照预算不满足时传回错误。
    /// 范围两端还须位于字素边界；计数只扣除旧源区间，不包含待插入文本。
    ///
    /// # 状态
    /// 查询可能物化存储索引而需要独占借用，但不提交文档编辑或推进源 epoch。
    pub(in crate::ui) fn retained_grapheme_count(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        range: Range<usize>,
    ) -> Result<usize, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        self.store
            .retained_grapheme_count(binding.document_id, binding.revision, range)
            .map_err(Into::into)
    }

    /// 在输入分派前同步当前 Surface 身份并回收已脱离树的 owner。
    ///
    /// # 参数与状态
    /// `tree` 提供树身份、拓扑代数及节点集合；`surface_identity` 移入当前会话身份记录。
    /// 身份变化先关闭全部旧绑定；随后按拓扑变化全扫绑定，或仅检查待处理变更中的删除节点。
    /// 关闭 owner 同时清除其历史和同步错误，再记下本轮拓扑标记；此方法不打开新文本源。
    pub(in crate::ui) fn synchronize_owners(
        &mut self,
        tree: &UiTree,
        surface_identity: UiSurfaceSessionIdentityHandle,
    ) {
        self.activate_surface(&tree.tree_id, surface_identity);
        let topology_changed = self.observed_layout_order_generation
            != Some(tree.layout_order_generation())
            || self.observed_node_count != tree.nodes.len();
        if topology_changed {
            let detached = self
                .bindings
                .keys()
                .filter(|key| !tree.nodes.contains_key(&key.node_id))
                .cloned()
                .collect::<Vec<_>>();
            for key in detached {
                self.close_binding(&key);
            }
        } else {
            let detached = tree
                .pending_mutation_node_ids()
                .iter()
                .filter(|node_id| !tree.nodes.contains_key(node_id))
                .map(|node_id| UiTextDocumentBindingKey {
                    tree_id: tree.tree_id.clone(),
                    node_id: *node_id,
                })
                .collect::<Vec<_>>();
            for key in detached {
                self.close_binding(&key);
            }
        }
        self.observed_layout_order_generation = Some(tree.layout_order_generation());
        self.observed_node_count = tree.nodes.len();
    }

    /// 为随后与属性共同提交的文档替换准备独占编辑结果。
    ///
    /// # 参数
    /// `tree_id`、`node_id` 定位已同步 owner，`source_epoch` 是当前 Surface 源 epoch。
    /// `intent.old` 指定旧源 UTF-8 字节范围；替换文本从 `next_state` 的意图范围提取。
    /// `next_state` 同时提供公开回执的光标/选区；`source` 记录输入来源。
    ///
    /// # 返回与错误
    /// 返回 [`PreparedTextDocumentStoreEdit`]，其生命周期绑定本会话的独占借用。
    /// 源 epoch 无法再递增、替换范围不可提取、公开偏移不能编码为 u32、绑定不符或存储拒绝时返回错误。
    ///
    /// # 约束与生命周期
    /// 需要的历史动作应在取得该借用前准备；返回值提交前不发布文档版本，丢弃也不会提交编辑。
    /// 调用方将它与属性事务合并，成功后再用 [`Self::finish_edit`] 更新会话记录。
    /// 不同树会先激活并清除旧绑定，因此准备失败不表示此前的树切换被撤销。
    #[allow(clippy::too_many_arguments)]
    pub(in crate::ui) fn prepare_edit<'session>(
        &'session mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        intent: &CommittedTextEditIntent,
        next_state: &UiEditableTextState,
        source: UiTextEditSource,
    ) -> Result<PreparedTextDocumentStoreEdit<'session>, UiTextDocumentSessionError> {
        if source_epoch.checked_add(1).is_none() {
            return Err(UiTextDocumentSessionError::SourceEpochExhausted);
        }
        let replacement = intent
            .replacement(next_state)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)?;
        let selection = public_selection(next_state)?;
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.activate_tree(tree_id);
        let binding = self
            .bindings
            .get(&key)
            .copied()
            .filter(|binding| binding.source_epoch == source_epoch)
            .ok_or_else(|| {
                self.synchronization_errors
                    .get(&key)
                    .copied()
                    .unwrap_or(UiTextDocumentSessionError::SourceNotSynchronized)
            })?;
        self.store
            .prepare_replace_with_receipt(
                binding.document_id,
                binding.revision,
                intent.old.clone(),
                replacement,
                node_id,
                source,
                intent.kind,
                selection,
            )
            .map_err(Into::into)
    }

    /// 在独占文档准备之前决定本次成功编辑的历史动作。
    ///
    /// # 参数
    /// `tree_id`、`node_id`、`source_epoch` 定位源；`intent` 与 `current_state`、`next_state` 必须来自同次转换。
    /// `secure` 表示不得保留编辑正文；差量的删入文本会由返回值拥有，不借用两侧状态。
    ///
    /// # 返回与约束
    /// 安全输入返回屏障，Undo/Redo 返回对应动作；这些提前分支不证明文档绑定有效。
    /// 普通编辑核对绑定并读取被删源，差量字节总量溢出或超限则返回屏障，否则返回待记录条目。
    /// 绑定、意图或存储读取失败会返回错误；本方法不推进队列，也不代替后续文档编辑预检。
    #[allow(clippy::too_many_arguments)]
    pub(in crate::ui) fn prepare_history_commit(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        intent: &CommittedTextEditIntent,
        current_state: &UiEditableTextState,
        next_state: &UiEditableTextState,
        secure: bool,
    ) -> Result<UiTextHistoryCommit, UiTextDocumentSessionError> {
        if secure {
            return Ok(UiTextHistoryCommit::Barrier);
        }
        match intent.kind {
            UiTextEditKind::Undo => return Ok(UiTextHistoryCommit::Undo),
            UiTextEditKind::Redo => return Ok(UiTextHistoryCommit::Redo),
            _ => {}
        }
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        let inserted = intent
            .replacement(next_state)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)?;
        let retained_bytes = intent.old.len().checked_add(inserted.len());
        if retained_bytes.map_or(true, |bytes| bytes > MVP_TEXT_HISTORY_MAX_DELTA_BYTES) {
            return Ok(UiTextHistoryCommit::Barrier);
        }
        let removed =
            self.store
                .source_range(binding.document_id, binding.revision, intent.old.clone())?;
        Ok(UiTextHistoryCommit::Record(UiTextHistoryEntry::new(
            intent,
            removed,
            inserted.to_string(),
            current_state,
            next_state,
        )))
    }

    /// 从指定方向的最新历史准备下一编辑状态，不移动队列。
    ///
    /// # 参数与返回
    /// owner 与 `source_epoch` 须匹配绑定；`current_state` 被消费，`direction` 选择 Undo 或 Redo。
    /// 没有对应条目返回 Ok(None)；成功返回含替换意图的下一状态。
    /// 先核对存储源，再核对传入状态的预期范围正文；失配、范围或存储错误返回 Err。
    /// 调用方仍须提交文档/属性事务，成功后才应用相应历史动作。
    pub(in crate::ui) fn prepare_history_transition(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        current_state: UiEditableTextState,
        direction: UiTextHistoryDirection,
    ) -> Result<Option<crate::ui::text::TextEditStateTransition>, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        let Some(entry) = self
            .histories
            .get(&key)
            .and_then(|history| history.latest(direction))
        else {
            return Ok(None);
        };
        let range = entry.expected_range(direction);
        let source = self
            .store
            .source_range(binding.document_id, binding.revision, range)?;
        if source != entry.expected_text(direction) {
            return Err(UiTextDocumentSessionError::InvalidEditIntent);
        }
        entry
            .transition(current_state, direction)
            .map(Some)
            .ok_or(UiTextDocumentSessionError::InvalidEditIntent)
    }

    /// 释放指定树与节点的撤销/重做历史，不关闭其文档绑定。
    pub(in crate::ui) fn discard_history(&mut self, tree_id: &UiTreeId, node_id: UiNodeId) {
        self.histories.remove(&UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        });
    }

    /// 在失焦队列溢出等全局隔离边界释放所有历史，保留文档绑定。
    pub(in crate::ui) fn discard_all_histories(&mut self) {
        self.histories.clear();
    }

    /// 比较候选正文与 owner 当前绑定的已提交源，不比较 IME 可见投影。
    /// `candidate` 只在调用期间借用；身份/epoch 匹配后由存储继续核对 revision。
    /// 成功返回是否相等，绑定或存储失败返回 Err，不发布新源。
    pub(in crate::ui) fn committed_source_matches(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        candidate: &str,
    ) -> Result<bool, UiTextDocumentSessionError> {
        let binding = self.binding(tree_id, node_id, source_epoch)?;
        self.store
            .source_equals(binding.document_id, binding.revision, candidate)
            .map_err(Into::into)
    }

    /// 从可见编辑状态取得提交源，并在需要时建立 owner 绑定。
    ///
    /// # 参数与调用时机
    /// `tree_id`、`node_id`、`source_epoch` 标识本轮源；`state` 仅在调用期间借用。
    /// 首次接入带 restore_text 的组合状态时，用该文本替换可见组合范围还原提交源；
    /// 没有恢复文本则使用可见正文。已有同 epoch 绑定会直接复用，不再次核对正文。
    ///
    /// # 状态与错误
    /// 不同树先清除旧绑定；恢复范围不是有效 UTF-8 切片时记下意图错误并停止。
    /// 还原后的临时正文仅用于同步，最终源由存储拥有；存储错误供后续查询返回。
    pub(in crate::ui) fn synchronize_editable_source(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        state: &UiEditableTextState,
    ) {
        self.activate_tree(tree_id);
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        if self
            .bindings
            .get(&key)
            .is_some_and(|binding| binding.source_epoch == source_epoch)
        {
            self.synchronization_errors.remove(&key);
            return;
        }

        // An active preedit is a visible projection. A new session must open its
        // restored source, while an existing source lease avoids this materialization.
        let source = match state.composition.as_ref().and_then(|composition| {
            composition
                .restore_text
                .as_deref()
                .map(|restore| (composition.range, restore))
        }) {
            Some((range, restore)) => {
                let Some(replaced) = state.text.get(range.start..range.end) else {
                    self.synchronization_errors
                        .insert(key, UiTextDocumentSessionError::InvalidEditIntent);
                    return;
                };
                if replaced == restore {
                    Cow::Borrowed(state.text.as_str())
                } else {
                    let mut source =
                        String::with_capacity(state.text.len() - replaced.len() + restore.len());
                    source.push_str(&state.text[..range.start]);
                    source.push_str(restore);
                    source.push_str(&state.text[range.end..]);
                    Cow::Owned(source)
                }
            }
            None => Cow::Borrowed(state.text.as_str()),
        };
        self.synchronize_source(tree_id, node_id, source_epoch, source.as_ref());
    }

    /// 以调用方提供的提交正文为 owner 打开或重绑文档。
    ///
    /// # 参数与约束
    /// `tree_id`、`node_id` 定位 owner；`source_epoch` 表示该正文对应的 Surface 源世代。
    /// `current_text` 仅在调用期间借用，打开成功后由存储持有独立的共享字符串。
    /// 同 epoch 绑定直接复用且清除旧错误；外部源变化须通过 Surface 的 epoch 机制体现。
    ///
    /// # 状态与错误
    /// 重绑先关闭旧文档并移除历史，再尝试有界打开；失败不会恢复旧绑定，而是保存错误供查询返回。
    /// 不同树先清除旧会话绑定；此方法不直接返回诊断结果。
    pub(in crate::ui) fn synchronize_source(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        current_text: &str,
    ) {
        self.activate_tree(tree_id);
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        if self
            .bindings
            .get(&key)
            .is_some_and(|binding| binding.source_epoch == source_epoch)
        {
            self.synchronization_errors.remove(&key);
            return;
        }
        if let Some(binding) = self.bindings.remove(&key) {
            self.store.close(binding.document_id);
        }
        self.histories.remove(&key);
        match self.store.open(Arc::<str>::from(current_text)) {
            Ok(opened) => {
                self.bindings.insert(
                    key.clone(),
                    UiTextDocumentBinding {
                        document_id: opened.document_id,
                        revision: opened.revision,
                        source_epoch,
                    },
                );
                self.synchronization_errors.remove(&key);
            }
            Err(error) => {
                self.synchronization_errors.insert(key, error.into());
            }
        }
    }

    /// 在文档/属性事务成功后登记 owner 的文档版本、源 epoch 与历史后效。
    ///
    /// # 参数与约束
    /// `commit` 必须来自该树/节点对应的成功事务，`source_epoch` 是事务后的 Surface 源世代。
    /// `history_commit` 被消费；本方法不重做文档编辑、不校验回执归属，也不推进 Surface epoch。
    ///
    /// # 状态
    /// Changed 才推进历史或应用屏障；Unchanged 保留历史，但两者均刷新绑定并清除同步错误。
    /// 调用方须先结束准备编辑的独占借用，再传入提交回执。
    pub(in crate::ui) fn finish_edit(
        &mut self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
        commit: &TextDocumentStoreEditCommit,
        history_commit: UiTextHistoryCommit,
    ) {
        let (document_id, revision) = match commit {
            TextDocumentStoreEditCommit::Unchanged {
                document_id,
                revision,
            } => (*document_id, *revision),
            TextDocumentStoreEditCommit::Changed { public_receipt, .. } => {
                (public_receipt.document_id, public_receipt.revision)
            }
        };
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.synchronization_errors.remove(&key);
        if matches!(commit, TextDocumentStoreEditCommit::Changed { .. }) {
            match history_commit {
                UiTextHistoryCommit::Barrier => {
                    self.histories.remove(&key);
                }
                history_commit => self
                    .histories
                    .entry(key.clone())
                    .or_default()
                    .commit(history_commit),
            }
        }
        self.bindings.insert(
            key,
            UiTextDocumentBinding {
                document_id,
                revision,
                source_epoch,
            },
        );
    }

    /// 复制 epoch 匹配的绑定；缺失或失配时优先返回该 owner 已记录的同步错误。
    fn binding(
        &self,
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        source_epoch: u64,
    ) -> Result<UiTextDocumentBinding, UiTextDocumentSessionError> {
        let key = UiTextDocumentBindingKey {
            tree_id: tree_id.clone(),
            node_id,
        };
        self.bindings
            .get(&key)
            .copied()
            .filter(|binding| binding.source_epoch == source_epoch)
            .ok_or_else(|| {
                self.synchronization_errors
                    .get(&key)
                    .copied()
                    .unwrap_or(UiTextDocumentSessionError::SourceNotSynchronized)
            })
    }

    /// 直接源操作按树切换状态；同树不变，不以此检测同树标识下的 Surface 身份变化。
    fn activate_tree(&mut self, tree_id: &UiTreeId) {
        if self.active_tree.as_ref() == Some(tree_id) {
            return;
        }
        self.reset_bindings();
        self.active_surface = None;
        self.active_tree = Some(tree_id.clone());
    }

    /// 同时核对树与 Surface 会话身份；任一变化均释放旧 owner 状态后接纳新身份。
    fn activate_surface(
        &mut self,
        tree_id: &UiTreeId,
        surface_identity: UiSurfaceSessionIdentityHandle,
    ) {
        if self.active_tree.as_ref() == Some(tree_id)
            && self.active_surface.as_ref() == Some(&surface_identity)
        {
            return;
        }
        self.reset_bindings();
        self.active_surface = Some(surface_identity);
        self.active_tree = Some(tree_id.clone());
    }

    /// 先关闭所有已绑定文档，再清空历史、诊断及拓扑观察值；激活身份由调用方随后设置。
    fn reset_bindings(&mut self) {
        for binding in self.bindings.values() {
            self.store.close(binding.document_id);
        }
        self.bindings.clear();
        self.histories.clear();
        self.synchronization_errors.clear();
        self.observed_layout_order_generation = None;
        self.observed_node_count = 0;
    }

    /// 关闭单个 owner 的文档并释放同键历史/诊断；键不存在时仍清除可能残留的附属状态。
    fn close_binding(&mut self, key: &UiTextDocumentBindingKey) {
        if let Some(binding) = self.bindings.remove(key) {
            self.store.close(binding.document_id);
        }
        self.histories.remove(key);
        self.synchronization_errors.remove(key);
    }
}

/// 把状态选区或折叠光标编码为公开回执使用的 u32 字节偏移。
/// 选区方向按 anchor/focus 保留，亲和性取当前光标；越过 u32 上限返回错误。
/// 此处仅做偏移编码，不自行核对文本或字素边界。
fn public_selection(
    state: &UiEditableTextState,
) -> Result<UiTextByteSelection, UiTextDocumentSessionError> {
    let (anchor, focus) = state
        .selection
        .as_ref()
        .map(|selection| (selection.anchor, selection.focus))
        .unwrap_or((state.caret.offset, state.caret.offset));
    Ok(UiTextByteSelection {
        anchor_byte: u32::try_from(anchor)
            .map_err(|_| UiTextDocumentSessionError::ByteOffsetOverflow)?,
        focus_byte: u32::try_from(focus)
            .map_err(|_| UiTextDocumentSessionError::ByteOffsetOverflow)?,
        focus_affinity: state.caret.affinity,
    })
}

#[cfg(test)]
#[path = "../session/tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "../session/tests/committed_source.rs"]
mod committed_source_tests;
