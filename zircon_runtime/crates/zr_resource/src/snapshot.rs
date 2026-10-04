use std::ops::Deref;
use std::sync::Arc;

use crate::ResourceRecord;

/// Immutable payload paired atomically with the exact record revision that owns it.
#[derive(Debug)]
pub struct ResourceSnapshot<TData> {
    record: ResourceRecord,
    resource: Arc<TData>,
}

impl<TData> ResourceSnapshot<TData> {
    // TODO: [CR-RESOURCE-AUDIT-0001] 确认公开构造是否需要限制：此入口不验证记录与载荷来自同一发布，管理器在同一锁内配对；其他调用端的配对责任尚缺少明确约束测试。
    /// 构造已由调用端验证配对的读取结果；需要并发一致性时从管理器读取，不能分别读取记录与载荷后再拼装。
    pub fn new(record: ResourceRecord, resource: Arc<TData>) -> Self {
        Self { record, resource }
    }

    pub fn record(&self) -> &ResourceRecord {
        &self.record
    }

    pub fn revision(&self) -> u64 {
        self.record.revision
    }

    pub fn resource(&self) -> &Arc<TData> {
        &self.resource
    }
}

impl<TData> Deref for ResourceSnapshot<TData> {
    type Target = TData;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref()
    }
}
