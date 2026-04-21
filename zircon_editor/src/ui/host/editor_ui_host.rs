use std::collections::BTreeMap;
use std::sync::Mutex;

use zircon_runtime::core::CoreHandle;

use crate::ui::workbench::layout::LayoutManager;
use crate::ui::workbench::view::{ViewInstanceId, ViewRegistry};

use super::animation_editor_sessions::AnimationEditorWorkspaceEntry;
use super::asset_editor_sessions::UiAssetWorkspaceEntry;
use super::editor_error::EditorError;
use super::editor_session_state::EditorSessionState;
use super::window_host_manager::WindowHostManager;

pub(super) struct EditorUiHost {
    pub(super) core: CoreHandle,
    pub(super) view_registry: Mutex<ViewRegistry>,
    pub(super) layout_manager: LayoutManager,
    pub(super) window_host_manager: Mutex<WindowHostManager>,
    pub(super) session: Mutex<EditorSessionState>,
    pub(super) animation_editor_sessions:
        Mutex<BTreeMap<ViewInstanceId, AnimationEditorWorkspaceEntry>>,
    pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>,
}

impl EditorUiHost {
    pub(super) fn new(core: CoreHandle) -> Self {
        Self {
            core,
            view_registry: Mutex::new(ViewRegistry::default()),
            layout_manager: LayoutManager,
            window_host_manager: Mutex::new(WindowHostManager::default()),
            session: Mutex::new(EditorSessionState::default()),
            animation_editor_sessions: Mutex::new(BTreeMap::new()),
            ui_asset_sessions: Mutex::new(BTreeMap::new()),
        }
    }

    pub(super) fn bootstrap(core: CoreHandle) -> Result<Self, EditorError> {
        let host = Self::new(core);
        host.register_builtin_views()?;
        host.bootstrap_default_layout()?;
        Ok(host)
    }
}
