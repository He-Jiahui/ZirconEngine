use std::sync::Arc;

use crate::{ResourceData, ResourceDiagnostic, ResourceId, ResourceLocator, ResourceRecord};

use super::ResourceMutationOperation;

/// 按调用顺序记录待提交操作；构建过程不修改管理器，统一预检后才能原子发布。
/// 同一资源的多次操作按最终结果生成事件，调用端不能把批次当成逐条通知队列。
#[derive(Debug, Default)]
pub struct ResourceMutationBatch {
    operations: Vec<ResourceMutationOperation>,
}

impl ResourceMutationBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    pub fn upsert_lazy(mut self, record: ResourceRecord) -> Self {
        self.operations
            .push(ResourceMutationOperation::UpsertLazy(record));
        self
    }

    /// 发布记录及其对应载荷；来源、导入配置等内容身份应与载荷一致，改变内容时须更新相应字段。
    pub fn upsert_ready<TData>(self, record: ResourceRecord, payload: TData) -> Self
    where
        TData: ResourceData,
    {
        self.upsert_ready_erased(record, Arc::new(payload))
    }

    pub(crate) fn upsert_ready_erased(
        self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.push_ready(record, payload, false)
    }

    /// 导入成功可从 Error 恢复到 Ready；此入口仅擦除具体类型，调用端仍负责载荷与记录的种类及内容一致性。
    /// Adds an imported payload that has already crossed a type-erased importer boundary.
    pub fn upsert_imported_erased(
        self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.push_ready(record, payload, true)
    }

    fn push_ready(
        mut self,
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
        recover_from_error: bool,
    ) -> Self {
        self.operations
            .push(ResourceMutationOperation::UpsertReady {
                record,
                payload,
                recover_from_error,
            });
        self
    }

    /// 填充已有 Ready 记录的确切版本；传入异步加载开始时取得的版本，过期结果会使整个批次被拒绝。
    pub fn store_payload<TData>(
        self,
        id: ResourceId,
        expected_revision: u64,
        payload: TData,
    ) -> Self
    where
        TData: ResourceData,
    {
        self.store_payload_erased(id, expected_revision, Arc::new(payload))
    }

    /// 类型擦除不会改变版本保护；仅补充该版本的载荷，不表示新的来源内容发布。
    /// Stores a payload that has already crossed a type-erased resource-service boundary.
    pub fn store_payload_erased(
        mut self,
        id: ResourceId,
        expected_revision: u64,
        payload: Arc<dyn ResourceData>,
    ) -> Self {
        self.operations
            .push(ResourceMutationOperation::StorePayload {
                id,
                expected_revision,
                payload,
            });
        self
    }

    pub fn start_reload(mut self, id: ResourceId, diagnostics: Vec<ResourceDiagnostic>) -> Self {
        self.operations
            .push(ResourceMutationOperation::StartReload { id, diagnostics });
        self
    }

    pub fn fail_reload(mut self, id: ResourceId, diagnostics: Vec<ResourceDiagnostic>) -> Self {
        self.operations
            .push(ResourceMutationOperation::FailReload { id, diagnostics });
        self
    }

    pub fn rename(mut self, from: ResourceLocator, to: ResourceLocator) -> Self {
        self.operations
            .push(ResourceMutationOperation::Rename { from, to });
        self
    }

    pub fn remove(mut self, locator: ResourceLocator) -> Self {
        self.operations.push(ResourceMutationOperation::Remove {
            locator,
            expected_kind: None,
        });
        self
    }

    pub fn remove_kind(
        mut self,
        locator: ResourceLocator,
        expected_kind: crate::ResourceKind,
    ) -> Self {
        self.operations.push(ResourceMutationOperation::Remove {
            locator,
            expected_kind: Some(expected_kind),
        });
        self
    }

    pub(crate) fn operations(self) -> Vec<ResourceMutationOperation> {
        self.operations
    }
}
