use std::sync::Arc;
use std::time::Instant;

use crate::asset::artifact::{
    RenderArtifactBlockAdmissionError, RenderArtifactIoPriority, RenderArtifactManifest,
    RenderArtifactManifestAdmissionError, RenderArtifactManifestCancelReason,
    RenderArtifactManifestFailureCode, RenderArtifactManifestLoadStage,
    RenderArtifactManifestLoader, RenderArtifactManifestPoll, RenderArtifactManifestRequest,
    RenderArtifactManifestTicket,
};

use super::super::{RenderAssetResidencyRoute, RenderAssetResidencyTicket};
use super::{
    RenderAssetCpuBlockLease, RenderAssetSemanticBlockLoad, RenderAssetSemanticBlockLoadAdvance,
    RenderAssetSemanticBlockLoadError,
};

pub(crate) enum RenderAssetSemanticLoadStage {
    Manifest(RenderArtifactManifestLoadStage),
    Blocks(crate::asset::artifact::RenderArtifactBlockLoadStage),
}

pub(crate) enum RenderAssetSemanticLoadAdvance {
    Pending(RenderAssetSemanticLoad, RenderAssetSemanticLoadStage),
    Deferred(RenderAssetSemanticLoad, RenderArtifactBlockAdmissionError),
    Ready(RenderAssetCpuArtifactLease),
}

pub(crate) struct RenderAssetCpuArtifactLease {
    manifest: Arc<RenderArtifactManifest>,
    manifest_ticket: RenderArtifactManifestTicket,
    blocks: RenderAssetCpuBlockLease,
}

impl RenderAssetCpuArtifactLease {
    fn new(
        manifest: Arc<RenderArtifactManifest>,
        manifest_ticket: RenderArtifactManifestTicket,
        blocks: RenderAssetCpuBlockLease,
    ) -> Self {
        Self {
            manifest,
            manifest_ticket,
            blocks,
        }
    }

    pub(crate) fn manifest(&self) -> &Arc<RenderArtifactManifest> {
        &self.manifest
    }

    pub(crate) fn blocks(&self) -> &[crate::asset::artifact::RenderArtifactDecodedBlock] {
        self.blocks.blocks()
    }

    pub(crate) const fn ticket(&self) -> RenderAssetResidencyTicket {
        self.blocks.ticket()
    }

    pub(crate) const fn encoded_bytes(&self) -> u64 {
        self.blocks.encoded_bytes()
    }

    pub(crate) const fn decoded_bytes(&self) -> u64 {
        self.blocks.decoded_bytes()
    }

    pub(crate) fn block_batch_count(&self) -> usize {
        self.blocks.batch_count()
    }

    pub(crate) fn manifest_ticket_id(&self) -> u64 {
        self.manifest_ticket.id()
    }
}

enum RenderAssetSemanticLoadState {
    Manifest(RenderArtifactManifestTicket),
    ReadyManifest {
        manifest: Arc<RenderArtifactManifest>,
        manifest_ticket: RenderArtifactManifestTicket,
    },
    Blocks {
        manifest: Arc<RenderArtifactManifest>,
        manifest_ticket: RenderArtifactManifestTicket,
        load: RenderAssetSemanticBlockLoad,
    },
}

pub(crate) struct RenderAssetSemanticLoad {
    ticket: RenderAssetResidencyTicket,
    priority: RenderArtifactIoPriority,
    deadline: Option<Instant>,
    state: RenderAssetSemanticLoadState,
}

impl RenderAssetSemanticLoad {
    pub(crate) fn begin(
        ticket: RenderAssetResidencyTicket,
        target_platform: Arc<str>,
        manifest_loader: &RenderArtifactManifestLoader,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
    ) -> Result<Self, RenderAssetSemanticLoadError> {
        if ticket.route() != RenderAssetResidencyRoute::SemanticBlocks {
            return Err(RenderAssetSemanticLoadError::UnsupportedRoute {
                actual: ticket.route(),
            });
        }
        let request = RenderArtifactManifestRequest::new(
            ticket.resource(),
            ticket.asset_revision(),
            target_platform,
            priority,
        );
        let request = match deadline {
            Some(deadline) => request.with_deadline(deadline),
            None => request,
        };
        let manifest_ticket = manifest_loader.request(request)?;
        Ok(Self {
            ticket,
            priority,
            deadline,
            state: RenderAssetSemanticLoadState::Manifest(manifest_ticket),
        })
    }

    pub(crate) fn advance(
        self,
        block_loader: &crate::asset::artifact::RenderArtifactBlockLoader,
    ) -> Result<RenderAssetSemanticLoadAdvance, RenderAssetSemanticLoadError> {
        let Self {
            ticket,
            priority,
            deadline,
            state,
        } = self;
        match state {
            RenderAssetSemanticLoadState::Manifest(manifest_ticket) => {
                match manifest_ticket.poll() {
                    RenderArtifactManifestPoll::Pending(stage) => {
                        Ok(RenderAssetSemanticLoadAdvance::Pending(
                            Self {
                                ticket,
                                priority,
                                deadline,
                                state: RenderAssetSemanticLoadState::Manifest(manifest_ticket),
                            },
                            RenderAssetSemanticLoadStage::Manifest(stage),
                        ))
                    }
                    RenderArtifactManifestPoll::Ready(manifest) => Self::begin_blocks(
                        ticket,
                        priority,
                        deadline,
                        manifest,
                        manifest_ticket,
                        block_loader,
                    ),
                    RenderArtifactManifestPoll::Failed(failure) => {
                        Err(RenderAssetSemanticLoadError::ManifestFailed {
                            code: failure.code(),
                            detail: failure.detail().into(),
                        })
                    }
                    RenderArtifactManifestPoll::Cancelled(reason) => {
                        Err(RenderAssetSemanticLoadError::ManifestCancelled { reason })
                    }
                }
            }
            RenderAssetSemanticLoadState::ReadyManifest {
                manifest,
                manifest_ticket,
            } => Self::begin_blocks(
                ticket,
                priority,
                deadline,
                manifest,
                manifest_ticket,
                block_loader,
            ),
            RenderAssetSemanticLoadState::Blocks {
                manifest,
                manifest_ticket,
                load,
            } => match load.advance(block_loader)? {
                RenderAssetSemanticBlockLoadAdvance::Pending(load, stage) => {
                    Ok(RenderAssetSemanticLoadAdvance::Pending(
                        Self {
                            ticket,
                            priority,
                            deadline,
                            state: RenderAssetSemanticLoadState::Blocks {
                                manifest,
                                manifest_ticket,
                                load,
                            },
                        },
                        RenderAssetSemanticLoadStage::Blocks(stage),
                    ))
                }
                RenderAssetSemanticBlockLoadAdvance::Deferred(load, error) => {
                    Ok(RenderAssetSemanticLoadAdvance::Deferred(
                        Self {
                            ticket,
                            priority,
                            deadline,
                            state: RenderAssetSemanticLoadState::Blocks {
                                manifest,
                                manifest_ticket,
                                load,
                            },
                        },
                        error,
                    ))
                }
                RenderAssetSemanticBlockLoadAdvance::Ready(blocks) => {
                    Ok(RenderAssetSemanticLoadAdvance::Ready(
                        RenderAssetCpuArtifactLease::new(manifest, manifest_ticket, blocks),
                    ))
                }
            },
        }
    }

    fn begin_blocks(
        ticket: RenderAssetResidencyTicket,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
        manifest: Arc<RenderArtifactManifest>,
        manifest_ticket: RenderArtifactManifestTicket,
        block_loader: &crate::asset::artifact::RenderArtifactBlockLoader,
    ) -> Result<RenderAssetSemanticLoadAdvance, RenderAssetSemanticLoadError> {
        match RenderAssetSemanticBlockLoad::begin(
            ticket,
            manifest.as_ref(),
            block_loader,
            priority,
            deadline,
        ) {
            Ok(load) => Ok(RenderAssetSemanticLoadAdvance::Pending(
                RenderAssetSemanticLoad {
                    ticket,
                    priority,
                    deadline,
                    state: RenderAssetSemanticLoadState::Blocks {
                        manifest,
                        manifest_ticket,
                        load,
                    },
                },
                RenderAssetSemanticLoadStage::Blocks(
                    crate::asset::artifact::RenderArtifactBlockLoadStage::QueuedIo,
                ),
            )),
            Err(RenderAssetSemanticBlockLoadError::Admission(error)) => {
                Ok(RenderAssetSemanticLoadAdvance::Deferred(
                    RenderAssetSemanticLoad {
                        ticket,
                        priority,
                        deadline,
                        state: RenderAssetSemanticLoadState::ReadyManifest {
                            manifest,
                            manifest_ticket,
                        },
                    },
                    error,
                ))
            }
            Err(error) => Err(RenderAssetSemanticLoadError::Blocks(error)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum RenderAssetSemanticLoadError {
    #[error("render residency route {actual:?} is not a semantic-block route")]
    UnsupportedRoute { actual: RenderAssetResidencyRoute },
    #[error(transparent)]
    ManifestAdmission(#[from] RenderArtifactManifestAdmissionError),
    #[error("render semantic manifest failed with {code:?}: {detail}")]
    ManifestFailed {
        code: RenderArtifactManifestFailureCode,
        detail: Arc<str>,
    },
    #[error("render semantic manifest was cancelled: {reason:?}")]
    ManifestCancelled {
        reason: RenderArtifactManifestCancelReason,
    },
    #[error(transparent)]
    Blocks(#[from] RenderAssetSemanticBlockLoadError),
}
