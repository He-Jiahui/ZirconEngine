use std::error::Error;

use super::super::super::super::*;
use super::super::session::{resolve_editor_startup_session, resolve_startup_state};
use crate::core::gui_startup_request::EditorGuiStartupRequest;

pub(super) struct StartupSessionState {
    pub(super) startup_session: EditorStartupSessionDocument,
    pub(super) state: EditorState,
}

pub(super) fn resolve_startup_session_state(
    editor_manager: &EditorManager,
    startup_request: Option<EditorGuiStartupRequest>,
    viewport_size: UVec2,
) -> Result<StartupSessionState, Box<dyn Error>> {
    let startup_session = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_startup_session");
        resolve_editor_startup_session(editor_manager, startup_request)?
    };
    let state = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_startup_state");
        resolve_startup_state(editor_manager, &startup_session, viewport_size)?
    };

    Ok(StartupSessionState {
        startup_session,
        state,
    })
}
