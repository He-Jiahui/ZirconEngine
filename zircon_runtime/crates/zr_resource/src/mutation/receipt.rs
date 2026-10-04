use std::collections::HashMap;

use crate::{
    ResourceId, ResourceLocator, ResourceProjectionSnapshot, ResourceRecord, UntypedResourceHandle,
};

/// 一个已提交批次涉及的最终记录、移除结果与配对投影；后续视图应使用这里的投影，避免重新读取时混入下一代发布。
#[derive(Clone, Debug)]
pub struct ResourceMutationReceipt {
    records: HashMap<ResourceId, ResourceRecord>,
    removed: HashMap<ResourceId, ResourceRecord>,
    projections: ResourceProjectionSnapshot,
    published_event_count: usize,
}

impl ResourceMutationReceipt {
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

    pub fn record(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.records.get(&id)
    }

    pub fn removed(&self, id: ResourceId) -> Option<&ResourceRecord> {
        self.removed.get(&id)
    }

    pub fn record_by_locator(&self, locator: &ResourceLocator) -> Option<&ResourceRecord> {
        self.records
            .values()
            .find(|record| &record.primary_locator == locator)
    }

    pub fn removed_records(&self) -> impl Iterator<Item = &ResourceRecord> {
        self.removed.values()
    }

    pub fn handle(&self, id: ResourceId) -> Option<UntypedResourceHandle> {
        self.records
            .get(&id)
            .map(|record| UntypedResourceHandle::new(id, record.kind))
    }

    pub fn projection_snapshot(&self) -> &ResourceProjectionSnapshot {
        &self.projections
    }

    /// 此批次提交给事件日志的变化数；日志可合并或淘汰事件，不能据此推断任何订阅者实际收到的数量。
    pub fn published_event_count(&self) -> usize {
        self.published_event_count
    }
}
