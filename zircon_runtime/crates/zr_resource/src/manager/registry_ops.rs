use crate::{
    ResourceDiagnostic, ResourceId, ResourceLocator, ResourceMutationBatch, ResourceRecord,
    ResourceResult, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    /// 在线登记元数据而不提供载荷；身份与定位符冲突通过批次预检处理，后续加载须使用发布后的版本。
    pub fn register_record(&self, record: ResourceRecord) -> ResourceResult<UntypedResourceHandle> {
        let id = record.id;
        let receipt = self.commit(ResourceMutationBatch::new().upsert_lazy(record))?;
        Ok(receipt
            .handle(id)
            .expect("a committed record upsert produces a handle"))
    }

    /// 将现有资源标记为重载中，保留最后有效载荷；随后发布成功结果或调用失败入口结束这次重载。
    pub fn start_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> ResourceResult<ResourceRecord> {
        let receipt = self.commit(ResourceMutationBatch::new().start_reload(id, diagnostics))?;
        Ok(receipt
            .record(id)
            .expect("a committed reload start produces a record")
            .clone())
    }

    /// 发布重载失败诊断而保留已有载荷，允许使用者继续持有最后有效版本；Pending 首次加载也可报告失败。
    pub fn fail_reload(
        &self,
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    ) -> ResourceResult<ResourceRecord> {
        let receipt = self.commit(ResourceMutationBatch::new().fail_reload(id, diagnostics))?;
        Ok(receipt
            .record(id)
            .expect("a committed reload failure produces a record")
            .clone())
    }

    pub fn remove_by_locator(
        &self,
        locator: &ResourceLocator,
    ) -> ResourceResult<Option<ResourceRecord>> {
        let receipt = self.commit(ResourceMutationBatch::new().remove(locator.clone()))?;
        let removed = receipt.removed_records().next().cloned();
        Ok(removed)
    }

    /// 显式迁移定位符并保持资源 ID；直接改写同一 ID 的定位符不能替代此授权迁移。
    pub fn rename(
        &self,
        from: &ResourceLocator,
        to: ResourceLocator,
    ) -> ResourceResult<ResourceRecord> {
        let receipt = self.commit(ResourceMutationBatch::new().rename(from.clone(), to.clone()))?;
        Ok(receipt
            .record_by_locator(&to)
            .expect("a committed rename produces a record")
            .clone())
    }
}
