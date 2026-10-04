use std::sync::Arc;

use thiserror::Error;

use crate::asset::artifact::{
    RenderArtifactBlockAdmissionError, RenderArtifactBlockCancelReason,
    RenderArtifactBlockFailureCode, RenderArtifactBlockLoadStage, RenderArtifactBlockTicketBatch,
    RenderArtifactDecodedBlock, RenderArtifactLoadPlanError,
};
use crate::core::resource::UntypedResourceHandle;

use super::super::{RenderAssetResidencyRoute, RenderAssetResidencyTicket};
use super::load::RenderAssetSemanticBlockLoad;

pub(crate) enum RenderAssetSemanticBlockLoadAdvance {
    Pending(RenderAssetSemanticBlockLoad, RenderArtifactBlockLoadStage),
    Deferred(
        RenderAssetSemanticBlockLoad,
        RenderArtifactBlockAdmissionError,
    ),
    Ready(RenderAssetCpuBlockLease),
}

/// 把解码块、字节核算和 loader 批次作为同一所有权单元；GPU 计划与上传 lease 继续持有它，直至终态消费或显式丢弃。
pub(crate) struct RenderAssetCpuBlockLease {
    ticket: RenderAssetResidencyTicket,
    blocks: Vec<RenderArtifactDecodedBlock>,
    encoded_bytes: u64,
    decoded_bytes: u64,
    ticket_batches: Vec<RenderArtifactBlockTicketBatch>,
}

impl RenderAssetCpuBlockLease {
    pub(super) fn new(
        ticket: RenderAssetResidencyTicket,
        blocks: Vec<RenderArtifactDecodedBlock>,
        encoded_bytes: u64,
        decoded_bytes: u64,
        ticket_batches: Vec<RenderArtifactBlockTicketBatch>,
    ) -> Self {
        Self {
            ticket,
            blocks,
            encoded_bytes,
            decoded_bytes,
            ticket_batches,
        }
    }

    pub(crate) fn ticket(&self) -> RenderAssetResidencyTicket {
        self.ticket.clone()
    }

    pub(crate) fn blocks(&self) -> &[RenderArtifactDecodedBlock] {
        &self.blocks
    }

    pub(crate) const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    pub(crate) const fn decoded_bytes(&self) -> u64 {
        self.decoded_bytes
    }

    pub(crate) fn batch_count(&self) -> usize {
        self.ticket_batches.len()
    }
}

#[derive(Debug, Error)]
pub(crate) enum RenderAssetSemanticBlockLoadError {
    #[error("render residency route {actual:?} is not a semantic-block route")]
    UnsupportedRoute { actual: RenderAssetResidencyRoute },
    #[error("render manifest resource {actual:?} does not match residency resource {expected:?}")]
    ManifestResourceMismatch {
        expected: UntypedResourceHandle,
        actual: UntypedResourceHandle,
    },
    #[error("render manifest revision {actual} does not match residency revision {expected}")]
    ManifestRevisionMismatch { expected: u64, actual: u64 },
    #[error(transparent)]
    Plan(#[from] RenderArtifactLoadPlanError),
    #[error(transparent)]
    Admission(#[from] RenderArtifactBlockAdmissionError),
    #[error("render semantic block failed with {code:?}: {detail}")]
    BlockFailed {
        code: RenderArtifactBlockFailureCode,
        detail: Arc<str>,
    },
    #[error("render semantic block was cancelled: {reason:?}")]
    BlockCancelled {
        reason: RenderArtifactBlockCancelReason,
    },
    #[error("render semantic block batch completed without every decoded block")]
    IncompleteBatch,
}
