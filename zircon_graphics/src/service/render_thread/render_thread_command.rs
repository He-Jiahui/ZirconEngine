use crate::types::EditorOrRuntimeFrame;

pub(in crate::service) enum RenderThreadCommand {
    Frame(EditorOrRuntimeFrame),
    Shutdown,
}
