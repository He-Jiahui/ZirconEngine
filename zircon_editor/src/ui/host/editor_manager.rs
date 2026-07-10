use super::editor_ui_host::EditorUiHost;
use crate::core::context::{EditorContext, EditorContextBuilder};
use std::sync::Arc;
use zircon_runtime::core::CoreHandle;

pub struct EditorManager {
    pub(super) host: EditorUiHost,
    context: Arc<EditorContext>,
}

impl EditorManager {
    pub fn new(core: CoreHandle) -> Self {
        let host = EditorUiHost::bootstrap(core).expect("bootstrap editor ui host");
        let context = EditorContextBuilder::new().build();
        Self { host, context }
    }

    pub fn context(&self) -> &Arc<EditorContext> {
        &self.context
    }
}
