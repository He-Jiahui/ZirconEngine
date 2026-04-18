use super::{
    move_handle_tool::MoveHandleTool, rotate_handle_tool::RotateHandleTool,
    scale_handle_tool::ScaleHandleTool,
};

#[derive(Clone, Debug)]
pub(crate) struct HandleToolRegistry {
    pub(in crate::scene::viewport::handles) move_tool: MoveHandleTool,
    pub(in crate::scene::viewport::handles) rotate_tool: RotateHandleTool,
    pub(in crate::scene::viewport::handles) scale_tool: ScaleHandleTool,
}
