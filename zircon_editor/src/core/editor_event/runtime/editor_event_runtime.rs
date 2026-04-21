use std::sync::Mutex;

pub struct EditorEventRuntime {
    pub(crate) inner: Mutex<super::editor_event_runtime_inner::EditorEventRuntimeInner>,
}
