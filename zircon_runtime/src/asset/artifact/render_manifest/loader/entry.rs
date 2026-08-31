use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::runtime::TaskHandle;

use super::super::RenderArtifactBlockDescriptor;
use super::contract::{
    RenderArtifactBlockCancelReason, RenderArtifactBlockFailure, RenderArtifactBlockLoadStage,
    RenderArtifactBlockPoll, RenderArtifactDecodedBlock,
};

pub(super) struct RenderArtifactBlockEntry {
    descriptor: RenderArtifactBlockDescriptor,
    retained_bytes: usize,
    state: Mutex<RenderArtifactBlockEntryState>,
}

struct RenderArtifactBlockEntryState {
    outcome: RenderArtifactBlockEntryOutcome,
    ticket_count: usize,
    tasks: Vec<TaskHandle>,
}

#[derive(Clone)]
enum RenderArtifactBlockEntryOutcome {
    Pending(RenderArtifactBlockLoadStage),
    Ready(Arc<[u8]>),
    Failed(Arc<RenderArtifactBlockFailure>),
    Cancelled(RenderArtifactBlockCancelReason),
}

impl RenderArtifactBlockEntry {
    pub(super) fn new(
        descriptor: RenderArtifactBlockDescriptor,
        retained_bytes: usize,
        ticket_count: usize,
    ) -> Self {
        Self {
            descriptor,
            retained_bytes,
            state: Mutex::new(RenderArtifactBlockEntryState {
                outcome: RenderArtifactBlockEntryOutcome::Pending(
                    RenderArtifactBlockLoadStage::QueuedIo,
                ),
                ticket_count,
                tasks: Vec::with_capacity(2),
            }),
        }
    }

    pub(super) const fn descriptor(&self) -> &RenderArtifactBlockDescriptor {
        &self.descriptor
    }

    pub(super) const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(super) fn poll(
        &self,
        descriptor: &RenderArtifactBlockDescriptor,
    ) -> RenderArtifactBlockPoll {
        match self.lock().outcome.clone() {
            RenderArtifactBlockEntryOutcome::Pending(stage) => {
                RenderArtifactBlockPoll::Pending(stage)
            }
            RenderArtifactBlockEntryOutcome::Ready(bytes) => RenderArtifactBlockPoll::Ready(
                RenderArtifactDecodedBlock::new(descriptor.clone(), bytes),
            ),
            RenderArtifactBlockEntryOutcome::Failed(failure) => {
                RenderArtifactBlockPoll::Failed(failure)
            }
            RenderArtifactBlockEntryOutcome::Cancelled(reason) => {
                RenderArtifactBlockPoll::Cancelled(reason)
            }
        }
    }

    pub(super) fn ticket_count(&self) -> usize {
        self.lock().ticket_count
    }

    pub(super) fn add_tickets(&self, count: usize) {
        let mut state = self.lock();
        state.ticket_count = state.ticket_count.saturating_add(count);
    }

    pub(super) fn remove_ticket(&self) -> bool {
        let mut state = self.lock();
        state.ticket_count = state.ticket_count.saturating_sub(1);
        state.ticket_count == 0
    }

    pub(super) fn install_task(&self, task: TaskHandle) {
        let mut state = self.lock();
        if matches!(
            &state.outcome,
            RenderArtifactBlockEntryOutcome::Ready(_)
                | RenderArtifactBlockEntryOutcome::Failed(_)
                | RenderArtifactBlockEntryOutcome::Cancelled(_)
        ) {
            return;
        }
        state.tasks.retain(|task| !task.status().is_terminal());
        state.tasks.push(task);
    }

    pub(super) fn begin_io(&self) -> bool {
        self.transition_pending(
            RenderArtifactBlockLoadStage::QueuedIo,
            RenderArtifactBlockLoadStage::Reading,
        )
    }

    pub(super) fn queue_decode(&self) -> bool {
        self.transition_pending(
            RenderArtifactBlockLoadStage::Reading,
            RenderArtifactBlockLoadStage::QueuedDecode,
        )
    }

    pub(super) fn begin_decode(&self) -> bool {
        self.transition_pending(
            RenderArtifactBlockLoadStage::QueuedDecode,
            RenderArtifactBlockLoadStage::Decoding,
        )
    }

    pub(super) fn complete(&self, bytes: Arc<[u8]>) -> bool {
        let mut state = self.lock();
        if !matches!(
            &state.outcome,
            RenderArtifactBlockEntryOutcome::Pending(RenderArtifactBlockLoadStage::Reading)
                | RenderArtifactBlockEntryOutcome::Pending(RenderArtifactBlockLoadStage::Decoding)
        ) {
            return false;
        }
        state.outcome = RenderArtifactBlockEntryOutcome::Ready(bytes);
        true
    }

    pub(super) fn fail(&self, failure: RenderArtifactBlockFailure) -> bool {
        let mut state = self.lock();
        if !matches!(&state.outcome, RenderArtifactBlockEntryOutcome::Pending(_)) {
            return false;
        }
        state.outcome = RenderArtifactBlockEntryOutcome::Failed(Arc::new(failure));
        true
    }

    pub(super) fn cancel(&self, reason: RenderArtifactBlockCancelReason) -> bool {
        let tasks = {
            let mut state = self.lock();
            if !matches!(&state.outcome, RenderArtifactBlockEntryOutcome::Pending(_)) {
                return false;
            }
            state.outcome = RenderArtifactBlockEntryOutcome::Cancelled(reason);
            std::mem::take(&mut state.tasks)
        };
        drop(tasks);
        true
    }

    fn transition_pending(
        &self,
        expected: RenderArtifactBlockLoadStage,
        next: RenderArtifactBlockLoadStage,
    ) -> bool {
        let mut state = self.lock();
        if !matches!(&state.outcome, RenderArtifactBlockEntryOutcome::Pending(stage) if *stage == expected)
        {
            return false;
        }
        state.outcome = RenderArtifactBlockEntryOutcome::Pending(next);
        true
    }

    fn lock(&self) -> MutexGuard<'_, RenderArtifactBlockEntryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
