use std::num::NonZeroU64;

use zircon_runtime::asset::project::ResolvedProjectPath;
use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime::platform::EventLoopPolicy;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::entry) struct RuntimeEntryAppConfig {
    pub(in crate::entry::runtime_entry_app) window_descriptor: WindowDescriptor,
    pub(in crate::entry::runtime_entry_app) event_loop_policy: EventLoopPolicy,
    pub(in crate::entry::runtime_entry_app) window_lifecycle_policy: WindowLifecyclePolicy,
    pub(in crate::entry::runtime_entry_app) exit_after_presented_frames: Option<NonZeroU64>,
    pub(in crate::entry::runtime_entry_app) first_frame_capture_path: Option<ResolvedProjectPath>,
    pub(in crate::entry::runtime_entry_app) require_persisted_scene_diagnostics: bool,
    pub(in crate::entry::runtime_entry_app) reference_cpu_presenter: bool,
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
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::*;

    #[test]
    fn default_runtime_entry_app_config_uses_visible_game_window() {
        let config = RuntimeEntryAppConfig::default();

        assert!(config.window_descriptor.primary_window.is_some());
        assert!(config.window_descriptor.visible);
        assert_eq!(config.event_loop_policy, EventLoopPolicy::Game);
        assert!(config.window_lifecycle_policy.should_close_on_request());
        assert!(config
            .window_lifecycle_policy
            .should_exit_after_primary_close());
        assert!(!config.exit_after_first_presented_frame());
        assert!(!config.reference_cpu_presenter());
    }

    #[test]
    fn runtime_entry_app_config_can_select_absent_primary_window_policy() {
        let config = RuntimeEntryAppConfig::default()
            .with_window_descriptor(WindowDescriptor::default().without_primary_window())
            .with_event_loop_policy(EventLoopPolicy::Headless);

        assert_eq!(config.window_descriptor.primary_window, None);
        assert!(!config.window_descriptor.visible);
        assert_eq!(config.event_loop_policy, EventLoopPolicy::Headless);
    }

    #[test]
    fn runtime_entry_app_config_can_disable_close_when_requested_policy() {
        let config = RuntimeEntryAppConfig::default().with_close_when_requested(false);

        assert!(!config.window_lifecycle_policy.close_when_requested);
        assert!(!config
            .window_lifecycle_policy()
            .should_exit_after_primary_close());
    }

    #[test]
    fn runtime_entry_app_config_can_override_window_lifecycle_policy() {
        let policy = WindowLifecyclePolicy::default().with_close_when_requested(false);
        let config = RuntimeEntryAppConfig::default().with_window_lifecycle_policy(policy);

        assert_eq!(config.window_lifecycle_policy(), policy);
    }

    #[test]
    fn runtime_entry_app_config_can_exit_after_first_presented_frame() {
        let config = RuntimeEntryAppConfig::default().with_exit_after_first_presented_frame(true);

        assert!(config.exit_after_first_presented_frame());
    }

    #[test]
    fn runtime_entry_app_config_can_exit_after_a_requested_presented_frame_count() {
        let limit = NonZeroU64::new(120).unwrap();
        let config = RuntimeEntryAppConfig::default().with_exit_after_presented_frames(limit);

        assert_eq!(config.exit_after_presented_frames(), Some(limit));
        assert!(!config.exit_after_first_presented_frame());
    }

    #[test]
    fn runtime_entry_app_config_can_request_a_first_frame_capture() {
        let path = std::path::PathBuf::from("E:/evidence/runtime-first-frame.png");
        let resolved_path = zircon_runtime::asset::project::ProjectPaths::resolve_path(&path)
            .expect("capture path should resolve");
        let config = RuntimeEntryAppConfig::default()
            .with_first_frame_capture_path(Some(resolved_path.clone()));

        assert_eq!(config.first_frame_capture_path(), Some(&resolved_path));
    }

    #[test]
    fn runtime_entry_app_config_requires_persisted_scene_diagnostics_only_when_enabled() {
        assert!(
            !RuntimeEntryAppConfig::default().require_persisted_scene_diagnostics(),
            "the F0 no-project startup path must not require F2 scene diagnostics"
        );

        let config = RuntimeEntryAppConfig::default().with_persisted_scene_diagnostics(true);

        assert!(config.require_persisted_scene_diagnostics());
    }

    #[test]
    fn runtime_entry_app_config_requires_explicit_reference_cpu_presenter_opt_in() {
        let config = RuntimeEntryAppConfig::default().with_reference_cpu_presenter(true);

        assert!(config.reference_cpu_presenter());
    }
}
