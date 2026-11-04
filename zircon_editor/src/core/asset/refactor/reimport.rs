//! One source reimport resolves its current locator from stable asset identity at execution.
use std::fmt;
use std::time::Instant;

use zircon_runtime::asset::{AssetStatusRecord, AssetUri, AssetUuid};

use crate::core::jobs::{JobError, JobId, JobTicket};

#[derive(Clone, Debug, PartialEq)]
pub struct EditorAssetReimportResult {
    target_uuid: AssetUuid,
    source: AssetUri,
    status: Option<AssetStatusRecord>,
}

impl EditorAssetReimportResult {
    pub fn new(
        target_uuid: AssetUuid,
        source: AssetUri,
        status: Option<AssetStatusRecord>,
    ) -> Self {
        Self {
            target_uuid,
            source,
            status,
        }
    }

    pub fn target_uuid(&self) -> &AssetUuid {
        &self.target_uuid
    }
    pub fn source(&self) -> &AssetUri {
        &self.source
    }
    pub fn status(&self) -> Option<&AssetStatusRecord> {
        self.status.as_ref()
    }
}

pub struct EditorAssetReimportTicket {
    ticket: JobTicket<EditorAssetReimportResult>,
    target_uuid: AssetUuid,
}

impl EditorAssetReimportTicket {
    pub(crate) fn new(
        ticket: JobTicket<EditorAssetReimportResult>,
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
    pub fn try_take(&self) -> Option<Result<EditorAssetReimportResult, JobError>> {
        self.ticket.try_take()
    }
    pub fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetReimportResult, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for EditorAssetReimportTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorAssetReimportTicket")
            .field("target_uuid", &self.target_uuid)
            .field("job_id", &self.id())
            .finish()
    }
}
