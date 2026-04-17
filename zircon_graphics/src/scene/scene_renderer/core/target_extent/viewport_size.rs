use crate::types::EditorOrRuntimeFrame;

pub(crate) fn viewport_size(frame: &EditorOrRuntimeFrame) -> zircon_math::UVec2 {
    zircon_math::UVec2::new(frame.viewport_size.x.max(1), frame.viewport_size.y.max(1))
}
