use std::fmt::Display;

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZrRuntimeViewportHandle};

use super::{failure::RuntimeEntryAppFailure, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(super) fn dispatch_runtime_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: ZrRuntimeEventV1,
    ) -> bool {
        let event_kind = event.kind;
        match self.session.handle_event(event) {
            Ok(()) => true,
            Err(error) => {
                let failure = runtime_event_dispatch_failure(event_kind, self.viewport, error);
                zircon_runtime::diagnostic_log::write_error(
                    "runtime_event_dispatch",
                    failure.to_string(),
                );
                self.failure_state.record(failure);
                event_loop.exit();
                false
            }
        }
    }
}

fn runtime_event_dispatch_failure(
    event_kind: u32,
    viewport: ZrRuntimeViewportHandle,
    error: impl Display,
) -> RuntimeEntryAppFailure {
    RuntimeEntryAppFailure::new(
        "runtime_event_dispatch",
        format!("event_kind={event_kind} viewport={viewport:?}"),
        format!("runtime event dispatch failed: {error}"),
        "verify the runtime library ABI and event handler, then restart zircon_runtime",
    )
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ZrRuntimeViewportHandle;

    use super::runtime_event_dispatch_failure;

    #[test]
    fn runtime_event_dispatch_failure_is_actionable() {
        let failure = runtime_event_dispatch_failure(
            17,
            ZrRuntimeViewportHandle::new(3),
            "runtime rejected event",
        );

        assert_eq!(
            failure.to_string(),
            "runtime startup diagnostic: component=runtime_event_dispatch requested=event_kind=17 viewport=ZrRuntimeViewportHandle(3) cause=runtime event dispatch failed: runtime rejected event recovery=verify the runtime library ABI and event handler, then restart zircon_runtime"
        );
    }
}
