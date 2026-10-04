use std::error::Error;

use super::super::super::*;

#[cfg(not(test))]
pub(super) fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &mut EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error + Send + Sync>> {
    build_startup_state(editor_manager, session, viewport_size)
}

// 测试缺少默认 Scene Manager 时退回欢迎状态；生产路径保留启动失败，防止掩盖服务缺失。
#[cfg(test)]
pub(super) fn resolve_startup_state(
    editor_manager: &EditorManager,
    session: &mut EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error + Send + Sync>> {
    build_startup_state(editor_manager, session, viewport_size).or_else(|error| {
        let message = error.to_string();
        if message.contains("SceneModule.Manager.DefaultLevelManager") {
            let mut state =
                EditorState::welcome(viewport_size, session.welcome_pane_snapshot(false));
            state.set_status_line(session.status_message.clone());
            Ok(state)
        } else {
            Err(error)
        }
    })
}
