use crate::ui::retained_host::primitives::{PhysicalPosition, PhysicalSize};
use winit::dpi::{PhysicalPosition as WinitPhysicalPosition, PhysicalSize as WinitPhysicalSize};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_error;

use super::super::UiHostWindowEventLoop;
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::UiPerfScenario;

impl UiHostWindowEventLoop {
    pub(super) fn handle_surface_resized(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        size: WinitPhysicalSize<u32>,
    ) {
        self.host
            .window()
            .set_size(PhysicalSize::new(size.width, size.height));
        self.queue_redraw(HostRedrawRequest::full_frame_for_scenario(
            UiPerfScenario::Startup,
            true,
        ));
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
        if let Some(presenter) = self.presenter.as_mut() {
            if let Err(error) = presenter.resize((size.width, size.height)) {
                write_error(
                    "editor_host_window",
                    format!(
                        "presenter resize failed size={}x{}: {error}",
                        size.width, size.height
                    ),
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
