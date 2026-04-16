use super::transform_handle_drag_session::TransformHandleDragSession;

#[derive(Clone, Debug)]
pub(crate) enum HandleDragSession {
    Move(TransformHandleDragSession),
    Rotate(TransformHandleDragSession),
    Scale(TransformHandleDragSession),
}
