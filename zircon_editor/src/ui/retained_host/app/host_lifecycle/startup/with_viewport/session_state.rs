use std::error::Error;
use zircon_runtime::core::runtime::ServiceHandle;

use super::super::super::super::*;
#[cfg(test)]
use super::super::session::resolve_startup_state;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
#[cfg(test)]
use crate::ui::host::resolve_editor_startup_session;
use crate::ui::host::EditorHostStartupSession;

pub(super) type StartupSessionState = EditorHostStartupSession;

pub(super) fn resolve_startup_session_state(
    editor_manager: ServiceHandle<EditorManager>,
    startup_request: Option<EditorGuiStartupRequest>,
    viewport_size: UVec2,
) -> Result<StartupSessionState, Box<dyn Error + Send + Sync>> {
    #[cfg(not(test))]
    {
        return EditorHostStartupSession::open(editor_manager, startup_request, viewport_size);
    }

    #[cfg(test)]
    {
        let (startup_session, state) = {
            let manager = editor_manager.enter()?;
            let mut startup_session = resolve_editor_startup_session(&manager, startup_request)?;
            let state = resolve_startup_state(&manager, &mut startup_session, viewport_size)?;
            (startup_session, state)
        };
        EditorHostStartupSession::from_parts(startup_session, state, editor_manager)
    }
}
