use super::super::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    WireframePass,
};

pub(crate) struct ViewportOverlayRenderer {
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) preview_sky:
        PreviewSkyPass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) base_scene:
        BaseScenePass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) selection_outline:
        SelectionOutlinePass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) wireframe:
        WireframePass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) grid:
        GridPass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) scene_gizmo:
        SceneGizmoPass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) handle:
        HandlePass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) line_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) sky_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) grid_vertex_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) grid_vertex_count:
        u32,
}
