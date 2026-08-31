use std::error::Error;
use std::sync::Arc;

use super::super::super::super::*;
#[cfg(test)]
use super::super::session::resolve_startup_state;
use crate::core::gui_startup_request::EditorGuiStartupRequest;
#[cfg(test)]
use crate::ui::host::resolve_editor_startup_session;
use crate::ui::host::EditorHostStartupSession;

pub(super) type StartupSessionState = EditorHostStartupSession;

pub(super) fn resolve_startup_session_state(
    editor_manager: Arc<EditorManager>,
    startup_request: Option<EditorGuiStartupRequest>,
    viewport_size: UVec2,
) -> Result<StartupSessionState, Box<dyn Error>> {
    #[cfg(not(test))]
    {
        return EditorHostStartupSession::open(editor_manager, startup_request, viewport_size);
    }

    #[cfg(test)]
    {
        let mut startup_session =
            resolve_editor_startup_session(editor_manager.as_ref(), startup_request)?;
        let state =
            resolve_startup_state(editor_manager.as_ref(), &mut startup_session, viewport_size)?;
        EditorHostStartupSession::from_parts(startup_session, state, editor_manager)
    }
}
