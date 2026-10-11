use std::collections::HashMap;

use crate::{
    ResourceId, ResourceLocator, ResourceProjectionSnapshot, ResourceRecord, UntypedResourceHandle,
};

/// 一个已提交批次涉及的最终记录、移除结果与配对投影；后续视图应使用这里的投影，避免重新读取时混入下一代发布。
///
/// 回执保留一个成功提交批次的最终记录和配对快照。
///
/// # 约束与生命周期
///
/// 由 [`crate::ResourceManager::commit`] 成功提交后返回，保存本批次触及资源的最终保留记录，
/// 以及提交前已存在、最终被移除的记录；它不是整个目录，也不按操作保留中间历史。
/// 创建后又在同批次移除且提交前不存在的临时记录不会出现在两组记录中。
/// 记录与配对投影在同一权威写锁内形成；后续发布不会刷新旧回执。
/// 回执不持有发布锁或载荷驻留租约，丢弃不会撤销提交，返回的句柄也不证明载荷可读。
/// Clone 会复制记录映射并共享投影 Arc；记录借用和迭代器的生命周期受该回执限制。
///
/// # 参考
///
/// [`crate::ResourceMutationBatch`]、[`crate::ResourceProjectionSnapshot`]、
/// [`Self::record`]、[`Self::removed`]。
#[derive(Clone, Debug)]
pub struct ResourceMutationReceipt {
    records: HashMap<ResourceId, ResourceRecord>,
    removed: HashMap<ResourceId, ResourceRecord>,
    projections: ResourceProjectionSnapshot,
    published_event_count: usize,
}

impl ResourceMutationReceipt {
    /// 接收已应用批次的最终记录与同次投影，构造只读提交回执。
    ///
    /// # 参数
    ///
    /// `records` 转移批次触及且最终保留的记录；`removed` 转移提交前已有、最终移除的记录。
    /// `projections` 是同一权威锁内捕获的管理与就绪配对；`published_event_count` 是批次提交给事件日志的变化数。
    ///
    /// # 返回
    ///
    /// 返回拥有两组记录与配对投影的回执，不再次查询权威状态。
    ///
    /// # 约束与生命周期
    ///
    /// 仅供提交路径使用；调用方保证 ID 键与记录一致、两组成员互斥以及投影时间边界正确。
    /// 本构造不验证这些关系或返回错误。
    ///
    /// # 参考
    ///
    /// [`ResourceMutationReceipt`]、[`Self::projection_snapshot`]。
    pub(crate) fn new(
        records: HashMap<ResourceId, ResourceRecord>,
        removed: HashMap<ResourceId, ResourceRecord>,
        projections: ResourceProjectionSnapshot,
        published_event_count: usize,
    ) -> Self {
        Self {
            records,
            removed,
            projections,
            published_event_count,
        }
    }

    /// 按 ID 查找本批次触及且提交后仍保留的最终记录。
    ///
    /// # 参数
    ///
    /// `id` 是批次资源身份，不是操作下标。
    ///
    /// # 返回
    ///
    /// 命中时借用最终记录；未触及或最终移除的资源返回 `None`，不能据此断定当前全局目录不存在该 ID。
    ///
    /// # 约束与生命周期
    ///
    /// 只读回执内容；后续提交不刷新返回记录，借用期受 `self` 限制。
    ///
    /// # 参考
    ///
    /// [`Self::removed`]、[`Self::projection_snapshot`]。
    pub fn record(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.records.get(&id)
    }

    /// 按 ID 查找本批次最终移除的原有记录。
    ///
    /// # 参数
    ///
    /// `id` 是被移除资源的身份。
    ///
    /// # 返回
    ///
    /// 返回提交开始前已存在的记录借用；未最终移除、未触及或先新增再移除的临时资源返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 保存的是提交前基线，并非中间操作刚移除的版本，也不查询后来目录。
    ///
    /// # 参考
    ///
    /// [`Self::removed_records`]、[`Self::record`]。
    pub fn removed(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.removed.get(&id)
    }

    /// 查找本批次涉及、提交后仍保留的最终记录；移除结果应使用 removed 或 removed_records。
    ///
    /// 此查询只覆盖回执保存的最终保留记录。
    ///
    /// # 参数
    ///
    /// `locator` 借用要精确匹配的最终主定位符，使用 [`crate::ResourceLocator`] 的值相等规则。
    ///
    /// # 返回
    ///
    /// 命中时借用本批次最终保留的记录；旧重命名定位符、最终移除记录以及未触及的全局记录不在此查找范围内。
    ///
    /// # 约束与生命周期
    ///
    /// 只在回执记录映射中查找，不查询实时目录或 artifact 定位符；正常提交保证最终主定位符唯一。
    /// 借用期受 `self` 限制，后续提交不会刷新结果。
    ///
    /// # 参考
    ///
    /// [`Self::record`]、[`Self::removed_records`]。
    pub fn record_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.records
            .values()
            .find(|record| &record.primary_locator == locator)
    }

    /// 遍历本批次提交前已存在且最终被移除的记录。
    ///
    /// # 返回
    ///
    /// 返回记录借用迭代器，其生命周期受 `self` 限制；无移除项时为空。
    ///
    /// # 约束与生命周期
    ///
    /// 底层使用 HashMap，迭代顺序不稳定；批次中间移除后重新保留的记录不在此列表，
    /// 提交前不存在且在同批次消失的临时记录也不在列表。
    ///
    /// # 参考
    ///
    /// [`Self::removed`]、[`Self::record`]。
    pub fn removed_records(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.removed.values()
    }

    /// 为本批次最终保留的记录构造携带 ID 与种类的非类型化句柄。
    ///
    /// # 参数
    ///
    /// `id` 是要查询的批次资源身份。
    ///
    /// # 返回
    ///
    /// 命中最终保留记录时返回 [`UntypedResourceHandle`]，否则返回 `None`。
    ///
    /// # 约束与生命周期
    ///
    /// 句柄不包含修订或驻留所有权；它不证明后来目录仍存在该资源、种类转换成功后载荷可读，
    /// 也不触发加载。读取载荷仍须通过资源管理器。
    ///
    /// # 参考
    ///
    /// [`Self::record`]、[`crate::UntypedResourceHandle::typed`]。
    pub fn handle(&self, id: ResourceId) -> Option<UntypedResourceHandle> {
        self.records
            .get(&id)
            .map(|record| UntypedResourceHandle::new(id, record.kind))
    }

    /// 借用本批次在同一权威写锁内形成的管理与就绪配对投影。
    ///
    /// # 返回
    ///
    /// 返回 [`ResourceProjectionSnapshot`] 的借用；两个 generation 来自该次提交时刻，
    /// 未变化的一侧可以复用早先发布的对象。
    ///
    /// # 约束与生命周期
    ///
    /// 后续视图应使用此配对更新；分别重新读取管理器可能混入后来发布。
    /// 保留或克隆投影只保留不可变快照，不持有提交锁或载荷驻留租约。
    ///
    /// # 参考
    ///
    /// [`crate::ResourceProjectionSnapshot::management_identity`]、
    /// [`crate::ResourceProjectionSnapshot::readiness_identity`]。
    pub fn projection_snapshot(&self) -> &ResourceProjectionSnapshot {
        &self.projections
    }

    /// 此批次提交给事件日志的变化数；日志可合并或淘汰事件，不能据此推断任何订阅者实际收到的数量。
    ///
    /// 该观察值描述提交给事件日志的净变化数。
    ///
    /// # 返回
    ///
    /// 返回该批次提交给事件日志的最终净变化事件数；它不是输入操作数、触及记录数或管理发布次数。
    ///
    /// # 约束与生命周期
    ///
    /// 该观察值不提供订阅者逐项送达确认；订阅者仍须处理日志的合并、淘汰与序列缺口。
    /// 无净事件变化时可以为零，即使批次成功返回回执。
    ///
    /// # 参考
    ///
    /// [`Self::projection_snapshot`]、[`crate::ResourceManager::subscribe`]。
    pub fn published_event_count(&self) -> usize {
        self.published_event_count
    }
}
