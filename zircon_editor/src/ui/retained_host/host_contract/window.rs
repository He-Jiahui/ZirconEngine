use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::globals::HostContractState;
use super::presenter::RuntimeUiSurfacePresenterFactory;
use crate::core::jobs::{EditorJobSystem, JobId};

mod attention;
mod capture;
mod constants;
mod diagnostics;
mod event_loop;
mod event_wake;
mod failure;
mod handle;
mod lifecycle;
mod metadata;
mod presentation;
mod redraw;
mod resize_reflow;
#[cfg(test)]
mod test_support;
mod text_input;

pub(crate) use handle::{HostWindowHandle, HostWindowSnapshot};

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
    event_wake: event_wake::HostEventLoopWake,
    attention: attention::HostWindowAttention,
    fatal_failure: Rc<RefCell<Option<failure::EditorHostWindowFailure>>>,
    runtime_presenter_factory: Rc<RefCell<Option<Arc<dyn RuntimeUiSurfacePresenterFactory>>>>,
    profile_artifact_job_owner: Rc<ProfileArtifactJobOwner>,
    direct_viewport_products_active: Rc<Cell<bool>>,
}

/// Keeps the one-shot profile artifact job tied to the host lifetime without
/// introducing a retained-host worker or queue.
#[derive(Default)]
struct ProfileArtifactJobOwner {
    jobs: RefCell<Option<EditorJobSystem>>,
    active_job: Cell<Option<JobId>>,
}

impl ProfileArtifactJobOwner {
    fn bind(&self, jobs: EditorJobSystem) {
        let active_job = self.active_job.replace(None);
        let previous_jobs = self.jobs.replace(Some(jobs));
        if let (Some(jobs), Some(id)) = (previous_jobs, active_job) {
            jobs.cancel(id);
        }
    }

    fn jobs(&self) -> Option<EditorJobSystem> {
        self.jobs.borrow().clone()
    }

    fn track(&self, id: JobId) {
        let previous_job = self.active_job.replace(Some(id));
        if let (Some(jobs), Some(previous_id)) = (self.jobs(), previous_job) {
            jobs.cancel(previous_id);
        }
    }
}

impl Drop for ProfileArtifactJobOwner {
    fn drop(&mut self) {
        let Some(id) = self.active_job.get() else {
            return;
        };
        let Some(jobs) = self.jobs.get_mut().as_ref() else {
            return;
        };
        jobs.cancel(id);
    }
}

#[cfg(test)]
mod profile_artifact_job_tests;
#[cfg(test)]
mod tests;
