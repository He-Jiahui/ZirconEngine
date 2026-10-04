//! EntryRunner 交给 Winit 应用的运行前宿主策略。
//! 构造时消费并分发到窗口、帧调度与诊断所有者；运行中不重新解析路径。

use std::num::NonZeroU64;

use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime::platform::EventLoopPolicy;

#[derive(Clone, Debug, PartialEq)]
/// 由产品入口在创建事件循环应用前设定，供 RuntimeEntryApp 构造一次性消费。
pub(in crate::entry) struct RuntimeEntryAppConfig {
    pub(in crate::entry::runtime_entry_app) window_descriptor: WindowDescriptor,
    pub(in crate::entry::runtime_entry_app) event_loop_policy: EventLoopPolicy,
    pub(in crate::entry::runtime_entry_app) window_lifecycle_policy: WindowLifecyclePolicy,
    pub(in crate::entry::runtime_entry_app) exit_after_presented_frames: Option<NonZeroU64>,
    pub(in crate::entry::runtime_entry_app) first_frame_capture_path: Option<ResolvedProjectPath>,
    pub(in crate::entry::runtime_entry_app) require_persisted_scene_diagnostics: bool,
    pub(in crate::entry::runtime_entry_app) reference_cpu_presenter: bool,
    /// Native AppSession V2 is opt-in. False keeps the published Winit V1 text/cursor path.
    pub(in crate::entry::runtime_entry_app) native_ime_composition_requested: bool,
}

impl RuntimeEntryAppConfig {
    pub(in crate::entry) fn with_window_descriptor(
        mut self,
        window_descriptor: WindowDescriptor,
    ) -> Self {
        self.window_descriptor = window_descriptor;
        self
    }

    pub(in crate::entry) fn with_event_loop_policy(
        mut self,
        event_loop_policy: EventLoopPolicy,
    ) -> Self {
        self.event_loop_policy = event_loop_policy;
        self
    }

    #[cfg(test)]
    pub(in crate::entry) fn with_close_when_requested(
        mut self,
        close_when_requested: bool,
    ) -> Self {
        self.window_lifecycle_policy = self
            .window_lifecycle_policy
            .with_close_when_requested(close_when_requested);
        self
    }

    pub(in crate::entry) fn with_window_lifecycle_policy(
        mut self,
        window_lifecycle_policy: WindowLifecyclePolicy,
    ) -> Self {
        self.window_lifecycle_policy = window_lifecycle_policy;
        self
    }

    pub(in crate::entry) fn with_exit_after_first_presented_frame(mut self, exit: bool) -> Self {
        self.exit_after_presented_frames = exit.then_some(NonZeroU64::MIN);
        self
    }

    pub(in crate::entry) fn with_exit_after_presented_frames(mut self, limit: NonZeroU64) -> Self {
        self.exit_after_presented_frames = Some(limit);
        self
    }

    /// 仅接受入口已解析的物理目标路径，避免在窗口回调中改变路径基准。
    pub(in crate::entry) fn with_first_frame_capture_path(
        mut self,
        path: Option<ResolvedProjectPath>,
    ) -> Self {
        self.first_frame_capture_path = path;
        self
    }

    pub(in crate::entry) fn with_persisted_scene_diagnostics(mut self, require: bool) -> Self {
        self.require_persisted_scene_diagnostics = require;
        self
    }

    pub(in crate::entry) fn with_reference_cpu_presenter(mut self, enabled: bool) -> Self {
        self.reference_cpu_presenter = enabled;
        self
    }

    /// Requests the standalone AppSession V2 contract. The request is rejected before any
    /// producer or state-10 callback is activated when the loaded runtime or native adapter is
    /// unavailable; it never infers opt-in from a loaded symbol.
    pub(in crate::entry) fn with_native_ime_composition_requested(
        mut self,
        requested: bool,
    ) -> Self {
        self.native_ime_composition_requested = requested;
        self
    }

    #[cfg(test)]
    pub(in crate::entry) fn window_descriptor(&self) -> &WindowDescriptor {
        &self.window_descriptor
    }

    #[cfg(test)]
    pub(in crate::entry) fn event_loop_policy(&self) -> EventLoopPolicy {
        self.event_loop_policy
    }

    #[cfg(test)]
    pub(in crate::entry) fn window_lifecycle_policy(&self) -> WindowLifecyclePolicy {
        self.window_lifecycle_policy
    }

    #[cfg(test)]
    pub(in crate::entry) fn exit_after_first_presented_frame(&self) -> bool {
        self.exit_after_presented_frames == Some(NonZeroU64::MIN)
    }

    #[cfg(test)]
    pub(in crate::entry) fn exit_after_presented_frames(&self) -> Option<NonZeroU64> {
        self.exit_after_presented_frames
    }

    #[cfg(test)]
    pub(in crate::entry) fn first_frame_capture_path(&self) -> Option<&ResolvedProjectPath> {
        self.first_frame_capture_path.as_ref()
    }

    #[cfg(test)]
    pub(in crate::entry) fn require_persisted_scene_diagnostics(&self) -> bool {
        self.require_persisted_scene_diagnostics
    }

    #[cfg(test)]
    pub(in crate::entry) fn reference_cpu_presenter(&self) -> bool {
        self.reference_cpu_presenter
    }

    #[cfg(test)]
    pub(in crate::entry) fn native_ime_composition_requested(&self) -> bool {
        self.native_ime_composition_requested
    }
}

impl Default for RuntimeEntryAppConfig {
    fn default() -> Self {
        Self {
            window_descriptor: WindowDescriptor::default(),
            event_loop_policy: EventLoopPolicy::Game,
            window_lifecycle_policy: WindowLifecyclePolicy::default(),
            exit_after_presented_frames: None,
            first_frame_capture_path: None,
            require_persisted_scene_diagnostics: false,
            reference_cpu_presenter: false,
            native_ime_composition_requested: false,
        }
    }
}

#[cfg(test)]
#[path = "tests/app_config.rs"]
mod tests;
