use crate::{ResourceMutationBatch, ResourceRecord, ResourceResult, UntypedResourceHandle};

use super::resource_manager::ResourceManager;

// 项目扫描先发布目录记录，再按需加载载荷；未改变的 Ready 元数据保留已有载荷，内容身份变化使旧载荷失效。
impl ResourceManager {
    pub fn register_lazy_record(
        &self,
        record: ResourceRecord,
    ) -> ResourceResult<UntypedResourceHandle> {
        Ok(self
            .register_lazy_records(std::iter::once(record))?
            .pop()
            .expect("one lazy record produces one handle"))
    }

    /// 将一组离线记录作为一个在线批次发布，返回句柄顺序与输入顺序一致；记录迭代仍在调用线程同步完成。
    pub fn register_lazy_records(
        &self,
        records: impl IntoIterator<Item = ResourceRecord>,
    ) -> ResourceResult<Vec<UntypedResourceHandle>> {
        let records = records.into_iter();
        let (lower_bound, _) = records.size_hint();
        let mut ids = Vec::with_capacity(lower_bound);
        let mut batch = ResourceMutationBatch::new();
        for record in records {
            ids.push(record.id);
            batch = batch.upsert_lazy(record);
        }
        let receipt = self.commit(batch)?;
        Ok(ids
            .into_iter()
            .map(|id| {
                receipt
                    .handle(id)
                    .expect("a committed lazy upsert produces a handle")
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "tests/lazy_registration.rs"]
mod tests;
