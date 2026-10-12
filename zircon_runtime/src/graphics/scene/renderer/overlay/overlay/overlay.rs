use super::super::{
    BaseScenePass, GridPass, HandlePass, PreviewSkyPass, SceneGizmoPass, SelectionOutlinePass,
    WireframePass,
};
use super::depth_reconstruction::OverlayDepthReconstruction;

pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) struct ViewportInteractionOverlays
{
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
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) depth_reconstruction:
        OverlayDepthReconstruction,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) grid_vertex_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) grid_vertex_count:
        u32,
}

/// 场景天空与交互辅助层的资源所有者；图执行器调用各阶段，环境捕获只消费场景内容。
/// 交互资源可整体缺席，避免环境捕获构造或准备编辑器专用几何。
pub(crate) struct ViewportOverlayRenderer {
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) preview_sky:
        PreviewSkyPass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) base_scene:
        BaseScenePass,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) interaction_overlays:
        Option<ViewportInteractionOverlays>,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) sky_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) sky_volumetric_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) sky_volumetric_apply:
        crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
}
