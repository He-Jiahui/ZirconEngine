use std::{
    cell::RefCell,
    collections::{btree_map, BTreeMap, BTreeSet},
    ops::{Deref, Index, IndexMut},
};

use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiSlot, UiSlotKind};

use super::{UiDirtyFlags, UiTreeError, UiTreeNode};

/// 保留式界面树的基础数据与增量修改入口；Runtime 扩展执行布局、命中和事件处理。
/// 调用方须保持映射键、节点标识、根列表及父子边一致且无环；这些入口不做整图验证。
/// 序列化保留作者数据和节点缓存；派生索引反序列化后重建，待处理集合不从序列化恢复。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiTree {
    /// 树标识数据；单独相等不证明来自同一运行会话或同一表面。
    pub tree_id: UiTreeId,
    /// 有序根节点列表；直接修改节点映射时须同步维护。
    pub roots: Vec<UiNodeId>,
    /// 跟踪可变入口的节点映射；低层插入、删除不自动修复根列表、父子边或槽。
    pub nodes: UiTreeNodes,
    /// Parent-owned placement records for each retained parent-child edge.
    /// Older serialized trees omit this field, so deserialization defaults it empty.
    #[serde(default)]
    slots: Vec<UiSlot>,
    /// Runtime edge authority rebuilt once after deserializing the flat compatibility payload.
    #[serde(skip)]
    layout_slot_authority: RefCell<UiLayoutSlotAuthority>,
    /// Runtime-only generation for parent-child and slot-order topology.
    #[serde(skip)]
    pub(crate) layout_order_generation: u64,
    /// 待消费的父级顺序变化；清理前须先交给相应增量消费者。
    #[serde(skip)]
    pub(crate) pending_layout_order_parent_ids: BTreeSet<UiNodeId>,
}

impl UiTree {
    /// 用给定标识创建空树；索引按需建立，顺序代次从零开始。
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            roots: Vec::new(),
            nodes: UiTreeNodes::default(),
            slots: Vec::new(),
            layout_slot_authority: RefCell::default(),
            layout_order_generation: 0,
            pending_layout_order_parent_ids: BTreeSet::new(),
        }
    }

    /// 插入根节点并分配绘制序号；标识已存在时直接返回，保留现有节点。
    /// 本入口将父节点设为空并设置结构失效，不检查传入的子列表。
    pub fn insert_root(&mut self, mut node: UiTreeNode) {
        if self.nodes.contains_key(&node.node_id) {
            return;
        }
        node.parent = None;
        mark_structure_dirty(&mut node);
        self.roots.push(node.node_id);
        self.nodes.insert(node.node_id, node);
    }

    /// 将节点挂到给定父节点末尾，并登记父级布局顺序和结构失效。
    ///
    /// # 参数与返回
    /// `parent_id` 必须存在；新节点标识已存在时先返回重复错误，否则缺父节点返回缺父错误。
    /// 两项检查在修改前完成；成功返回后父子关系及新节点序号已写入。
    pub fn insert_child(
        &mut self,
        parent_id: UiNodeId,
        mut node: UiTreeNode,
    ) -> Result<(), UiTreeError> {
        if self.nodes.contains_key(&node.node_id) {
            return Err(UiTreeError::DuplicateNode(node.node_id));
        }
        if !self.nodes.contains_key(&parent_id) {
            return Err(UiTreeError::MissingParent(parent_id));
        }
        self.mark_layout_order_changed(parent_id);
        let parent = self
            .nodes
            .get_mut_preserving_paint_order(&parent_id)
            .expect("validated parent must remain present");
        mark_structure_dirty(parent);
        parent.children.push(node.node_id);
        node.parent = Some(parent_id);
        mark_structure_dirty(&mut node);
        self.nodes.insert(node.node_id, node);
        Ok(())
    }

    pub fn node(&self, node_id: UiNodeId) -> Option<&UiTreeNode> {
        self.nodes.get(&node_id)
    }

    /// 此查询直接读取节点绘制序号，不是跨树或跨会话认证；公开修改该字段也会改变结果。
    /// 极值导入后的换代复用限制见序号游标分配处的问题标记。
    /// Runtime identity for one live node allocation inside this tree instance.
    ///
    /// The value stays stable across property, layout, and sibling topology changes. Removing and
    /// reinserting the same `UiNodeId` assigns a new value, so retained-node consumers can reject
    /// stale compare-and-swap keys without invalidating unrelated owners.
    pub fn node_incarnation(&self, node_id: UiNodeId) -> Option<u64> {
        self.nodes.get(&node_id).map(|node| node.paint_order)
    }

    /// 获得存在节点的可变借用并登记修改候选；调用方仍须设置对应脏域并传播布局失效。
    pub fn node_mut(&mut self, node_id: UiNodeId) -> Option<&mut UiTreeNode> {
        self.nodes.get_mut(&node_id)
    }

    /// 当前父子及槽顺序代次；变化时跳过零，回绕后不能用作永久唯一标识。
    pub fn layout_order_generation(&self) -> u64 {
        self.layout_order_generation
    }

    pub fn pending_layout_order_parent_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.pending_layout_order_parent_ids
    }

    /// 借用当前扁平槽载荷；槽可重复记录同一条边，索引不承担数值或拓扑准入。
    pub fn layout_slots(&self) -> &[UiSlot] {
        &self.slots
    }

    /// 按当前扁平位置借用槽；替换或保留过滤后，旧位置不再保证指向同一条边。
    pub fn layout_slot(&self, slot_index: usize) -> Option<&UiSlot> {
        self.slots.get(slot_index)
    }

    /// 返回该父子边在扁平载荷中的首个位置；缺失返回空，不按槽内排序键重排优先级。
    /// 首次查询可重建派生索引，后续由槽修改入口维护。
    pub fn first_layout_slot_index_for_edge(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> Option<usize> {
        self.ensure_layout_slot_authority();
        self.layout_slot_authority
            .borrow()
            .first_index(parent_id, child_id)
    }

    /// 返回该边中种类匹配的首个扁平位置；缺边或缺该种类均返回空。
    pub fn layout_slot_index_for_edge_kind(
        &self,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: UiSlotKind,
    ) -> Option<usize> {
        self.ensure_layout_slot_authority();
        self.layout_slot_authority
            .borrow()
            .index_for_kind(&self.slots, parent_id, child_id, kind)
    }

    /// 追加槽并登记其父级顺序变化；仅在索引已初始化时增量加入端点记录。
    /// 不检查端点是否存在，也不验证槽载荷的数值范围。
    pub fn push_layout_slot(&mut self, slot: UiSlot) {
        let parent_id = slot.parent_id;
        let slot_index = self.slots.len();
        self.slots.push(slot);
        self.layout_slot_authority
            .get_mut()
            .insert_if_initialized(&self.slots, slot_index);
        self.mark_layout_order_changed(parent_id);
    }

    /// 替换全部槽，立即重建端点索引，并登记新旧载荷涉及的全部父级。
    /// 原位置不再稳定；重复边和载荷值按输入保留。
    pub fn replace_layout_slots(&mut self, slots: Vec<UiSlot>) {
        let parent_ids = self
            .slots
            .iter()
            .chain(&slots)
            .map(|slot| slot.parent_id)
            .collect::<BTreeSet<_>>();
        self.slots = slots;
        self.layout_slot_authority.get_mut().rebuild(&self.slots);
        for parent_id in parent_ids {
            self.mark_layout_order_changed(parent_id);
        }
    }

    /// 原地保留满足谓词的槽；实际删除后才重建已初始化索引，并登记被删除槽的父级。
    pub fn retain_layout_slots(&mut self, mut retain: impl FnMut(&UiSlot) -> bool) {
        let mut removed_parent_ids = BTreeSet::new();
        let previous_slot_count = self.slots.len();
        self.slots.retain(|slot| {
            let retained = retain(slot);
            if !retained {
                removed_parent_ids.insert(slot.parent_id);
            }
            retained
        });
        if self.slots.len() != previous_slot_count
            && self.layout_slot_authority.get_mut().initialized
        {
            self.layout_slot_authority.get_mut().rebuild(&self.slots);
        }
        for parent_id in removed_parent_ids {
            self.mark_layout_order_changed(parent_id);
        }
    }

    /// 按需维护已初始化的端点索引；parent/child/kind/order 变化会增加父级布局顺序代次并标脏。
    /// placement、z_order 等槽载荷变化的布局或绘制失效仍由调用方按域显式传播。
    /// `slot_index` 越界时返回空且不调用闭包；存在时调用一次并在正常返回后维护索引。
    /// 闭包及槽数值不在此处校验；本入口不提供闭包 panic 时的回滚。
    pub fn mutate_layout_slot(
        &mut self,
        slot_index: usize,
        mutate: impl FnOnce(&mut UiSlot),
    ) -> Option<()> {
        let previous = self.slots.get(slot_index)?.clone();
        mutate(
            self.slots
                .get_mut(slot_index)
                .expect("validated layout slot"),
        );
        let current = self
            .slots
            .get(slot_index)
            .expect("mutated layout slot remains present");
        if previous.parent_id != current.parent_id || previous.child_id != current.child_id {
            self.layout_slot_authority.get_mut().rebind_if_initialized(
                &self.slots,
                slot_index,
                (previous.parent_id, previous.child_id),
            );
        }
        let order_changed = layout_order_slot_changed(&previous, current);
        let current_parent_id = current.parent_id;
        if order_changed {
            self.mark_layout_order_changed(previous.parent_id);
            if current_parent_id != previous.parent_id {
                self.mark_layout_order_changed(current_parent_id);
            }
        }
        Some(())
    }

    /// 推进顺序代次、登记父级并失效其测量缓存；父节点不存在时仍保留顺序变化记录。
    /// 布局来源登记不等于直接设置布局脏位，后续 Runtime 再传播对应失效。
    pub fn mark_layout_order_changed(&mut self, parent_id: UiNodeId) {
        self.layout_order_generation = next_layout_order_generation(self.layout_order_generation);
        self.pending_layout_order_parent_ids.insert(parent_id);
        self.nodes.mark_layout_dirty_source(parent_id);
    }

    /// 借用可能发生修改的节点标识集合；其中也可含已删除节点，不等于当前脏节点集合。
    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        self.nodes.pending_mutation_node_ids()
    }

    /// 借用待 Runtime 传播的布局来源集合。
    pub fn pending_layout_source_node_ids(&self) -> &BTreeSet<UiNodeId> {
        self.nodes.pending_layout_source_node_ids()
    }

    /// 清除修改、布局来源和父级顺序的待处理记录，不清除节点脏位。
    /// 调用前须同步控件索引等依赖候选集的消费者；表面重建按此顺序消费。
    pub fn clear_pending_mutation_node_ids(&mut self) {
        self.nodes.clear_pending_mutation_node_ids();
        self.pending_layout_order_parent_ids.clear();
    }

    fn ensure_layout_slot_authority(&self) {
        let needs_rebuild = {
            let authority = self.layout_slot_authority.borrow();
            !authority.initialized || authority.slot_count != self.slots.len()
        };
        if needs_rebuild {
            self.layout_slot_authority.borrow_mut().rebuild(&self.slots);
        }
    }

    #[cfg(test)]
    fn layout_slot_authority_rebuild_count(&self) -> usize {
        self.layout_slot_authority.borrow().rebuild_count
    }
}

/// 扁平槽载荷的派生端点索引；每条边的位置列表始终按载荷下标递增。
#[derive(Clone, Debug, Default)]
struct UiLayoutSlotAuthority {
    initialized: bool,
    slot_count: usize,
    indices_by_edge: BTreeMap<(UiNodeId, UiNodeId), Vec<usize>>,
    #[cfg(test)]
    rebuild_count: usize,
}

impl UiLayoutSlotAuthority {
    fn rebuild(&mut self, slots: &[UiSlot]) {
        self.indices_by_edge.clear();
        for (slot_index, slot) in slots.iter().enumerate() {
            self.indices_by_edge
                .entry((slot.parent_id, slot.child_id))
                .or_default()
                .push(slot_index);
        }
        self.slot_count = slots.len();
        self.initialized = true;
        #[cfg(test)]
        {
            self.rebuild_count = self.rebuild_count.saturating_add(1);
        }
    }

    fn insert_if_initialized(&mut self, slots: &[UiSlot], slot_index: usize) {
        if !self.initialized {
            return;
        }
        let slot = &slots[slot_index];
        let indices = self
            .indices_by_edge
            .entry((slot.parent_id, slot.child_id))
            .or_default();
        let insert_at = indices
            .binary_search(&slot_index)
            .unwrap_or_else(|index| index);
        indices.insert(insert_at, slot_index);
        self.slot_count = slots.len();
    }

    fn rebind_if_initialized(
        &mut self,
        slots: &[UiSlot],
        slot_index: usize,
        previous_edge: (UiNodeId, UiNodeId),
    ) {
        if !self.initialized {
            return;
        }
        if let Some(indices) = self.indices_by_edge.get_mut(&previous_edge) {
            indices.retain(|index| *index != slot_index);
            if indices.is_empty() {
                self.indices_by_edge.remove(&previous_edge);
            }
        }
        self.insert_if_initialized(slots, slot_index);
    }

    fn first_index(&self, parent_id: UiNodeId, child_id: UiNodeId) -> Option<usize> {
        self.indices_by_edge
            .get(&(parent_id, child_id))?
            .first()
            .copied()
    }

    fn index_for_kind(
        &self,
        slots: &[UiSlot],
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: UiSlotKind,
    ) -> Option<usize> {
        self.indices_by_edge
            .get(&(parent_id, child_id))?
            .iter()
            .copied()
            .find(|slot_index| slots[*slot_index].kind == kind)
    }
}

// 这里只比较拓扑与顺序键；放置数据、绘制层级等变化仍由调用方选择失效域。
fn layout_order_slot_changed(previous: &UiSlot, current: &UiSlot) -> bool {
    previous.parent_id != current.parent_id
        || previous.child_id != current.child_id
        || previous.kind != current.kind
        || previous.order != current.order
}

fn next_layout_order_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

/// 跟踪节点修改候选与布局来源，并按需维护七个失效域的汇总。
/// 可变借用会保守登记候选；调用方显式设置的节点脏位决定最终汇总内容。
/// 透明序列化只保存节点映射，派生集合和索引不随其写出。
/// A serialized node map whose mutable entry points retain incremental-dirty ownership.
///
/// Immutable access dereferences to `BTreeMap`; mutable access stays explicit so a caller
/// cannot change a retained node without making it a rebuild candidate.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiTreeNodes {
    nodes: BTreeMap<UiNodeId, UiTreeNode>,
    #[serde(skip)]
    mutation_node_ids: BTreeSet<UiNodeId>,
    #[serde(skip)]
    layout_source_node_ids: BTreeSet<UiNodeId>,
    #[serde(skip)]
    paint_order_cursor: PaintOrderCursor,
    /// Incremental dirty-domain aggregate. Mutable entry points enqueue only the affected node;
    /// the first query after deserialization performs the one required full index build.
    #[serde(skip)]
    dirty_index: RefCell<UiTreeDirtyIndex>,
    #[serde(skip)]
    dirty_index_pending_node_ids: RefCell<BTreeSet<UiNodeId>>,
    #[cfg(test)]
    #[serde(skip)]
    paint_order_cursor_rebuild_node_visits: usize,
}

// 非空失效节点及各域计数；域顺序必须与标志汇总和增减计数三处一致。
#[derive(Clone, Debug, Default)]
struct UiTreeDirtyIndex {
    initialized: bool,
    node_flags: BTreeMap<UiNodeId, UiDirtyFlags>,
    domain_counts: [usize; 7],
    #[cfg(test)]
    node_visits: usize,
}

impl UiTreeDirtyIndex {
    fn update(&mut self, node_id: UiNodeId, next: UiDirtyFlags) {
        let previous = self.node_flags.get(&node_id).copied().unwrap_or_default();
        if previous == next {
            return;
        }
        self.remove_counts(previous);
        if next.any() {
            self.node_flags.insert(node_id, next);
            self.add_counts(next);
        } else {
            self.node_flags.remove(&node_id);
        }
    }

    fn flags(&self) -> UiDirtyFlags {
        UiDirtyFlags {
            layout: self.domain_counts[0] != 0,
            hit_test: self.domain_counts[1] != 0,
            render: self.domain_counts[2] != 0,
            style: self.domain_counts[3] != 0,
            text: self.domain_counts[4] != 0,
            input: self.domain_counts[5] != 0,
            visible_range: self.domain_counts[6] != 0,
        }
    }

    fn add_counts(&mut self, flags: UiDirtyFlags) {
        for (count, enabled) in self.domain_counts.iter_mut().zip([
            flags.layout,
            flags.hit_test,
            flags.render,
            flags.style,
            flags.text,
            flags.input,
            flags.visible_range,
        ]) {
            if enabled {
                *count = count.saturating_add(1);
            }
        }
    }

    fn remove_counts(&mut self, flags: UiDirtyFlags) {
        for (count, enabled) in self.domain_counts.iter_mut().zip([
            flags.layout,
            flags.hit_test,
            flags.render,
            flags.style,
            flags.text,
            flags.input,
            flags.visible_range,
        ]) {
            if enabled {
                *count = count.saturating_sub(1);
            }
        }
    }
}

impl UiTreeNodes {
    /// 存在时登记候选并使序号游标失效，再返回可变借用；不存在不登记。
    /// 因调用方可能修改绘制序号，下次分配前会扫描当前节点并保留历史高水位。
    pub fn get_mut(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mark_mutation(*node_id);
            self.paint_order_cursor.invalidate();
        }
        self.nodes.get_mut(node_id)
    }

    /// 对存在节点登记布局来源并清除测量有效位；不直接设置该节点的布局脏位。
    pub fn mark_layout_dirty_source(&mut self, node_id: UiNodeId) {
        if self.nodes.contains_key(&node_id) {
            self.mark_mutation(node_id);
            self.layout_source_node_ids.insert(node_id);
            if let Some(node) = self.nodes.get_mut(&node_id) {
                node.layout_cache.invalidate_measure();
            }
        }
    }

    /// 分配新绘制序号后写入映射，返回被替换的旧节点；传入节点原序号会被覆盖。
    /// 调用方须让键与节点标识一致，并另行维护树的拓扑和槽。
    pub fn insert(&mut self, node_id: UiNodeId, mut node: UiTreeNode) -> Option<UiTreeNode> {
        self.mark_mutation(node_id);
        // Keep allocation identity and paint order on the same monotonic insertion serial.
        node.paint_order = self.allocate_paint_order();
        self.nodes.insert(node_id, node)
    }

    /// 删除键并登记候选供索引移除旧记录；缺失时也会登记，返回被删除节点或空。
    pub fn remove(&mut self, node_id: &UiNodeId) -> Option<UiTreeNode> {
        self.mark_mutation(*node_id);
        self.layout_source_node_ids.remove(node_id);
        self.nodes.remove(node_id)
    }

    /// 在交出可变迭代器前登记全部当前节点并失效序号游标；返回借用受映射借用期约束。
    pub fn iter_mut(&mut self) -> btree_map::IterMut<'_, UiNodeId, UiTreeNode> {
        self.track_all_nodes();
        self.nodes.iter_mut()
    }

    /// 与可变键值迭代相同，先保守登记全部节点，再交出可变值迭代器。
    pub fn values_mut(&mut self) -> btree_map::ValuesMut<'_, UiNodeId, UiTreeNode> {
        self.track_all_nodes();
        self.nodes.values_mut()
    }

    /// 谓词可修改节点，故先登记全部当前节点；完成后仅为保留节点留下布局来源。
    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&UiNodeId, &mut UiTreeNode) -> bool,
    {
        self.track_all_nodes();
        self.nodes.retain(predicate);
    }

    /// 登记并清空当前映射及布局来源；保留退役序号高水位，根列表和槽由外层维护。
    pub fn clear(&mut self) {
        self.track_all_nodes();
        self.nodes.clear();
        self.layout_source_node_ids.clear();
    }

    pub fn pending_mutation_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.mutation_node_ids
    }

    pub fn pending_layout_source_node_ids(&self) -> &BTreeSet<UiNodeId> {
        &self.layout_source_node_ids
    }

    /// Returns the aggregate dirty domain mask, refreshing only nodes changed since the last
    /// query (or all nodes once for a newly deserialized tree).
    pub fn dirty_flags(&self) -> UiDirtyFlags {
        self.refresh_dirty_index();
        self.dirty_index.borrow().flags()
    }

    /// Returns the number of nodes with at least one effective dirty domain.
    pub fn dirty_node_count(&self) -> usize {
        self.refresh_dirty_index();
        self.dirty_index.borrow().node_flags.len()
    }

    /// Returns the currently dirty node IDs from the aggregate index.
    pub fn dirty_node_ids(&self) -> BTreeSet<UiNodeId> {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .keys()
            .copied()
            .collect()
    }

    /// Extends a caller-owned set with dirty node IDs without allocating an intermediate set.
    /// This is the hot-path form used when a surface already has a candidate collection ready.
    /// 追加到调用方集合，不清空原有元素；集合按标识去重。
    pub fn extend_dirty_node_ids(&self, target: &mut BTreeSet<UiNodeId>) {
        self.refresh_dirty_index();
        target.extend(self.dirty_index.borrow().node_flags.keys().copied());
    }

    /// Extends a caller-owned entry buffer directly from the dirty index, avoiding a temporary
    /// vector when a consumer already owns the destination used for its summary or delta.
    /// 追加到调用方缓冲区，不清空或去重已有条目；追加条目是标识与失效位的值拷贝。
    pub fn extend_dirty_node_entries(&self, target: &mut Vec<(UiNodeId, UiDirtyFlags)>) {
        self.refresh_dirty_index();
        let index = self.dirty_index.borrow();
        target.reserve(index.node_flags.len());
        target.extend(
            index
                .node_flags
                .iter()
                .map(|(node_id, flags)| (*node_id, *flags)),
        );
    }

    /// Returns the effective dirty domains for every currently dirty node. The index refresh is
    /// shared with `dirty_flags`/`dirty_node_count`, so repeated idle queries remain O(1) apart
    /// from the caller's requested result materialization.
    pub fn dirty_node_entries(&self) -> Vec<(UiNodeId, UiDirtyFlags)> {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .iter()
            .map(|(node_id, flags)| (*node_id, *flags))
            .collect()
    }

    /// Returns one node's effective dirty domains without scanning unrelated nodes.
    /// 节点不存在或没有有效脏域时都返回全空标志；查询不消费脏位。
    pub fn dirty_node_flags(&self, node_id: &UiNodeId) -> UiDirtyFlags {
        self.refresh_dirty_index();
        self.dirty_index
            .borrow()
            .node_flags
            .get(node_id)
            .copied()
            .unwrap_or_default()
    }

    /// 清理候选前先刷新已初始化汇总；未初始化时延后首次全量扫描。
    /// 节点脏位保留，布局来源一并清除；其他候选集消费者须先完成同步。
    pub fn clear_pending_mutation_node_ids(&mut self) {
        // Reconcile mutable references before dropping their candidate IDs. This keeps the
        // aggregate correct for callers that mutate a node and clear bookkeeping directly. A
        // newly constructed/deserialized map can defer its first full index build until a dirty
        // query; clearing mutation bookkeeping alone must not force that O(N) scan.
        let index_initialized = self.dirty_index.borrow().initialized;
        if index_initialized {
            self.refresh_dirty_index();
        }
        self.mutation_node_ids.clear();
        self.dirty_index_pending_node_ids.borrow_mut().clear();
        self.layout_source_node_ids.clear();
    }

    fn track_all_nodes(&mut self) {
        let mut dirty_index_pending_node_ids = self.dirty_index_pending_node_ids.borrow_mut();
        for node_id in self.nodes.keys().copied() {
            self.mutation_node_ids.insert(node_id);
            dirty_index_pending_node_ids.insert(node_id);
        }
        self.paint_order_cursor.invalidate();
    }

    // 父节点仅追加子边时使用此内部借用，避免把未改动的绘制序号当作游标失效来源。
    fn get_mut_preserving_paint_order(&mut self, node_id: &UiNodeId) -> Option<&mut UiTreeNode> {
        if self.nodes.contains_key(node_id) {
            self.mark_mutation(*node_id);
        }
        self.nodes.get_mut(node_id)
    }

    fn mark_mutation(&mut self, node_id: UiNodeId) {
        self.mutation_node_ids.insert(node_id);
        self.dirty_index_pending_node_ids
            .borrow_mut()
            .insert(node_id);
    }

    // 首次查询全量建立索引；此后只刷新待处理标识，已删除节点以空标志移除旧计数。
    fn refresh_dirty_index(&self) {
        let pending = {
            let mut pending = self.dirty_index_pending_node_ids.borrow_mut();
            let is_initialized = self.dirty_index.borrow().initialized;
            if pending.is_empty() && is_initialized {
                return;
            }
            std::mem::take(&mut *pending)
        };

        let mut index = self.dirty_index.borrow_mut();
        if !index.initialized {
            index.node_flags.clear();
            index.domain_counts = [0; 7];
            for (node_id, node) in &self.nodes {
                #[cfg(test)]
                {
                    index.node_visits = index.node_visits.saturating_add(1);
                }
                index.update(*node_id, effective_dirty_flags(node));
            }
            index.initialized = true;
            return;
        }

        for node_id in pending {
            #[cfg(test)]
            {
                index.node_visits = index.node_visits.saturating_add(1);
            }
            let dirty = self
                .nodes
                .get(&node_id)
                .map(effective_dirty_flags)
                .unwrap_or_default();
            index.update(node_id, dirty);
        }
    }

    // 游标失效时才重扫，包括反序列化初态与可变入口；重建仍保留已退役的历史高水位。
    fn allocate_paint_order(&mut self) -> u64 {
        if !self.paint_order_cursor.is_valid() {
            #[cfg(test)]
            {
                self.paint_order_cursor_rebuild_node_visits += self.nodes.len();
            }
            self.paint_order_cursor
                .rebuild(self.nodes.values().map(|node| node.paint_order));
        }
        self.paint_order_cursor.allocate()
    }

    #[cfg(test)]
    fn dirty_index_node_visits(&self) -> usize {
        self.dirty_index.borrow().node_visits
    }

    #[cfg(test)]
    fn paint_order_cursor_rebuild_node_visits(&self) -> usize {
        self.paint_order_cursor_rebuild_node_visits
    }
}

impl Default for UiTreeNodes {
    fn default() -> Self {
        Self {
            nodes: BTreeMap::new(),
            mutation_node_ids: BTreeSet::new(),
            layout_source_node_ids: BTreeSet::new(),
            paint_order_cursor: PaintOrderCursor::new(),
            dirty_index: RefCell::new(UiTreeDirtyIndex::default()),
            dirty_index_pending_node_ids: RefCell::new(BTreeSet::new()),
            #[cfg(test)]
            paint_order_cursor_rebuild_node_visits: 0,
        }
    }
}

/// 只提供共享映射解引用；可变访问保留在显式入口，以便跟踪候选。
impl Deref for UiTreeNodes {
    type Target = BTreeMap<UiNodeId, UiTreeNode>;

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl Index<&UiNodeId> for UiTreeNodes {
    type Output = UiTreeNode;

    fn index(&self, node_id: &UiNodeId) -> &Self::Output {
        &self.nodes[node_id]
    }
}

/// 按键可变索引先登记候选并失效游标；键不存在时按映射索引惯例触发 panic。
impl IndexMut<&UiNodeId> for UiTreeNodes {
    fn index_mut(&mut self, node_id: &UiNodeId) -> &mut Self::Output {
        self.mark_mutation(*node_id);
        self.paint_order_cursor.invalidate();
        self.nodes.get_mut(node_id).expect("no entry found for key")
    }
}

/// 相等性仅比较节点数据，不比较待处理集合、派生索引或序号游标。
impl PartialEq for UiTreeNodes {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

/// 导入节点映射并保留原绘制序号，由已有最大值建立分配游标；不检查图或序号唯一性。
/// 修改候选为空，脏域索引延后建立；导入不等于清除节点已携带的脏位。
impl From<BTreeMap<UiNodeId, UiTreeNode>> for UiTreeNodes {
    fn from(nodes: BTreeMap<UiNodeId, UiTreeNode>) -> Self {
        let mut paint_order_cursor = PaintOrderCursor::new();
        paint_order_cursor.rebuild(nodes.values().map(|node| node.paint_order));
        Self {
            nodes,
            mutation_node_ids: BTreeSet::new(),
            layout_source_node_ids: BTreeSet::new(),
            paint_order_cursor,
            dirty_index: RefCell::new(UiTreeDirtyIndex::default()),
            dirty_index_pending_node_ids: RefCell::new(BTreeSet::new()),
            #[cfg(test)]
            paint_order_cursor_rebuild_node_visits: 0,
        }
    }
}

impl FromIterator<(UiNodeId, UiTreeNode)> for UiTreeNodes {
    fn from_iter<T: IntoIterator<Item = (UiNodeId, UiTreeNode)>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<BTreeMap<_, _>>())
    }
}

impl<'a> IntoIterator for &'a UiTreeNodes {
    type Item = (&'a UiNodeId, &'a UiTreeNode);
    type IntoIter = btree_map::Iter<'a, UiNodeId, UiTreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

/// 交出可变迭代器前登记全部节点；共享迭代则只借用数据。
impl<'a> IntoIterator for &'a mut UiTreeNodes {
    type Item = (&'a UiNodeId, &'a mut UiTreeNode);
    type IntoIter = btree_map::IterMut<'a, UiNodeId, UiTreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.track_all_nodes();
        self.nodes.iter_mut()
    }
}

// 绘制序号的历史高水位游标；失效后扫描现有节点补充观察值，不降低退役高水位。
#[derive(Clone, Copy, Debug, Default)]
struct PaintOrderCursor {
    next: u64,
    valid: bool,
}

impl PaintOrderCursor {
    const fn new() -> Self {
        Self {
            next: 0,
            valid: true,
        }
    }

    const fn is_valid(self) -> bool {
        self.valid
    }

    fn allocate(&mut self) -> u64 {
        debug_assert!(self.valid, "paint-order cursor must be rebuilt before use");
        let next = self.next;
        // BUG: [CR-ZCA-R041-0001] 导入绘制序号为 u64::MAX 的节点后，游标也饱和在该值；
        // 移除并重插相同节点标识仍分配 MAX，导致实例标识无法区分这次换代。
        // 可由公开节点映射导入路径触发，无需耗尽正常插入；应补极值导入与重插边界夹具。
        self.next = next.saturating_add(1);
        next
    }

    // 只扫描当前节点会丢失已移除节点的序号；保留历史高水位可避免重新插入相同 UiNodeId 时复用退役的 node_incarnation。
    fn rebuild(&mut self, paint_orders: impl Iterator<Item = u64>) {
        let observed_next = paint_orders
            .max()
            .map_or(0, |paint_order| paint_order.saturating_add(1));
        self.next = self.next.max(observed_next);
        self.valid = true;
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }
}

/// 树相等性比较标识、根、节点与槽载荷；不比较顺序代次或待处理集合。
impl PartialEq for UiTree {
    fn eq(&self, other: &Self) -> bool {
        self.tree_id == other.tree_id
            && self.roots == other.roots
            && self.nodes == other.nodes
            && self.slots == other.slots
    }
}

fn mark_structure_dirty(node: &mut UiTreeNode) {
    node.dirty.layout = true;
    node.dirty.hit_test = true;
    node.dirty.render = true;
    node.dirty.input = true;
}

// 兼容旧总脏位仅补命中、绘制和输入域；布局、样式、文本及可见范围仍读显式域。
fn effective_dirty_flags(node: &UiTreeNode) -> UiDirtyFlags {
    let mut dirty = node.dirty;
    if node.state_flags.dirty {
        dirty.hit_test = true;
        dirty.render = true;
        dirty.input = true;
    }
    dirty
}

#[cfg(test)]
#[path = "tests/ui_tree.rs"]
mod tests;
