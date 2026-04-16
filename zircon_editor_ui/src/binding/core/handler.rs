use super::EditorUiBinding;

pub(crate) type Handler<T> = Box<dyn Fn(&EditorUiBinding) -> T + Send + Sync + 'static>;
