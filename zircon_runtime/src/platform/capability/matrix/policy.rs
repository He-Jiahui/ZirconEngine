use crate::RuntimeTargetMode;

use super::super::backends::EventLoopPolicy;
use super::PlatformCapabilityMatrix;
use crate::platform::PlatformTarget;

impl PlatformCapabilityMatrix {
    pub(super) fn event_loop_policy(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
    ) -> EventLoopPolicy {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            EventLoopPolicy::Headless
        } else if target.is_mobile() {
            EventLoopPolicy::Mobile
        } else if target_mode == RuntimeTargetMode::EditorHost {
            EventLoopPolicy::DesktopApp
        } else {
            EventLoopPolicy::Game
        }
    }

    pub(super) fn explicit_event_loop_policy(
        self,
        target: PlatformTarget,
        target_mode: RuntimeTargetMode,
        requested: EventLoopPolicy,
    ) -> EventLoopPolicy {
        if target_mode == RuntimeTargetMode::ServerRuntime || target == PlatformTarget::Headless {
            return EventLoopPolicy::Headless;
        }

        if requested == EventLoopPolicy::Headless {
            self.event_loop_policy(target, target_mode)
        } else {
            requested
        }
    }
}
