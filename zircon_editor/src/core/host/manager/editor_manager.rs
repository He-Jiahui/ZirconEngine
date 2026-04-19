use std::collections::BTreeMap;
use std::sync::Mutex;

use zircon_runtime::core::CoreHandle;

use crate::layout::LayoutManager;
use crate::view::ViewRegistry;
use crate::ViewInstanceId;

use super::editor_session_state::EditorSessionState;
use super::ui_asset_sessions::UiAssetWorkspaceEntry;
use super::window_host_manager::WindowHostManager;

pub struct EditorManager {
    pub(super) core: CoreHandle,
    pub(super) view_registry: Mutex<ViewRegistry>,
    pub(super) layout_manager: LayoutManager,
    pub(super) window_host_manager: Mutex<WindowHostManager>,
    pub(super) session: Mutex<EditorSessionState>,
    pub(super) ui_asset_sessions: Mutex<BTreeMap<ViewInstanceId, UiAssetWorkspaceEntry>>,
}

impl EditorManager {
    pub fn new(core: CoreHandle) -> Self {
        let manager = Self {
            core,
            view_registry: Mutex::new(ViewRegistry::default()),
            layout_manager: LayoutManager,
            window_host_manager: Mutex::new(WindowHostManager::default()),
            session: Mutex::new(EditorSessionState::default()),
            ui_asset_sessions: Mutex::new(BTreeMap::new()),
        };
        manager
            .register_builtin_views()
            .expect("builtin editor views");
        manager
            .bootstrap_default_layout()
            .expect("default workbench");
        manager
    }
}
