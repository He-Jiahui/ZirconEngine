use zircon_runtime::core::framework::window::{WindowDescriptor, WindowLifecyclePolicy};
use zircon_runtime::platform::EventLoopPolicy;

#[derive(Clone, Debug, PartialEq)]
pub(in crate::entry) struct RuntimeEntryAppConfig {
    pub(in crate::entry::runtime_entry_app) window_descriptor: WindowDescriptor,
    pub(in crate::entry::runtime_entry_app) event_loop_policy: EventLoopPolicy,
    pub(in crate::entry::runtime_entry_app) window_lifecycle_policy: WindowLifecyclePolicy,
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

    pub(in crate::entry) fn window_descriptor(&self) -> &WindowDescriptor {
        &self.window_descriptor
    }

    pub(in crate::entry) fn event_loop_policy(&self) -> EventLoopPolicy {
        self.event_loop_policy
    }

    pub(in crate::entry) fn close_when_requested(&self) -> bool {
        self.window_lifecycle_policy.close_when_requested
    }

    pub(in crate::entry) fn window_lifecycle_policy(&self) -> WindowLifecyclePolicy {
        self.window_lifecycle_policy
    }
}

impl Default for RuntimeEntryAppConfig {
    fn default() -> Self {
        Self {
            window_descriptor: WindowDescriptor::default(),
            event_loop_policy: EventLoopPolicy::Game,
            window_lifecycle_policy: WindowLifecyclePolicy::default(),
        }
    }
}

#[cfg(test)]
mod tests {
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

        assert!(!config.close_when_requested());
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
}
