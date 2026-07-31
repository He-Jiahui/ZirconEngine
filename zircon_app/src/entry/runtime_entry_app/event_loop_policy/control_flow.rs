use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn apply_event_loop_policy(
        &self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let control_flow = self.frame_cadence.control_flow();
        event_loop.set_control_flow(control_flow);
    }
}
