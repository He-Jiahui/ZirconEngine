use super::editor_ui_host::EditorUiHost;
use zircon_runtime::core::CoreHandle;

pub struct EditorManager {
    pub(super) host: EditorUiHost,
}

impl EditorManager {
    pub fn new(core: CoreHandle) -> Self {
        let host = EditorUiHost::bootstrap(core).expect("bootstrap editor ui host");
        Self { host }
    }
}
