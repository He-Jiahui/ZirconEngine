use crate::scene::viewport::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SceneViewportToolbarState {
    pub(crate) tool: SceneViewportTool,
    pub(crate) transform_space: TransformSpace,
    pub(crate) projection_mode: ProjectionMode,
    pub(crate) view_orientation: ViewOrientation,
    pub(crate) display_mode: DisplayMode,
    pub(crate) grid_mode: GridMode,
    pub(crate) preview_lighting: bool,
    pub(crate) preview_skybox: bool,
    pub(crate) gizmos_enabled: bool,
    pub(crate) has_selection: bool,
    pub(crate) can_frame_selection: bool,
    pub(crate) handle_drag_active: bool,
}
