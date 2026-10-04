use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::framework::scene::{
    SceneArtifactTerminal, SceneArtifactTicket, SceneArtifactWaitResult,
};
use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoFailure, BoundedKeyedIoLane, BoundedKeyedIoLimits,
    BoundedKeyedIoTicket, BoundedKeyedIoWork, BoundedKeyedIoWorkDeadline, JobScheduler, TaskPool,
};
use crate::scene::world::SceneProjectError;

pub(super) const MAX_SCENE_ARTIFACT_BYTES: usize = 64 * 1024 * 1024;
const MAX_PENDING_SCENE_ARTIFACTS: usize = 8;

pub(super) struct SceneArtifactIo {
    lane: BoundedKeyedIoLane,
    submit_gate: Mutex<()>,
    next_generation: AtomicU64,
}

impl SceneArtifactIo {
    pub(super) fn new(io_pool: TaskPool) -> Self {
        Self {
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(
                    MAX_PENDING_SCENE_ARTIFACTS + 1,
                    MAX_PENDING_SCENE_ARTIFACTS * MAX_SCENE_ARTIFACT_BYTES,
                ),
                JobScheduler::from_pool(io_pool),
            ),
            submit_gate: Mutex::new(()),
            next_generation: AtomicU64::new(0),
        }
    }

    pub(super) fn submit(
        &self,
        key: String,
        work: BoundedKeyedIoWork,
    ) -> Result<Arc<dyn SceneArtifactTicket>, SceneProjectError> {
        let _submit = self.lock_submit_gate();
        if self.lane.diagnostics().queue_entries >= MAX_PENDING_SCENE_ARTIFACTS {
            return Err(admission_error(
                BoundedKeyedIoAdmissionError::EntryCapacityExceeded,
            ));
        }
        let generation = self
            .next_generation
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        self.lane
            .try_admit(
                key,
                generation,
                MAX_SCENE_ARTIFACT_BYTES,
                BoundedKeyedIoWorkDeadline::none(),
                work,
            )
            .map_err(admission_error)
            .map(|admission| Arc::new(admission.activate()) as Arc<dyn SceneArtifactTicket>)
    }

    fn lock_submit_gate(&self) -> MutexGuard<'_, ()> {
        self.submit_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl SceneArtifactTicket for BoundedKeyedIoTicket {
    fn generation(&self) -> u64 {
        BoundedKeyedIoTicket::generation(self)
    }

    fn terminal(&self) -> Option<SceneArtifactTerminal> {
        BoundedKeyedIoTicket::terminal(self).map(project_terminal)
    }

    fn wait_until(&self, deadline: std::time::Instant) -> SceneArtifactWaitResult {
        match BoundedKeyedIoTicket::wait_until(self, deadline) {
            crate::core::runtime::BoundedKeyedIoWaitResult::Terminal(terminal) => {
                SceneArtifactWaitResult::Terminal(project_terminal(terminal))
            }
            crate::core::runtime::BoundedKeyedIoWaitResult::ObserverTimedOut => {
                SceneArtifactWaitResult::ObserverTimedOut
            }
        }
    }
}

impl fmt::Debug for SceneArtifactIo {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneArtifactIo")
            .field("diagnostics", &self.lane.diagnostics())
            .finish()
    }
}

impl Drop for SceneArtifactIo {
    fn drop(&mut self) {
        let _fence = self.lane.submit_fence(
            0,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok::<(), BoundedKeyedIoFailure>(())),
        );
        drop(self.lane.shutdown());
    }
}

fn admission_error(error: BoundedKeyedIoAdmissionError) -> SceneProjectError {
    SceneProjectError::SceneAsset(format!("scene artifact I/O admission failed: {error:?}"))
}

fn project_terminal(
    terminal: crate::core::runtime::BoundedKeyedIoTerminal,
) -> SceneArtifactTerminal {
    match terminal {
        crate::core::runtime::BoundedKeyedIoTerminal::Succeeded => SceneArtifactTerminal::Succeeded,
        crate::core::runtime::BoundedKeyedIoTerminal::Failed(failure) => {
            SceneArtifactTerminal::Failed { code: failure.code }
        }
        crate::core::runtime::BoundedKeyedIoTerminal::DeadlineBeforeStart => {
            SceneArtifactTerminal::DeadlineBeforeStart
        }
        crate::core::runtime::BoundedKeyedIoTerminal::CancelledBeforeStart => {
            SceneArtifactTerminal::CancelledBeforeStart
        }
        crate::core::runtime::BoundedKeyedIoTerminal::Superseded { successor } => {
            SceneArtifactTerminal::Superseded { successor }
        }
        crate::core::runtime::BoundedKeyedIoTerminal::Shutdown => SceneArtifactTerminal::Shutdown,
    }
}

#[cfg(test)]
#[path = "tests/scene_artifact_io.rs"]
mod tests;
