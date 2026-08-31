use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use winit::event_loop::EventLoop;
use zircon_runtime::asset::project::ResolvedProjectPath;

use super::super::presenter::RuntimeUiSurfacePresenterFactory;
use super::constants::{DEFAULT_HOST_WINDOW_HEIGHT, DEFAULT_HOST_WINDOW_WIDTH};
use super::handle::HostWindowHandle;
use super::metadata::platform_error;
use super::UiHostWindow;
use crate::core::jobs::{EditorJobSystem, JobId};
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::primitives::{CloseRequestResponse, PhysicalSize, PlatformError};

impl UiHostWindow {
    pub(crate) fn new() -> Result<Self, PlatformError> {
        let event_wake = super::event_wake::HostEventLoopWake::default();
        let visual_asset_wake = super::event_wake::HostEventLoopWake::default();
        let attention = super::attention::HostWindowAttention::new(event_wake.callback());
        Ok(Self {
            state: Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
                DEFAULT_HOST_WINDOW_WIDTH,
                DEFAULT_HOST_WINDOW_HEIGHT,
            )))),
            event_wake,
            visual_asset_wake,
            attention,
            fatal_failure: Rc::new(RefCell::new(None)),
            first_present_notification: Rc::new(
                super::first_present::FirstPresentNotification::default(),
            ),
            native_focus_observer: Rc::new(
                super::focus_observer::NativeWindowFocusObserver::default(),
            ),
            runtime_presenter_factory: Rc::new(RefCell::new(None)),
            profile_artifact_job_owner: Rc::new(super::ProfileArtifactJobOwner::default()),
            visual_asset_load_binding_owner: Rc::new(super::VisualAssetLoadBindingOwner::default()),
            direct_viewport_products_active: Rc::new(std::cell::Cell::new(false)),
        })
    }

    pub(crate) fn clone_strong(&self) -> Self {
        self.clone()
    }

    pub(in crate::ui::retained_host) fn set_runtime_presenter_factory(
        &self,
        factory: Arc<dyn RuntimeUiSurfacePresenterFactory>,
    ) {
        *self.runtime_presenter_factory.borrow_mut() = Some(factory);
    }

    pub(in crate::ui::retained_host) fn bind_profile_artifact_jobs(&self, jobs: EditorJobSystem) {
        self.profile_artifact_job_owner.bind(jobs);
    }

    pub(in crate::ui::retained_host) fn bind_visual_asset_jobs(
        &self,
        jobs: EditorJobSystem,
        completion_wake: zircon_runtime::core::framework::channel::ChannelWakeCallback,
    ) {
        let binding_epoch =
            super::super::paint_template_nodes::bind_visual_asset_loader(jobs, completion_wake);
        self.visual_asset_load_binding_owner.bind(binding_epoch);
    }

    pub(in crate::ui::retained_host::host_contract) fn profile_artifact_jobs(
        &self,
    ) -> Option<EditorJobSystem> {
        self.profile_artifact_job_owner.jobs()
    }

    pub(in crate::ui::retained_host::host_contract) fn track_profile_artifact_job(
        &self,
        id: JobId,
    ) {
        self.profile_artifact_job_owner.track(id);
    }

    pub(in crate::ui::retained_host::host_contract) fn runtime_presenter_factory(
        &self,
    ) -> Option<Arc<dyn RuntimeUiSurfacePresenterFactory>> {
        self.runtime_presenter_factory.borrow().clone()
    }

    pub(in crate::ui::retained_host::host_contract) fn set_direct_viewport_products_active(
        &self,
        active: bool,
    ) {
        self.direct_viewport_products_active.set(active);
    }

    pub(in crate::ui::retained_host) fn window_attention(
        &self,
    ) -> super::attention::HostWindowAttention {
        self.attention.clone()
    }

    pub(in crate::ui::retained_host::host_contract) fn has_window_attention_request(&self) -> bool {
        self.attention.is_requested()
    }

    pub(in crate::ui::retained_host::host_contract) fn take_window_attention_request(
        &self,
    ) -> bool {
        self.attention.take_request()
    }

    pub(in crate::ui::retained_host) fn direct_viewport_products_active(&self) -> bool {
        self.direct_viewport_products_active.get()
    }

    pub(crate) fn show(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = true;
        Ok(())
    }

    pub(crate) fn hide(&self) -> Result<(), PlatformError> {
        self.state.borrow_mut().window_visible = false;
        Ok(())
    }

    pub(crate) fn run(&self) -> Result<(), PlatformError> {
        let event_loop = EventLoop::new().map_err(platform_error)?;
        let proxy = event_loop.create_proxy();
        self.event_wake.install_proxy(proxy.clone());
        self.visual_asset_wake.install_proxy(proxy);
        let app = super::event_loop::UiHostWindowEventLoop::new(self.clone_strong());
        let result = event_loop.run_app(app).map_err(platform_error);
        self.event_wake.clear_proxy();
        self.visual_asset_wake.clear_proxy();
        result
    }

    pub(crate) fn window(&self) -> HostWindowHandle {
        HostWindowHandle {
            state: self.state.clone(),
            attention: self.attention.clone(),
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn close_requested_response(
        &self,
    ) -> CloseRequestResponse {
        let callback = self.state.borrow().close_requested.clone();
        callback
            .as_ref()
            .map(|callback| callback())
            .unwrap_or(CloseRequestResponse::HideWindow)
    }

    pub(crate) fn global<T>(&self) -> T
    where
        T: HostContractGlobal,
    {
        T::from_state(self.state.clone())
    }

    pub(crate) fn request_exit(&self) {
        let mut state = self.state.borrow_mut();
        state.window_visible = false;
        state.exit_requested = true;
    }

    pub(crate) fn set_exit_after_first_presented_frame(&self, exit: bool) {
        self.state.borrow_mut().exit_after_first_presented_frame = exit;
    }

    pub(crate) fn set_first_presented_frame_capture_path(&self, path: Option<ResolvedProjectPath>) {
        self.state.borrow_mut().first_presented_frame_capture_path = path;
    }

    pub(crate) fn take_first_presented_frame_capture_error(&self) -> Option<String> {
        self.state
            .borrow_mut()
            .first_presented_frame_capture_error
            .take()
    }

    /// Registers a one-shot callback that runs only after the native presenter submits its first
    /// frame successfully. It is intentionally unavailable to ordinary UI refresh code.
    pub(in crate::ui::retained_host) fn on_first_presented(
        &self,
        callback: impl FnOnce() -> Result<(), String> + 'static,
    ) -> Result<(), super::FirstPresentNotificationError> {
        self.first_present_notification.register(callback)
    }

    pub(in crate::ui::retained_host::host_contract) fn notify_first_presented(
        &self,
    ) -> Result<(), String> {
        self.first_present_notification.notify()
    }

    pub(in crate::ui::retained_host) fn on_native_window_focused(
        &self,
        callback: impl Fn() + 'static,
    ) -> Result<(), super::NativeWindowFocusObserverError> {
        self.native_focus_observer.register(callback)
    }

    pub(in crate::ui::retained_host::host_contract) fn notify_native_window_focused(&self) {
        self.native_focus_observer.notify();
    }

    pub(in crate::ui::retained_host) fn take_fatal_failure(
        &self,
    ) -> Option<super::failure::EditorHostWindowFailure> {
        self.fatal_failure.borrow_mut().take()
    }

    pub(in crate::ui::retained_host::host_contract) fn report_fatal_failure(
        &self,
        component: &'static str,
        requested: impl std::fmt::Display,
        cause: impl std::fmt::Display,
        recovery: &'static str,
    ) {
        let failure =
            super::failure::EditorHostWindowFailure::new(component, requested, cause, recovery);
        self.record_host_diagnostic(HostWindowDiagnosticSeverity::Error, failure.to_string());
        let mut recorded_failure = self.fatal_failure.borrow_mut();
        if recorded_failure.is_none() {
            *recorded_failure = Some(failure);
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn record_first_presented_frame_capture_error(
        &self,
        error: impl std::fmt::Display,
    ) {
        self.state.borrow_mut().first_presented_frame_capture_error = Some(error.to_string());
    }

    pub(in crate::ui::retained_host::host_contract) fn exit_after_first_presented_frame(
        &self,
    ) -> bool {
        self.state.borrow().exit_after_first_presented_frame
    }

    #[cfg(test)]
    pub(crate) fn exit_requested_for_test(&self) -> bool {
        self.state.borrow().exit_requested
    }
}
