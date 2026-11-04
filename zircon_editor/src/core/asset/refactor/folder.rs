//! Folder commands retain canonical locators while Runtime owns the filesystem transaction.
use std::fmt;
use std::time::Instant;

use zircon_runtime::asset::{AssetStatusRecord, AssetUri};

use crate::core::jobs::{JobError, JobId, JobTicket};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorAssetFolderOperation {
    Create,
    Rename,
    Delete,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EditorAssetFolderResult {
    operation: EditorAssetFolderOperation,
    source: AssetUri,
    target: Option<AssetUri>,
    statuses: Vec<AssetStatusRecord>,
}

impl EditorAssetFolderResult {
    pub fn new(
        operation: EditorAssetFolderOperation,
        source: AssetUri,
        target: Option<AssetUri>,
        statuses: Vec<AssetStatusRecord>,
    ) -> Self {
        Self {
            operation,
            source,
            target,
            statuses,
        }
    }

    pub fn operation(&self) -> EditorAssetFolderOperation {
        self.operation
    }
    pub fn source(&self) -> &AssetUri {
        &self.source
    }
    pub fn target(&self) -> Option<&AssetUri> {
        self.target.as_ref()
    }
    pub fn statuses(&self) -> &[AssetStatusRecord] {
        &self.statuses
    }
}

pub struct EditorAssetFolderTicket {
    ticket: JobTicket<EditorAssetFolderResult>,
    operation: EditorAssetFolderOperation,
    source: AssetUri,
    target: Option<AssetUri>,
}

impl EditorAssetFolderTicket {
    pub(crate) fn new(
        ticket: JobTicket<EditorAssetFolderResult>,
        operation: EditorAssetFolderOperation,
        source: AssetUri,
        target: Option<AssetUri>,
    ) -> Self {
        Self {
            ticket,
            operation,
            source,
            target,
        }
    }

    pub fn id(&self) -> JobId {
        self.ticket.id()
    }
    pub fn operation(&self) -> EditorAssetFolderOperation {
        self.operation
    }
    pub fn source(&self) -> &AssetUri {
        &self.source
    }
    pub fn target(&self) -> Option<&AssetUri> {
        self.target.as_ref()
    }
    pub fn try_take(&self) -> Option<Result<EditorAssetFolderResult, JobError>> {
        self.ticket.try_take()
    }
    pub fn wait_until(
        &self,
        deadline: Instant,
    ) -> Option<Result<EditorAssetFolderResult, JobError>> {
        self.ticket.wait_until(deadline)
    }
}

impl fmt::Debug for EditorAssetFolderTicket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EditorAssetFolderTicket")
            .field("operation", &self.operation)
            .field("source", &self.source)
            .field("target", &self.target)
            .field("job_id", &self.id())
            .finish()
    }
}
