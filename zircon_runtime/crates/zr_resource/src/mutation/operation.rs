use std::sync::Arc;

use crate::{ResourceData, ResourceDiagnostic, ResourceId, ResourceLocator, ResourceRecord};

// 管理器内部命令协议；擦除载荷类型便于一个批次包含多种资源，状态与身份约束统一留到完整批次预检。
#[derive(Debug)]
pub(crate) enum ResourceMutationOperation {
    UpsertLazy(ResourceRecord),
    UpsertReady {
        record: ResourceRecord,
        payload: Arc<dyn ResourceData>,
        recover_from_error: bool,
    },
    StorePayload {
        id: ResourceId,
        expected_revision: u64,
        payload: Arc<dyn ResourceData>,
    },
    StartReload {
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    },
    FailReload {
        id: ResourceId,
        diagnostics: Vec<ResourceDiagnostic>,
    },
    Rename {
        from: ResourceLocator,
        to: ResourceLocator,
    },
    Remove {
        locator: ResourceLocator,
        expected_kind: Option<crate::ResourceKind>,
    },
}
