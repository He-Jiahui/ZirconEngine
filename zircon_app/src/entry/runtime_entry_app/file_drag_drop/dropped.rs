use std::path::PathBuf;

use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::byte_slice, RuntimeEntryApp};

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_files_dropped(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        paths: Vec<PathBuf>,
    ) {
        for path in paths {
            let path_text = path.to_string_lossy().to_string();
            let event = ZrRuntimeEventV1::file_dropped(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                self.viewport,
                byte_slice(path_text.as_str()),
            );
            if self.session.handle_event(event).is_err() {
                event_loop.exit();
                return;
            }
        }
    }
}
