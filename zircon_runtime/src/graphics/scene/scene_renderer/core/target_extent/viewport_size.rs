use crate::graphics::types::EditorOrRuntimeFrame;

pub(crate) fn viewport_size(frame: &EditorOrRuntimeFrame) -> crate::core::math::UVec2 {
    crate::core::math::UVec2::new(frame.viewport_size.x.max(1), frame.viewport_size.y.max(1))
}
