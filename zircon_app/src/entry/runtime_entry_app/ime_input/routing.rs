use winit::event::Ime;
use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZrRuntimeViewportHandle};

use super::super::RuntimeEntryApp;
use super::admission::RuntimeImeInputAdmission;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_ime_input(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        ime: Ime,
    ) {
        if let Some(event) =
            admitted_runtime_ime_event(&mut self.ime_input_admission, self.viewport, &ime)
        {
            self.dispatch_runtime_event(event_loop, event);
        }
    }
}

pub(super) fn admitted_runtime_ime_event(
    admission: &mut RuntimeImeInputAdmission,
    viewport: ZrRuntimeViewportHandle,
    ime: &Ime,
) -> Option<ZrRuntimeEventV1> {
    // Text events retain raw ABI slices into `ime`; callers dispatch the result synchronously.
    match ime {
        Ime::Enabled if admission.enable() => Some(super::lifecycle::ime_enabled_event(viewport)),
        Ime::Disabled if admission.disable() => {
            Some(super::lifecycle::ime_disabled_event(viewport))
        }
        Ime::Preedit(value, cursor) if admission.admits_composition() => Some(
            super::composition::ime_preedit_event(viewport, value.as_str(), *cursor),
        ),
        Ime::Commit(value) if admission.admits_composition() => Some(
            super::composition::ime_commit_event(viewport, value.as_str()),
        ),
        Ime::DeleteSurrounding {
            before_bytes,
            after_bytes,
        } if admission.admits_composition() => Some(super::deletion::ime_delete_surrounding_event(
            viewport,
            *before_bytes,
            *after_bytes,
        )),
        _ => None,
    }
}
