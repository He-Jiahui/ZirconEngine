use winit::event_loop::{ActiveEventLoop, ControlFlow};
use zircon_runtime::platform::EventLoopPolicy;

use super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(super) fn apply_event_loop_policy(&self, event_loop: &dyn ActiveEventLoop) {
        event_loop.set_control_flow(winit_control_flow(self.event_loop_policy));
    }
}

fn winit_control_flow(policy: EventLoopPolicy) -> ControlFlow {
    match policy {
        EventLoopPolicy::Game | EventLoopPolicy::Continuous => ControlFlow::Poll,
        EventLoopPolicy::DesktopApp | EventLoopPolicy::Mobile | EventLoopPolicy::Headless => {
            ControlFlow::Wait
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_and_continuous_policies_poll_winit_event_loop() {
        assert_eq!(winit_control_flow(EventLoopPolicy::Game), ControlFlow::Poll);
        assert_eq!(
            winit_control_flow(EventLoopPolicy::Continuous),
            ControlFlow::Poll
        );
    }

    #[test]
    fn desktop_mobile_and_headless_policies_wait_for_events() {
        assert_eq!(
            winit_control_flow(EventLoopPolicy::DesktopApp),
            ControlFlow::Wait
        );
        assert_eq!(
            winit_control_flow(EventLoopPolicy::Mobile),
            ControlFlow::Wait
        );
        assert_eq!(
            winit_control_flow(EventLoopPolicy::Headless),
            ControlFlow::Wait
        );
    }
}
