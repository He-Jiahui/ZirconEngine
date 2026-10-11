use std::cmp::Ordering;
use std::collections::{hash_map::RandomState, HashMap};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;

use crate::{ResourceId, ResourceKind, ResourceRecord, ResourceState};

/// 管理投影的 ID 索引分片数；生产与查询必须使用同一哈希 authority，并保持该值为二的幂。
pub(crate) const RESOURCE_MANAGEMENT_ID_SHARD_COUNT: usize = 1_024;
/// 管理投影的定位符索引分片数；键是规范显示文本，生产与查询共享同一定位符哈希 authority。
pub(crate) const RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT: usize = 1_024;
/// 有序存储页的行数上限；平衡分组可生成较小的页，此值不是查询分页的 `limit`。
pub(crate) const RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS: usize = 256;

/// 为同一管理投影链保留 ID 分片、ID 映射和定位符分片的随机哈希种子。
///
/// # 约束与生命周期
///
/// 增量发布保留此 authority，才能继续复用旧分片；独立默认构造会产生新的随机状态。
/// 哈希分片只加速查找，查询顺序由有序页确定，不由哈希遍历顺序确定。
///
/// # 参考
///
/// [`ResourceManagementGeneration`]、[`ResourceManagementIdMap`]。
#[derive(Clone, Debug)]
pub(crate) struct ResourceManagementHashAuthority {
    id_shard_hasher: RandomState,
    id_map_hasher: RandomState,
    locator_shard_hasher: RandomState,
}

impl Default for ResourceManagementHashAuthority {
    fn default() -> Self {
        Self {
            id_shard_hasher: RandomState::new(),
            id_map_hasher: RandomState::new(),
            locator_shard_hasher: RandomState::new(),
        }
    }
}

impl ResourceManagementHashAuthority {
    fn id_shard_index(&self, id: ResourceId) -> usize {
        debug_assert!(RESOURCE_MANAGEMENT_ID_SHARD_COUNT.is_power_of_two());
        self.id_shard_hasher.hash_one(id) as usize & (RESOURCE_MANAGEMENT_ID_SHARD_COUNT - 1)
    }

    fn id_map_with_capacity<V>(&self, capacity: usize) -> ResourceManagementIdMap<V> {
        ResourceManagementIdMap::with_capacity_and_hasher(capacity, self.id_map_hasher.clone())
    }

    fn locator_shard_index(&self, locator: &str) -> usize {
        debug_assert!(RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT.is_power_of_two());
        self.locator_shard_hasher.hash_one(locator) as usize
            & (RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT - 1)
    }
}

pub(crate) type ResourceManagementIdMap<V> = HashMap<ResourceId, V, RandomState>;

/// Observation-only counters for one management publication.
///
/// 该观察值描述一条管理投影链的发布次数。
///
/// # 约束与生命周期
///
/// 默认管理投影从零开始；正常生产器只在管理行出现净差异并发布新 generation 时饱和加一。
/// 该计数属于一条投影发布链，不能代替 [`ResourceManagementGenerationIdentity`] 判断快照是否相同。
/// 同样的计数可以属于不同对象，管理行未变的提交也可以保留原计数。
///
/// # 参考
///
/// [`ResourceManagementGeneration::diagnostics`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementGenerationDiagnostics {
    pub publication_count: u64,
}

/// Compact immutable resource row shared by runtime and authoring projections.
///
/// 此管理行保存目录元数据的紧凑值快照。
///
/// # 约束与生命周期
///
/// 管理器发布后通过 `Arc` 共享这些字段；它们表示捕获时的目录元数据，后续发布不会刷新旧行。
/// 该行只保存诊断数量，不保存诊断内容、依赖或载荷；相同数量的诊断内容变化可保留同一管理行。
/// `state` 是目录阶段；即使为 `Ready`，也不证明运行时载荷已驻留。
/// [`PartialEq`] 比较字段值；要判断共享发布身份，使用 [`ResourceManagementRowIdentity`]。
///
/// # 参考
///
/// [`crate::ResourceRecord`]、[`ResourceManagementGeneration::row_by_id`]。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceManagementRow {
    pub id: ResourceId,
    pub kind: ResourceKind,
    /// 主定位符的完整显示文本；含 scheme 与可选标签，按整个字符串排序并共享其 `Arc`。
    pub primary_locator: Arc<str>,
    /// 捕获记录的修订值；该字段不是 generation 发布次数，也不证明当前载荷驻留。
    pub revision: u64,
    pub state: ResourceState,
    pub diagnostic_count: usize,
}

impl ResourceManagementRow {
    #[cfg(test)]
    pub(crate) fn from_record(record: &ResourceRecord) -> Self {
        Self::from_record_reusing_identity(record, None)
    }

    // 正常生产器传入同一 ID 的旧行；这里只复用相同显示文本的分配，
    // 不校验 previous.id，也不复用整个管理行的发布身份。
    pub(crate) fn from_record_reusing_identity(
        record: &ResourceRecord,
        previous: Option<&Self>,
    ) -> Self {
        let primary_locator = previous
            .filter(|row| {
                record
                    .primary_locator
                    .matches_display(row.primary_locator.as_ref())
            })
            .map(|row| Arc::clone(&row.primary_locator))
            .unwrap_or_else(|| Arc::from(record.primary_locator.to_string()));
        Self {
            id: record.id,
            kind: record.kind,
            primary_locator,
            revision: record.revision,
            state: record.state,
            diagnostic_count: record.diagnostics.len(),
        }
    }
}

/// Process-local identity of one immutable management row publication.
///
/// 缓存可用此身份判断是否仍引用同一个行发布。
///
/// # 约束与生命周期
///
/// 包装值持有行的 `Arc`；克隆保留同一分配，`Eq` 与 `Hash` 按该分配的指针身份工作。
/// 不同分配即使字段值相同也有不同身份；未变化行可在多个 generation 间共享同一身份。
/// 适合进程内缓存复用，不是持久化 ID、内容摘要或可跨进程比较的版本号。
///
/// # 参考
///
/// [`ResourceManagementGeneration::row_identity_by_id`]、[`Self::row`]。
#[derive(Clone)]
pub struct ResourceManagementRowIdentity(Arc<ResourceManagementRow>);

impl ResourceManagementRowIdentity {
    fn new(row: Arc<ResourceManagementRow>) -> Self {
        Self(row)
    }

    pub fn row(&self) -> &ResourceManagementRow {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceManagementRowIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ResourceManagementRowIdentity")
            .field(&self.0.id)
            .finish()
    }
}

impl PartialEq for ResourceManagementRowIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceManagementRowIdentity {}

impl Hash for ResourceManagementRowIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
    }
}

/// 保存某一资源种类在管理快照中的总行数和各目录阶段行数。
///
/// # 约束与生命周期
///
/// 正常生产器维护 `total_count` 为四个阶段计数之和；计数单位均为行，不是事件或载荷。
/// 这是可复制的值快照；调用方自行构造字段时须自行维护该不变量。
///
/// # 参考
///
/// [`ResourceManagementSummary::kind`]、[`crate::ResourceState`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementKindSummary {
    pub total_count: usize,
    pub pending_count: usize,
    pub ready_count: usize,
    pub error_count: usize,
    pub reloading_count: usize,
}

impl ResourceManagementKindSummary {
    fn count_for_state(self, state: ResourceState) -> usize {
        match state {
            ResourceState::Pending => self.pending_count,
            ResourceState::Ready => self.ready_count,
            ResourceState::Error => self.error_count,
            ResourceState::Reloading => self.reloading_count,
        }
    }

    pub(crate) fn add(&mut self, state: ResourceState) {
        self.total_count += 1;
        match state {
            ResourceState::Pending => self.pending_count += 1,
            ResourceState::Ready => self.ready_count += 1,
            ResourceState::Error => self.error_count += 1,
            ResourceState::Reloading => self.reloading_count += 1,
        }
    }

    pub(crate) fn remove(&mut self, state: ResourceState) {
        self.total_count = self.total_count.saturating_sub(1);
        match state {
            ResourceState::Pending => self.pending_count = self.pending_count.saturating_sub(1),
            ResourceState::Ready => self.ready_count = self.ready_count.saturating_sub(1),
            ResourceState::Error => self.error_count = self.error_count.saturating_sub(1),
            ResourceState::Reloading => {
                self.reloading_count = self.reloading_count.saturating_sub(1)
            }
        }
    }
}

/// 保存一份管理 generation 的总行数和按种类、目录阶段的行数汇总。
///
/// # 约束与生命周期
///
/// 生产器随行增删或种类、阶段变化维护汇总；查询用它计算精确的过滤后总数，无需扫描行。
/// 该汇总与拥有它的 generation 配对，不能与后来从管理器读取的页混用。
///
/// # 参考
///
/// [`ResourceManagementGeneration::summary`]、[`ResourceManagementQuery`]。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementSummary {
    total_count: usize,
    by_kind: HashMap<ResourceKind, ResourceManagementKindSummary>,
}

impl ResourceManagementSummary {
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    pub fn kind(&self, kind: ResourceKind) -> ResourceManagementKindSummary {
        self.by_kind.get(&kind).copied().unwrap_or_default()
    }

    fn matching_count(&self, query: ResourceManagementQuery) -> usize {
        match (query.kind, query.state) {
            (Some(kind), Some(state)) => self.kind(kind).count_for_state(state),
            (Some(kind), None) => self.kind(kind).total_count,
            (None, Some(state)) => self
                .by_kind
                .values()
                .map(|summary| summary.count_for_state(state))
                .sum(),
            (None, None) => self.total_count,
        }
    }

    pub(crate) fn add(&mut self, row: &ResourceManagementRow) {
        self.total_count += 1;
        self.by_kind.entry(row.kind).or_default().add(row.state);
    }

    // 移除必须使用已计入汇总的旧行，且只能扣减一次；饱和减法不校验成员关系。
    pub(crate) fn remove(&mut self, row: &ResourceManagementRow) {
        self.total_count = self.total_count.saturating_sub(1);
        if let Some(summary) = self.by_kind.get_mut(&row.kind) {
            summary.remove(row.state);
            if summary.total_count == 0 {
                self.by_kind.remove(&row.kind);
            }
        }
    }
}

/// 为管理快照的页和扫描指定种类、目录阶段的精确过滤条件。
///
/// # 约束与生命周期
///
/// 两个 `Some` 条件取交集；每个 `None` 放开该维度，默认值匹配所有行。
/// `state` 指 [`crate::ResourceState`] 的目录阶段，不是运行时载荷驻留状态。
///
/// # 参考
///
/// [`ResourceManagementGeneration::page`]、[`ResourceManagementGeneration::scan`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementQuery {
    pub kind: Option<ResourceKind>,
    pub state: Option<ResourceState>,
}

impl ResourceManagementQuery {
    fn matches(self, row: &ResourceManagementRow) -> bool {
        self.kind.is_none_or(|kind| row.kind == kind)
            && self.state.is_none_or(|state| row.state == state)
    }
}

/// 保存一次管理查询的分页结果以及产生它的精确 generation 身份。
///
/// # 约束与生命周期
///
/// 正常由 [`ResourceManagementGeneration::page`] 构造；`total_matching_count` 是过滤后的全快照总数。
/// `rows` 是选出的连续匹配区间，按完整定位符显示文本、再按 ID 排列；零上限或越界偏移可使它为空。
/// 结果新建共享切片并克隆选中行的 `Arc`，不复制行字段；它不是某个内部存储页的原切片。
/// 克隆页面共享切片与快照；后续管理器发布不会修改这些对象。公开字段允许调用方自行构造，
/// 上述配对关系只由正常查询构造路径保证。
///
/// # 参考
///
/// [`ResourceManagementGenerationIdentity`]、[`ResourceManagementRow`]。
#[derive(Clone)]
pub struct ResourceManagementPage {
    pub generation: ResourceManagementGenerationIdentity,
    pub total_matching_count: usize,
    pub rows: Arc<[Arc<ResourceManagementRow>]>,
}

impl fmt::Debug for ResourceManagementPage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceManagementPage")
            .field("generation", &self.generation)
            .field("total_matching_count", &self.total_matching_count)
            .field("rows", &self.rows)
            .finish()
    }
}

/// Query-local work counters consumed by crate-owned profiling tests and benchmarks.
///
/// 分页指标只描述当前查询已实际执行的工作。
///
/// # 约束与生命周期
///
/// 只统计这次分页实际执行的工作；达到 `limit` 后提前结束，未访问的尾部不会计入。
/// `shard_candidate_checks` 在当前实现计有序存储页，不计 ID 或定位符哈希分片。
/// `candidate_rows` 包括为跳过 `offset` 而检查的匹配行；`rows_returned` 仅计结果行。
/// 零 `limit` 的分页不访问存储页；这些计数不用于快照身份或业务验收。
///
/// # 参考
///
/// [`ResourceManagementGeneration::page_with_profile_metrics`]。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ResourceManagementPageProfileMetrics {
    pub(crate) shard_candidate_checks: u64,
    pub(crate) filtered_rows_skipped: u64,
    pub(crate) candidate_rows: u64,
    pub(crate) rows_returned: u64,
}

/// Incremental stable-order scan over immutable globally ordered pages.
///
/// 扫描保留捕获快照，并按调用次数推进独立游标。
///
/// # 约束与生命周期
///
/// 由 [`ResourceManagementGeneration::scan`] 创建，持有一个 generation 的 `Arc`、查询和独立游标。
/// 每次 [`Self::next_row`] 从完整定位符显示文本、再按 ID 的顺序返回下一匹配行；
/// 后续发布不改变扫描源。克隆复制当前游标与计数，两个副本以后独立推进。
/// [`Self::is_complete`] 只判断匹配行已全部返回；启用 profiling 后，需要让 `next_row`
/// 返回 `None` 才把尾部被过滤的行计入诊断。
#[derive(Clone, Debug)]
pub struct ResourceManagementScan {
    generation: Arc<ResourceManagementGeneration>,
    query: ResourceManagementQuery,
    page_index: usize,
    row_index: usize,
    yielded_count: usize,
    total_matching_count: usize,
    #[cfg(feature = "profiling")]
    profile_metrics: ResourceManagementScanDiagnostics,
}

/// Read-only query-local work counters for a profiling-enabled immutable generation scan.
/// Capturing this value does not advance the cursor, acquire a manager lock, or clone rows.
///
/// 扫描诊断区分访问存储页、过滤行与返回行的累计窗口。
///
/// # 约束与生命周期
///
/// 这些值仅累计到所属扫描当前游标位置，并使用饱和加法；复制后不会随扫描继续变化。
/// `shard_candidate_checks` 在当前实现表示进入的有序存储页数，两个行计数区分过滤与返回。
/// 最后一个匹配行返回后可能还有被过滤的尾部；完整工作诊断须继续调用扫描直到 `None`。
/// 这些观察值不表示目录发布次数、跨查询的总工作量或资源载荷状态。
///
/// # 参考
///
/// [`ResourceManagementScan::diagnostics`]、[`ResourceManagementScan::next_row`]。
#[cfg(feature = "profiling")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceManagementScanDiagnostics {
    shard_candidate_checks: u64,
    filtered_rows_skipped: u64,
    rows_emitted: u64,
}

#[cfg(feature = "profiling")]
impl ResourceManagementScanDiagnostics {
    /// Ordered page candidates entered by this scan so far.
    pub fn shard_candidate_checks(&self) -> u64 {
        self.shard_candidate_checks
    }

    /// Rows visited and rejected by this scan's query so far.
    pub fn filtered_rows_skipped(&self) -> u64 {
        self.filtered_rows_skipped
    }

    /// Matching rows emitted by this scan so far.
    pub fn rows_emitted(&self) -> u64 {
        self.rows_emitted
    }
}

impl ResourceManagementScan {
    /// 推进此快照的游标，取得下一条满足查询的共享管理行。
    ///
    /// # 返回
    ///
    /// 有匹配行时返回其 `Arc` 克隆；遍历所有有序页后返回 `None`，以后调用仍返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 每个游标按完整定位符显示文本、再按 ID 遍历，每条匹配行最多返回一次。
    /// 调用会跨过不匹配行并更新该游标的计数，不重新读取管理器，也不复制行字段。
    /// 启用 profiling 时，即使 [`Self::is_complete`] 已为真，也应继续调用至 `None`，
    /// 才能观察最后匹配行之后被过滤的行；丢弃扫描仅释放其持有的快照引用。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementGeneration::scan`]、[`Self::total_matching_count`]。
    pub fn next_row(&mut self) -> Option<Arc<ResourceManagementRow>> {
        while let Some(page) = self.generation.ordered_pages.get(self.page_index) {
            if self.row_index == 0 {
                #[cfg(feature = "profiling")]
                {
                    self.profile_metrics.shard_candidate_checks = self
                        .profile_metrics
                        .shard_candidate_checks
                        .saturating_add(1);
                }
            }
            while let Some(row) = page.get(self.row_index) {
                self.row_index += 1;
                if !self.query.matches(row) {
                    #[cfg(feature = "profiling")]
                    {
                        self.profile_metrics.filtered_rows_skipped =
                            self.profile_metrics.filtered_rows_skipped.saturating_add(1);
                    }
                    continue;
                }
                self.yielded_count = self.yielded_count.saturating_add(1);
                #[cfg(feature = "profiling")]
                {
                    self.profile_metrics.rows_emitted =
                        self.profile_metrics.rows_emitted.saturating_add(1);
                }
                return Some(Arc::clone(row));
            }
            self.page_index += 1;
            self.row_index = 0;
        }
        None
    }

    /// 匹配行已全部返回即完成；启用 profiling 时，仍须让 next_row 返回 None 才包含尾部过滤行的诊断计数。
    ///
    /// 完成标志只反映匹配行的返回进度。
    ///
    /// # 返回
    ///
    /// 已返回的匹配行数达到本快照的匹配总数时返回 `true`；零匹配的扫描创建后即为真。
    ///
    /// # 约束与生命周期
    ///
    /// 此读取不推进游标，也不证明所有存储行已经访问。需要完整 profiling 工作计数时，
    /// 继续调用 [`Self::next_row`] 直到它返回 `None`。
    ///
    /// # 参考
    ///
    /// [`Self::total_matching_count`]。
    pub fn is_complete(&self) -> bool {
        self.yielded_count >= self.total_matching_count
    }

    pub fn total_matching_count(&self) -> usize {
        self.total_matching_count
    }

    #[cfg(feature = "profiling")]
    /// Captures work performed so far; exhausting next_row includes trailing filtered rows.
    ///
    /// 诊断副本保留本次读取时已经发生的扫描工作。
    ///
    /// # 返回
    ///
    /// 返回当前查询局部计数的值副本，后续游标推进不会修改已返回的副本。
    ///
    /// # 约束与生命周期
    ///
    /// 只在 profiling 配置中提供；完整计数需先让 [`Self::next_row`] 返回 `None`。
    /// 读取不推进扫描、取得管理器锁或克隆行。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementScanDiagnostics`]。
    pub fn diagnostics(&self) -> ResourceManagementScanDiagnostics {
        self.profile_metrics
    }
}

/// 为一个管理 generation 保存单个 ID 哈希分片的不可变查找映射。
///
/// # 约束与生命周期
///
/// 生产器构造或替换映射后通过 `Arc` 发布；条目须使用拥有 generation 的 authority 选择分片。
/// 值与有序页共享同一行 `Arc`，未触及分片可跨发布共享；本映射不承担排序。
///
/// # 参考
///
/// [`ResourceManagementGeneration::row_by_id`]。
#[derive(Clone, Debug)]
pub(crate) struct ResourceManagementIdShard {
    by_id: ResourceManagementIdMap<Arc<ResourceManagementRow>>,
}

impl ResourceManagementIdShard {
    pub(crate) fn from_entries(by_id: ResourceManagementIdMap<Arc<ResourceManagementRow>>) -> Self {
        Self { by_id }
    }

    pub(crate) fn entries(&self) -> &ResourceManagementIdMap<Arc<ResourceManagementRow>> {
        &self.by_id
    }

    fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.by_id.get(&id).cloned()
    }
}

/// 为一个管理 generation 保存主定位符显示文本到资源 ID 的单个查找分片。
///
/// # 约束与生命周期
///
/// 条目只覆盖主定位符；生产器使用该 generation 的定位符 authority 选择分片。
/// 未变化映射可跨发布共享，读取命中后仍通过 ID 索引取得行；哈希遍历不规定查询顺序。
///
/// # 参考
///
/// [`ResourceManagementGeneration::row_by_locator`]。
#[derive(Clone, Debug, Default)]
pub(crate) struct ResourceManagementLocatorShard {
    by_locator: HashMap<Arc<str>, ResourceId>,
}

impl ResourceManagementLocatorShard {
    pub(crate) fn from_entries(by_locator: HashMap<Arc<str>, ResourceId>) -> Self {
        Self { by_locator }
    }

    pub(crate) fn entries(&self) -> &HashMap<Arc<str>, ResourceId> {
        &self.by_locator
    }

    fn id_by_locator(&self, locator: &str) -> Option<ResourceId> {
        self.by_locator.get(locator).copied()
    }
}

/// Immutable resource projection with independent lookup and canonical-order storage.
///
/// 调用方可保留管理快照独立执行查找、分页和扫描。
///
/// # 约束与生命周期
///
/// 从 [`crate::ResourceManager::management_generation`] 或配对的
/// [`crate::ResourceProjectionSnapshot`] 取得 `Arc` 后，可独立查询捕获的目录管理字段。
/// 后续提交不会修改此对象；正常生产器在没有管理行净变化时保留同一 generation 身份。
/// 有序存储按主定位符的完整显示文本、再按 ID 排列，独立 ID 与定位符分片只加速查找。
/// 页、扫描和两个索引引用同一行发布；增量 generation 可共享未触及的行、页和索引分片。
/// 该快照不包含完整诊断文本或载荷，不证明资源已驻留；观察发布次数也不能代替对象身份。
///
/// # 参考
///
/// [`Self::identity`]、[`Self::page`]、[`Self::scan`]、[`Self::summary`]。
#[derive(Clone, Debug)]
pub struct ResourceManagementGeneration {
    diagnostics: ResourceManagementGenerationDiagnostics,
    summary: ResourceManagementSummary,
    hash_authority: Arc<ResourceManagementHashAuthority>,
    ordered_pages: Arc<[Arc<[Arc<ResourceManagementRow>]>]>,
    id_shards: Arc<[Arc<ResourceManagementIdShard>]>,
    locator_shards: Arc<[Arc<ResourceManagementLocatorShard>]>,
}

/// Process-local identity of one immutable management generation publication.
///
/// 缓存与页面配对可使用此身份保留同一管理发布。
///
/// # 约束与生命周期
///
/// 持有一个 generation 的 `Arc`，克隆保留同一分配；`Eq` 与 `Hash` 使用分配指针。
/// 不同 generation 即使内容或发布计数相同也不是同一身份；身份只适合进程内缓存及快照配对。
/// 旧身份保留旧快照，不能当作跨进程序列或内容摘要；取配对管理与就绪视图时使用
/// [`crate::ResourceProjectionSnapshot`]。
///
/// # 参考
///
/// [`ResourceManagementGeneration::identity`]、[`Self::generation`]。
#[derive(Clone)]
pub struct ResourceManagementGenerationIdentity(Arc<ResourceManagementGeneration>);

impl ResourceManagementGenerationIdentity {
    fn new(generation: Arc<ResourceManagementGeneration>) -> Self {
        Self(generation)
    }

    pub fn generation(&self) -> &ResourceManagementGeneration {
        self.0.as_ref()
    }
}

impl fmt::Debug for ResourceManagementGenerationIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResourceManagementGenerationIdentity")
    }
}

impl PartialEq for ResourceManagementGenerationIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ResourceManagementGenerationIdentity {}

impl Hash for ResourceManagementGenerationIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(Arc::as_ptr(&self.0) as usize);
    }
}

impl Default for ResourceManagementGeneration {
    fn default() -> Self {
        let hash_authority = Arc::new(ResourceManagementHashAuthority::default());
        let empty_id_shard = Arc::new(ResourceManagementIdShard::from_entries(
            hash_authority.id_map_with_capacity(0),
        ));
        let empty_locator_shard = Arc::new(ResourceManagementLocatorShard::default());
        Self {
            diagnostics: ResourceManagementGenerationDiagnostics::default(),
            summary: ResourceManagementSummary::default(),
            hash_authority,
            ordered_pages: Arc::from([]),
            id_shards: vec![empty_id_shard; RESOURCE_MANAGEMENT_ID_SHARD_COUNT].into(),
            locator_shards: vec![empty_locator_shard; RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT]
                .into(),
        }
    }
}

impl ResourceManagementGeneration {
    /// 从当前共享快照取得进程内发布身份。
    ///
    /// # 返回
    ///
    /// 克隆 `self` 的 `Arc` 并包装身份，保持该 generation 存活；不会复制行或重新捕获管理器。
    ///
    /// # 约束与生命周期
    ///
    /// 调用者需要持有 `Arc<Self>`；缓存比较应使用此身份，而不是诊断发布次数。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementGenerationIdentity`]。
    pub fn identity(self: &Arc<Self>) -> ResourceManagementGenerationIdentity {
        ResourceManagementGenerationIdentity::new(Arc::clone(self))
    }

    /// 把生产器已协调好的有序页、汇总和查找索引封装为一个管理 generation。
    ///
    /// # 参数
    ///
    /// `diagnostics` 是该发布的观察值；`summary` 须描述这些行；`hash_authority`
    /// 须与两个索引的分片选择规则一致。`ordered_pages` 转移共享有序页，`id_shards`
    /// 与 `locator_shards` 转移相同成员的共享查找数组。
    ///
    /// # 返回
    ///
    /// 返回拥有各部分的快照，不复制共享行，也不返回验证错误。
    ///
    /// # 约束与生命周期
    ///
    /// 调用方须保证 ID 与主定位符唯一、汇总计数正确、索引与有序页成员及行身份一致。
    /// ID/定位符数组长度必须等于各自分片常量；页须非空且展平后按
    /// [`resource_management_row_order`] 全局有序。当前仅以 `debug_assertions` 检查
    /// 数组长度、页非空与排序，其余一致性仍是生产器责任。
    ///
    /// # 参考
    ///
    /// [`Self::from_sorted_rows_with_hash_authority`]。
    pub(crate) fn from_parts(
        diagnostics: ResourceManagementGenerationDiagnostics,
        summary: ResourceManagementSummary,
        hash_authority: Arc<ResourceManagementHashAuthority>,
        ordered_pages: Arc<[Arc<[Arc<ResourceManagementRow>]>]>,
        id_shards: Arc<[Arc<ResourceManagementIdShard>]>,
        locator_shards: Arc<[Arc<ResourceManagementLocatorShard>]>,
    ) -> Self {
        debug_assert_eq!(id_shards.len(), RESOURCE_MANAGEMENT_ID_SHARD_COUNT);
        debug_assert_eq!(
            locator_shards.len(),
            RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT
        );
        debug_assert!(ordered_pages.iter().all(|page| !page.is_empty()));
        debug_assert!(ordered_pages
            .iter()
            .flat_map(|page| page.iter())
            .is_sorted_by(|left, right| { resource_management_row_order(left, right).is_le() }));
        Self {
            diagnostics,
            summary,
            hash_authority,
            ordered_pages,
            id_shards,
            locator_shards,
        }
    }

    #[cfg(test)]
    pub(crate) fn from_rows(
        publication_count: u64,
        mut rows: Vec<Arc<ResourceManagementRow>>,
    ) -> Self {
        rows.sort_by(|left, right| resource_management_row_order(left, right));
        Self::from_sorted_rows(publication_count, rows)
    }

    #[cfg(test)]
    pub(crate) fn from_sorted_rows(
        publication_count: u64,
        rows: Vec<Arc<ResourceManagementRow>>,
    ) -> Self {
        Self::from_sorted_rows_with_hash_authority(
            ResourceManagementGenerationDiagnostics { publication_count },
            rows,
            Arc::new(ResourceManagementHashAuthority::default()),
        )
    }

    /// 从有序共享行重建汇总、平衡存储页和两个查找索引。
    ///
    /// # 参数
    ///
    /// `diagnostics` 直接作为该发布的观察值；`rows` 转移全局已排序且 ID/主定位符唯一的行列表；
    /// `hash_authority` 规定新索引的分片与映射种子，增量重建须保留旧 authority。
    ///
    /// # 返回
    ///
    /// 返回封装后的 generation；页与索引克隆同一行 `Arc`，不复制行字段。
    ///
    /// # 约束与生命周期
    ///
    /// 输入排序在 `debug_assertions` 配置下检查；本方法不按 ID 或定位符去重。重复键会覆盖索引条目，
    /// 而汇总与页仍计入每行，因此调用方必须先满足成员唯一性。
    /// 本方法不返回 `Result`；构造一致性由生产器负责。
    ///
    /// # 参考
    ///
    /// [`Self::from_parts`]、[`resource_management_pages_from_sorted_rows`]、
    /// [`resource_management_id_maps_from_ordered_pages`]。
    pub(crate) fn from_sorted_rows_with_hash_authority(
        diagnostics: ResourceManagementGenerationDiagnostics,
        rows: Vec<Arc<ResourceManagementRow>>,
        hash_authority: Arc<ResourceManagementHashAuthority>,
    ) -> Self {
        debug_assert!(
            rows.is_sorted_by(|left, right| { resource_management_row_order(left, right).is_le() })
        );
        let mut summary = ResourceManagementSummary::default();
        let mut locator_entries = (0..RESOURCE_MANAGEMENT_LOCATOR_SHARD_COUNT)
            .map(|_| HashMap::new())
            .collect::<Vec<_>>();
        for row in &rows {
            summary.add(row);
            locator_entries[hash_authority.locator_shard_index(&row.primary_locator)]
                .insert(Arc::clone(&row.primary_locator), row.id);
        }
        let ordered_pages = resource_management_pages_from_sorted_rows(rows);
        let id_entries =
            resource_management_id_maps_from_ordered_pages(&ordered_pages, hash_authority.as_ref());
        Self::from_parts(
            diagnostics,
            summary,
            hash_authority,
            ordered_pages,
            id_entries
                .into_iter()
                .map(ResourceManagementIdShard::from_entries)
                .map(Arc::new)
                .collect::<Vec<_>>()
                .into(),
            locator_entries
                .into_iter()
                .map(ResourceManagementLocatorShard::from_entries)
                .map(Arc::new)
                .collect::<Vec<_>>()
                .into(),
        )
    }

    pub fn diagnostics(&self) -> ResourceManagementGenerationDiagnostics {
        self.diagnostics
    }

    pub fn summary(&self) -> &ResourceManagementSummary {
        &self.summary
    }

    /// 按稳定资源 ID 从此快照取得共享管理行。
    ///
    /// # 参数
    ///
    /// `id` 是待查询的目录资源身份。
    ///
    /// # 返回
    ///
    /// 存在时返回索引保存的行 `Arc` 克隆，否则返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 只查询捕获快照，不读取当前管理器或取得载荷；共享行在本快照的页与定位符索引中身份一致。
    ///
    /// # 参考
    ///
    /// [`Self::row_identity_by_id`]、[`Self::row_by_locator`]。
    pub fn row_by_id(&self, id: ResourceId) -> Option<Arc<ResourceManagementRow>> {
        self.id_shards[self.id_shard_index(id)].row_by_id(id)
    }

    /// 按资源 ID 取得此快照中行的进程内发布身份。
    ///
    /// # 参数
    ///
    /// `id` 是待查询的资源身份。
    ///
    /// # 返回
    ///
    /// 存在时返回持有同一行分配的身份，缺少时返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 正常增量发布可共享未变化行，因而不同 generation 中此身份仍可相等。
    ///
    /// # 参考
    ///
    /// [`Self::row_by_id`]、[`ResourceManagementRowIdentity`]。
    pub fn row_identity_by_id(&self, id: ResourceId) -> Option<ResourceManagementRowIdentity> {
        self.row_by_id(id).map(ResourceManagementRowIdentity::new)
    }

    /// 按主定位符的完整显示文本从此快照取得共享管理行。
    ///
    /// # 参数
    ///
    /// `locator` 必须与索引中的显示文本精确相等；调用方可先解析
    /// [`crate::ResourceLocator`] 并使用其显示值，本方法不解析、规范化或查询 artifact 定位符。
    ///
    /// # 返回
    ///
    /// 命中主定位符且对应 ID 行存在时返回行 `Arc` 克隆，否则返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 只查询此快照；正常构造中返回行与按 ID 查询为同一分配。
    ///
    /// # 参考
    ///
    /// [`Self::row_by_id`]、[`Self::row_identity_by_locator`]。
    pub fn row_by_locator(&self, locator: &str) -> Option<Arc<ResourceManagementRow>> {
        let shard = &self.locator_shards[self.locator_shard_index(locator)];
        self.row_by_id(shard.id_by_locator(locator)?)
    }

    /// 按主定位符显示文本取得此快照中行的进程内身份。
    ///
    /// # 参数
    ///
    /// `locator` 使用完整显示文本，与 [`Self::row_by_locator`] 的输入规则相同。
    ///
    /// # 返回
    ///
    /// 命中时返回该行分配的身份，否则返回 `None`；同一行的 ID 查询产生相等身份。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementRowIdentity`]、[`Self::row_identity_by_id`]。
    pub fn row_identity_by_locator(&self, locator: &str) -> Option<ResourceManagementRowIdentity> {
        self.row_by_locator(locator)
            .map(ResourceManagementRowIdentity::new)
    }

    /// 为一个查询创建独立游标，逐条读取此快照的匹配管理行。
    ///
    /// # 参数
    ///
    /// `query` 的种类与目录阶段条件同时生效；默认查询允许所有行。
    ///
    /// # 返回
    ///
    /// 返回尚未推进的 [`ResourceManagementScan`]，匹配总数从此快照汇总取得。
    ///
    /// # 约束与生命周期
    ///
    /// 保留 `self` 的 `Arc`，后续发布不改变扫描源；不取得管理器锁或载荷。
    /// 初始游标和 profiling 计数为零；克隆扫描后各副本独立推进。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementScan::next_row`]、[`ResourceManagementScan::is_complete`]。
    pub fn scan(self: &Arc<Self>, query: ResourceManagementQuery) -> ResourceManagementScan {
        ResourceManagementScan {
            generation: Arc::clone(self),
            query,
            page_index: 0,
            row_index: 0,
            yielded_count: 0,
            total_matching_count: self.summary.matching_count(query),
            #[cfg(feature = "profiling")]
            profile_metrics: ResourceManagementScanDiagnostics::default(),
        }
    }

    /// 按此快照既有的定位符、ID 排列对匹配行分页；页面身份与总数不混入后续发布。
    ///
    /// 分页使用匹配序列的偏移与返回上限。
    ///
    /// # 参数
    ///
    /// `query` 的两个可选条件取交集；`offset` 是过滤后匹配序列中的零起点偏移，
    /// 不是内部页号或未过滤行偏移；`limit` 是最多返回的匹配行数。
    ///
    /// # 返回
    ///
    /// 返回 [`ResourceManagementPage`]，包含同一 generation 身份与全查询匹配总数。
    /// `limit == 0` 或偏移达到、超过总数时行切片为空；不足上限时返回剩余匹配行。
    ///
    /// # 约束与生命周期
    ///
    /// 每次调用从有序存储起点扫描到请求区间；连续分页应保留同一个 `Arc<Self>`，
    /// 重新捕获管理器会得到另一次发布的偏移与总数。页面新建切片并共享行，
    /// 零上限仍返回原快照身份与总数。长序列逐行消费可使用 [`Self::scan`]。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementQuery`]、[`Self::identity`]。
    pub fn page(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> ResourceManagementPage {
        self.page_with_profile_metrics::<false>(query, offset, limit)
            .0
    }

    /// 执行同样的快照分页，并返回这次查询实际完成的工作计数。
    ///
    /// # 参数
    ///
    /// `query`、`offset` 与 `limit` 使用 [`Self::page`] 的过滤与匹配序列规则。
    ///
    /// # 返回
    ///
    /// 返回页面和查询局部指标；指标只累计到达到上限或扫描结束，零上限不访问页。
    ///
    /// # 约束与生命周期
    ///
    /// 仅在 profiling 配置中供 crate 内观察使用，不改变正常分页内容或快照身份。
    ///
    /// # 参考
    ///
    /// [`ResourceManagementPageProfileMetrics`]。
    #[cfg(feature = "profiling")]
    pub(crate) fn profiled_page(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> (ResourceManagementPage, ResourceManagementPageProfileMetrics) {
        self.page_with_profile_metrics::<true>(query, offset, limit)
    }

    fn page_with_profile_metrics<const PROFILE: bool>(
        self: &Arc<Self>,
        query: ResourceManagementQuery,
        offset: usize,
        limit: usize,
    ) -> (ResourceManagementPage, ResourceManagementPageProfileMetrics) {
        let total_matching_count = self.summary.matching_count(query);
        let mut metrics = ResourceManagementPageProfileMetrics::default();
        let mut matching_index = 0usize;
        let mut rows = Vec::with_capacity(limit.min(total_matching_count.saturating_sub(offset)));
        if limit != 0 {
            'pages: for page in self.ordered_pages.iter() {
                if PROFILE {
                    metrics.shard_candidate_checks =
                        metrics.shard_candidate_checks.saturating_add(1);
                }
                for row in page.iter() {
                    if !query.matches(row) {
                        if PROFILE {
                            metrics.filtered_rows_skipped =
                                metrics.filtered_rows_skipped.saturating_add(1);
                        }
                        continue;
                    }
                    if PROFILE {
                        metrics.candidate_rows = metrics.candidate_rows.saturating_add(1);
                    }
                    if matching_index >= offset {
                        rows.push(Arc::clone(row));
                        if rows.len() == limit {
                            break 'pages;
                        }
                    }
                    matching_index = matching_index.saturating_add(1);
                }
            }
        }
        if PROFILE {
            metrics.rows_returned = rows.len() as u64;
        }
        (
            ResourceManagementPage {
                generation: self.identity(),
                total_matching_count,
                rows: rows.into(),
            },
            metrics,
        )
    }

    pub(crate) fn ordered_pages(&self) -> &[Arc<[Arc<ResourceManagementRow>]>] {
        &self.ordered_pages
    }

    pub(crate) fn ordered_rows(&self) -> impl Iterator<Item = &Arc<ResourceManagementRow>> {
        self.ordered_pages.iter().flat_map(|page| page.iter())
    }

    pub(crate) fn id_shards(&self) -> &[Arc<ResourceManagementIdShard>] {
        &self.id_shards
    }

    pub(crate) fn id_shards_arc(&self) -> Arc<[Arc<ResourceManagementIdShard>]> {
        Arc::clone(&self.id_shards)
    }

    // 复用旧分片时必须同时保留原哈希 authority，否则新发布会按不同种子选择分片。
    pub(crate) fn hash_authority_arc(&self) -> Arc<ResourceManagementHashAuthority> {
        Arc::clone(&self.hash_authority)
    }

    pub(crate) fn id_shard_index(&self, id: ResourceId) -> usize {
        self.hash_authority.id_shard_index(id)
    }

    pub(crate) fn locator_shard_index(&self, locator: &str) -> usize {
        self.hash_authority.locator_shard_index(locator)
    }

    pub(crate) fn locator_shards(&self) -> &[Arc<ResourceManagementLocatorShard>] {
        &self.locator_shards
    }

    pub(crate) fn locator_shards_arc(&self) -> Arc<[Arc<ResourceManagementLocatorShard>]> {
        Arc::clone(&self.locator_shards)
    }
}

/// 比较管理有序存储使用的定位符显示文本与资源 ID 排序键。
///
/// # 参数
///
/// `left`、`right` 借用要比较的管理行。
///
/// # 返回
///
/// 先按 `primary_locator` 整个字符串的字典序比较，相等时再按 ID 比较。
/// 这里不把 scheme、path 与 label 拆成独立排序字段。
///
/// # 约束与生命周期
///
/// 排序构造、增量合并、查找页边界和查询读取须遵守同一规则；只比较键，不比较其他行字段。
///
/// # 参考
///
/// [`ResourceManagementGeneration::from_parts`]。
pub(crate) fn resource_management_row_order(
    left: &ResourceManagementRow,
    right: &ResourceManagementRow,
) -> Ordering {
    left.primary_locator
        .cmp(&right.primary_locator)
        .then_with(|| left.id.cmp(&right.id))
}

/// 把已全局排序的共享行分成非空、长度平衡的不可变存储页。
///
/// # 参数
///
/// `rows` 转移排序行列表；调用方须先按 [`resource_management_row_order`] 排列并维护成员唯一性。
///
/// # 返回
///
/// 空输入返回空页数组；非空输入生成 `ceil(len / RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS)` 页。
/// 每页长度不超过该上限，长度至多相差一，较大的页置前；行 `Arc` 被移动而不复制行值。
///
/// # 约束与生命周期
///
/// 本方法不排序、不校验唯一性；分组保持输入顺序，这是内部存储页而非查询分页。
///
/// # 参考
///
/// [`RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS`]、[`ResourceManagementGeneration::page`]。
pub(crate) fn resource_management_pages_from_sorted_rows(
    rows: Vec<Arc<ResourceManagementRow>>,
) -> Arc<[Arc<[Arc<ResourceManagementRow>]>]> {
    if rows.is_empty() {
        return Arc::from([]);
    }
    let page_count = rows.len().div_ceil(RESOURCE_MANAGEMENT_ORDERED_PAGE_ROWS);
    let rows_per_page = rows.len() / page_count;
    let larger_page_count = rows.len() % page_count;
    let mut rows = rows.into_iter();
    (0..page_count)
        .map(|page_index| {
            let page_len = rows_per_page + usize::from(page_index < larger_page_count);
            rows.by_ref().take(page_len).collect::<Vec<_>>().into()
        })
        .collect::<Vec<_>>()
        .into()
}

/// 从有序页建立所有 ID 分片，使索引和页共享同一行发布。
///
/// # 参数
///
/// `ordered_pages` 借用要索引的页；行 ID 必须唯一。`hash_authority` 须与拥有该索引的 generation
/// 使用同一 ID 分片规则，映射容量按每个分片的行数预留。
///
/// # 返回
///
/// 返回长度为 [`RESOURCE_MANAGEMENT_ID_SHARD_COUNT`] 的映射列表；值为页中行 `Arc` 的克隆。
///
/// # 约束与生命周期
///
/// 两次遍历分别计数与插入，不复制行字段；重复 ID 会覆盖索引值，本方法不报告错误或去重页。
///
/// # 参考
///
/// [`ResourceManagementGeneration::from_sorted_rows_with_hash_authority`]。
pub(crate) fn resource_management_id_maps_from_ordered_pages(
    ordered_pages: &[Arc<[Arc<ResourceManagementRow>]>],
    hash_authority: &ResourceManagementHashAuthority,
) -> Vec<ResourceManagementIdMap<Arc<ResourceManagementRow>>> {
    let mut counts = vec![0usize; RESOURCE_MANAGEMENT_ID_SHARD_COUNT];
    for row in ordered_pages.iter().flat_map(|page| page.iter()) {
        let index = hash_authority.id_shard_index(row.id);
        counts[index] = counts[index].saturating_add(1);
    }
    let mut entries = counts
        .into_iter()
        .map(|capacity| hash_authority.id_map_with_capacity(capacity))
        .collect::<Vec<_>>();
    for row in ordered_pages.iter().flat_map(|page| page.iter()) {
        entries[hash_authority.id_shard_index(row.id)].insert(row.id, Arc::clone(row));
    }
    entries
}

#[cfg(test)]
mod tests;
