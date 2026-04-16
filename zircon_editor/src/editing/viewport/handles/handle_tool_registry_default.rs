use super::{
    handle_tool_registry::HandleToolRegistry, move_handle_tool::MoveHandleTool,
    rotate_handle_tool::RotateHandleTool, scale_handle_tool::ScaleHandleTool,
};

impl Default for HandleToolRegistry {
    fn default() -> Self {
        Self {
            move_tool: MoveHandleTool,
            rotate_tool: RotateHandleTool,
            scale_tool: ScaleHandleTool,
        }
    }
}
