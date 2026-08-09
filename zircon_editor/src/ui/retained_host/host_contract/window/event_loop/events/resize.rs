use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use std::time::{Duration, Instant};
use winit::dpi::{PhysicalPosition as WinitPhysicalPosition, PhysicalSize as WinitPhysicalSize};
use winit::event_loop::ActiveEventLoop;

use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

const NATIVE_RESIZE_REFLOW_DEBOUNCE: Duration = Duration::from_millis(80);

impl UiHostWindowEventLoop {
    pub(super) fn handle_surface_resized(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        size: WinitPhysicalSize<u32>,
    ) {
        self.host
            .window()
            .set_size(PhysicalSize::new(size.width, size.height));
        self.host.defer_native_resize_reflow();
        self.pending_resize_reflow_deadline = Some(Instant::now() + NATIVE_RESIZE_REFLOW_DEBOUNCE);
        let should_schedule = self.queue_redraw(HostRedrawRequest::full_frame_for_scenario(
            UiPerfScenario::WindowResize,
            false,
        ));
        if should_schedule {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
        }
        if let Some(presenter) = self.presenter.as_mut() {
            if let Err(error) = presenter.resize((size.width, size.height)) {
                self.host.report_fatal_failure(
                    "editor_host_window",
                    format!("presenter size={}x{}", size.width, size.height),
                    format!("presenter resize failed: {error}"),
                    "verify the graphics adapter and window surface, then restart zircon_editor",
                );
                event_loop.exit();
            }
        }
    }

    pub(super) fn handle_window_moved(&mut self, position: WinitPhysicalPosition<i32>) {
        self.host
            .window()
            .set_position(PhysicalPosition::new(position.x, position.y));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn surface_resize_defers_retained_layout_before_scheduling_native_redraw() {
        let source = include_str!("resize.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("resize production source");
        let defer = production
            .find("self.host.defer_native_resize_reflow();")
            .expect("surface resize must defer retained reflow");
        let redraw = production
            .find("self.queue_redraw")
            .expect("surface resize should still schedule an immediate native redraw");

        assert!(defer < redraw);
    }
}
