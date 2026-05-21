use winit::dpi::PhysicalPosition;
use winit::event_loop::ActiveEventLoop;
use winit::window::Theme;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::window_theme, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_destroyed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        let event =
            ZrRuntimeEventV1::window_destroyed(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_window_moved(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        position: PhysicalPosition<i32>,
    ) {
        let event = ZrRuntimeEventV1::window_moved(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            position.x,
            position.y,
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_window_occluded(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        occluded: bool,
    ) {
        let event = ZrRuntimeEventV1::window_occluded(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            occluded,
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }

    pub(in crate::entry::runtime_entry_app) fn handle_window_theme_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        theme: Theme,
    ) {
        let event = ZrRuntimeEventV1::window_theme_changed(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.viewport,
            window_theme(theme),
        );
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
