use std::fmt;
use std::time::Instant;

use zircon_runtime::asset::{AssetStatusRecord, AssetUuid};

use crate::core::jobs::{JobError, JobId, JobTicket};

#[derive(Clone, Debug, PartialEq)]
pub struct EditorAssetDeletionResult {
    target_uuid: AssetUuid,
    statuses: Vec<AssetStatusRecord>,
}

impl EditorAssetDeletionResult {
    pub fn new(target_uuid: AssetUuid, statuses: Vec<AssetStatusRecord>) -> Self {
        Self {
            target_uuid,
            statuses,
        }
    }

    pub fn target_uuid(&self) -> &AssetUuid {
        &self.target_uuid
    }

    pub fn statuses(&self) -> &[AssetStatusRecord] {
        &self.statuses
    }
}

pub struct EditorAssetDeletionTicket {
    ticket: JobTicket<EditorAssetDeletionResult>,
    target_uuid: AssetUuid,
}

impl EditorAssetDeletionTicket {
    pub(crate) fn new(
        ticket: JobTicket<EditorAssetDeletionResult>,
        target_uuid: AssetUuid,
    ) -> Self {
        Self {
            ticket,
            target_uuid,
        }
    }

    pub fn id(&self) -> JobId {
        self.ticket.id()
    }

    pub fn target_uuid(&self) -> &AssetUuid {
        &self.target_uuid
    }

    pub fn try_take(&self) -> Option<Result<EditorAssetDeletionResult, JobError>> {
        self.ticket.try_take()
    }

    pub fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetDeletionResult, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for EditorAssetDeletionTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorAssetDeletionTicket")
            .field("target_uuid", &self.target_uuid)
            .field("job_id", &self.id())
            .finish()
    }
}
