use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_ime_input(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        ime: Ime,
    ) {
        match ime {
            Ime::Enabled => super::lifecycle::forward_ime_enabled(self, event_loop),
            Ime::Disabled => super::lifecycle::forward_ime_disabled(self, event_loop),
            Ime::Preedit(value, cursor) => {
                super::composition::forward_ime_preedit(self, event_loop, value.as_str(), cursor)
            }
            Ime::Commit(value) => {
                super::composition::forward_ime_commit(self, event_loop, value.as_str())
            }
            Ime::DeleteSurrounding {
                before_bytes,
                after_bytes,
            } => super::deletion::forward_ime_delete_surrounding(
                self,
                event_loop,
                before_bytes,
                after_bytes,
            ),
        }
    }
}
