use crate::{
    ResourceDiagnostic, ResourceId, ResourceLocator, ResourceMutationBatch, ResourceRecord,
    ResourceResult, UntypedResourceHandle,
};

use super::resource_manager::ResourceManager;

impl ResourceManager {
    pub fn register_record(&self, record: ResourceRecord) -> ResourceResult<UntypedResourceHandle> {
        let id = record.id;
        let receipt = self.commit(ResourceMutationBatch::new().upsert_lazy(record))?;
        Ok(receipt
            .handle(id)
            .expect("a committed record upsert produces a handle"))
    }

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
