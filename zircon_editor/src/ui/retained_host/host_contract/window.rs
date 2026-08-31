use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use super::globals::HostContractState;
use super::presenter::RuntimeUiSurfacePresenterFactory;
use crate::core::jobs::{EditorJobSystem, JobId};
use zircon_runtime_interface::ui::dispatch::UiWindowId;

mod attention;
mod capture;
mod constants;
mod diagnostics;
mod event_loop;
mod event_wake;
mod failure;
mod first_present;
mod focus_observer;
mod handle;
mod lifecycle;
mod metadata;
mod presentation;
mod redraw;
#[cfg(test)]
mod test_support;
mod text_input;

pub(crate) use first_present::FirstPresentNotificationError;
pub(crate) use focus_observer::NativeWindowFocusObserverError;
pub(crate) use handle::{HostWindowHandle, HostWindowSnapshot};

pub(crate) fn primary_host_window_id() -> UiWindowId {
    UiWindowId::new(constants::NATIVE_HOST_WINDOW_ID)
}

#[derive(Clone)]
pub(crate) struct UiHostWindow {
    state: Rc<RefCell<HostContractState>>,
    event_wake: event_wake::HostEventLoopWake,
    visual_asset_wake: event_wake::HostEventLoopWake,
    attention: attention::HostWindowAttention,
    fatal_failure: Rc<RefCell<Option<failure::EditorHostWindowFailure>>>,
    first_present_notification: Rc<first_present::FirstPresentNotification>,
    native_focus_observer: Rc<focus_observer::NativeWindowFocusObserver>,
    runtime_presenter_factory: Rc<RefCell<Option<Arc<dyn RuntimeUiSurfacePresenterFactory>>>>,
    profile_artifact_job_owner: Rc<ProfileArtifactJobOwner>,
    visual_asset_load_binding_owner: Rc<VisualAssetLoadBindingOwner>,
    direct_viewport_products_active: Rc<Cell<bool>>,
}

#[derive(Default)]
struct VisualAssetLoadBindingOwner {
    binding_epoch: Cell<Option<u64>>,
}

impl VisualAssetLoadBindingOwner {
    fn bind(&self, binding_epoch: u64) {
        self.binding_epoch.set(Some(binding_epoch));
    }
}

impl Drop for VisualAssetLoadBindingOwner {
    fn drop(&mut self) {
        if let Some(binding_epoch) = self.binding_epoch.get() {
            super::paint_template_nodes::unbind_visual_asset_loader(binding_epoch);
        }
    }
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
