//! 迁移票据保留源资源身份和目标位置，完成结果携带是否真正改变及导入状态；工作区据此刷新身份投影，不从显示名称推断迁移效果。
use std::fmt;
use std::time::Instant;

use zircon_runtime::asset::{AssetStatusRecord, AssetUri, AssetUuid};

use crate::core::jobs::{JobError, JobId, JobTicket};

#[derive(Clone, Debug, PartialEq)]
pub struct EditorAssetRelocationResult {
    source_uuid: AssetUuid,
    target: AssetUri,
    statuses: Vec<AssetStatusRecord>,
}

impl EditorAssetRelocationResult {
    pub fn new(source_uuid: AssetUuid, target: AssetUri, statuses: Vec<AssetStatusRecord>) -> Self {
        Self {
            source_uuid,
            target,
            statuses,
        }
    }

    pub fn source_uuid(&self) -> &AssetUuid {
        &self.source_uuid
    }

    pub fn target(&self) -> &AssetUri {
        &self.target
    }

    pub fn statuses(&self) -> &[AssetStatusRecord] {
        &self.statuses
    }

    pub fn changed(&self) -> bool {
        !self.statuses.is_empty()
    }
}

pub struct EditorAssetRelocationTicket {
    ticket: JobTicket<EditorAssetRelocationResult>,
    target: AssetUri,
}

impl EditorAssetRelocationTicket {
    pub(crate) fn new(ticket: JobTicket<EditorAssetRelocationResult>, target: AssetUri) -> Self {
        Self { ticket, target }
    }

    pub fn id(&self) -> JobId {
        self.ticket.id()
    }

    pub fn target(&self) -> &AssetUri {
        &self.target
    }

    pub fn try_take(&self) -> Option<Result<EditorAssetRelocationResult, JobError>> {
        self.ticket.try_take()
    }

    pub fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetRelocationResult, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for EditorAssetRelocationTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorAssetRelocationTicket")
            .field("target", &self.target)
            .field("job_id", &self.id())
            .finish()
    }
}
