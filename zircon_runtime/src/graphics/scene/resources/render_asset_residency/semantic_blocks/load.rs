use std::time::Instant;

use crate::asset::artifact::{
    RenderArtifactBlockLoadStage, RenderArtifactBlockLoader, RenderArtifactBlockPoll,
    RenderArtifactBlockTicketBatch, RenderArtifactDecodedBlock, RenderArtifactIoPriority,
    RenderArtifactLoadPlan, RenderArtifactLoadScope, RenderArtifactManifest,
};

use super::super::{
    RenderAssetResidencyRoute, RenderAssetResidencyScope, RenderAssetResidencyTicket,
};
use super::contract::{
    RenderAssetCpuBlockLease, RenderAssetSemanticBlockLoadAdvance,
    RenderAssetSemanticBlockLoadError,
};

struct ActiveSemanticBlockBatch {
    tickets: RenderArtifactBlockTicketBatch,
    blocks: Vec<Option<RenderArtifactDecodedBlock>>,
}

pub(crate) struct RenderAssetSemanticBlockLoad {
    ticket: RenderAssetResidencyTicket,
    plan: RenderArtifactLoadPlan,
    priority: RenderArtifactIoPriority,
    deadline: Option<Instant>,
    next_batch_index: usize,
    active: Option<ActiveSemanticBlockBatch>,
    completed_ticket_batches: Vec<RenderArtifactBlockTicketBatch>,
    decoded_blocks: Vec<RenderArtifactDecodedBlock>,
}

impl RenderAssetSemanticBlockLoad {
    pub(crate) fn begin(
        ticket: RenderAssetResidencyTicket,
        manifest: &RenderArtifactManifest,
        loader: &RenderArtifactBlockLoader,
        priority: RenderArtifactIoPriority,
        deadline: Option<Instant>,
    ) -> Result<Self, RenderAssetSemanticBlockLoadError> {
        if ticket.route() != RenderAssetResidencyRoute::SemanticBlocks {
            return Err(RenderAssetSemanticBlockLoadError::UnsupportedRoute {
                actual: ticket.route(),
            });
        }
        if ticket.resource() != manifest.resource() {
            return Err(
                RenderAssetSemanticBlockLoadError::ManifestResourceMismatch {
                    expected: ticket.resource(),
                    actual: manifest.resource(),
                },
            );
        }
        if ticket.asset_revision() != manifest.asset_revision() {
            return Err(
                RenderAssetSemanticBlockLoadError::ManifestRevisionMismatch {
                    expected: ticket.asset_revision(),
                    actual: manifest.asset_revision(),
                },
            );
        }
        let scope = match ticket.scope() {
            RenderAssetResidencyScope::Bootstrap => RenderArtifactLoadScope::Bootstrap,
            RenderAssetResidencyScope::AllLods => RenderArtifactLoadScope::All,
        };
        let plan = manifest.load_plan(scope)?;
        let mut load = Self {
            ticket,
            decoded_blocks: Vec::with_capacity(plan.block_count()),
            completed_ticket_batches: Vec::with_capacity(plan.batches().len()),
            plan,
            priority,
            deadline,
            next_batch_index: 0,
            active: None,
        };
        load.request_next_batch(loader)?;
        Ok(load)
    }

    pub(crate) fn advance(
        mut self,
        loader: &RenderArtifactBlockLoader,
    ) -> Result<RenderAssetSemanticBlockLoadAdvance, RenderAssetSemanticBlockLoadError> {
        if self.active.is_none() {
            match self.request_next_batch(loader) {
                Ok(true) => {}
                Ok(false) => return Ok(RenderAssetSemanticBlockLoadAdvance::Ready(self.finish())),
                Err(error) => {
                    return Ok(RenderAssetSemanticBlockLoadAdvance::Deferred(self, error));
                }
            }
        }

        let Some(mut active) = self.active.take() else {
            return Err(RenderAssetSemanticBlockLoadError::IncompleteBatch);
        };
        let mut pending_stage = None;
        for (ticket, block) in active.tickets.tickets().iter().zip(&mut active.blocks) {
            if block.is_some() {
                continue;
            }
            match ticket.poll() {
                RenderArtifactBlockPoll::Pending(stage) => {
                    pending_stage = Some(furthest_stage(pending_stage, stage));
                }
                RenderArtifactBlockPoll::Ready(decoded) => *block = Some(decoded),
                RenderArtifactBlockPoll::Failed(failure) => {
                    return Err(RenderAssetSemanticBlockLoadError::BlockFailed {
                        code: failure.code(),
                        detail: failure.detail().into(),
                    });
                }
                RenderArtifactBlockPoll::Cancelled(reason) => {
                    return Err(RenderAssetSemanticBlockLoadError::BlockCancelled { reason });
                }
            }
        }
        if let Some(stage) = pending_stage {
            self.active = Some(active);
            return Ok(RenderAssetSemanticBlockLoadAdvance::Pending(self, stage));
        }

        for block in &mut active.blocks {
            let Some(block) = block.take() else {
                return Err(RenderAssetSemanticBlockLoadError::IncompleteBatch);
            };
            self.decoded_blocks.push(block);
        }
        self.completed_ticket_batches.push(active.tickets);
        match self.request_next_batch(loader) {
            Ok(true) => Ok(RenderAssetSemanticBlockLoadAdvance::Pending(
                self,
                RenderArtifactBlockLoadStage::QueuedIo,
            )),
            Ok(false) => Ok(RenderAssetSemanticBlockLoadAdvance::Ready(self.finish())),
            Err(error) => Ok(RenderAssetSemanticBlockLoadAdvance::Deferred(self, error)),
        }
    }

    fn request_next_batch(
        &mut self,
        loader: &RenderArtifactBlockLoader,
    ) -> Result<bool, crate::asset::artifact::RenderArtifactBlockAdmissionError> {
        let Some(batch) = self.plan.batches().get(self.next_batch_index) else {
            return Ok(false);
        };
        let tickets = loader.request_load_batch(batch, self.priority, self.deadline)?;
        self.next_batch_index = self.next_batch_index.saturating_add(1);
        self.active = Some(ActiveSemanticBlockBatch {
            blocks: std::iter::repeat_with(|| None)
                .take(tickets.tickets().len())
                .collect(),
            tickets,
        });
        Ok(true)
    }

    fn finish(self) -> RenderAssetCpuBlockLease {
        RenderAssetCpuBlockLease::new(
            self.ticket,
            self.decoded_blocks,
            self.plan.total_encoded_bytes(),
            self.plan.total_decoded_bytes(),
            self.completed_ticket_batches,
        )
    }
}

fn furthest_stage(
    current: Option<RenderArtifactBlockLoadStage>,
    candidate: RenderArtifactBlockLoadStage,
) -> RenderArtifactBlockLoadStage {
    match current {
        Some(current) if stage_rank(current) >= stage_rank(candidate) => current,
        _ => candidate,
    }
}

const fn stage_rank(stage: RenderArtifactBlockLoadStage) -> u8 {
    match stage {
        RenderArtifactBlockLoadStage::QueuedIo => 0,
        RenderArtifactBlockLoadStage::Reading => 1,
        RenderArtifactBlockLoadStage::QueuedDecode => 2,
        RenderArtifactBlockLoadStage::Decoding => 3,
    }
}
