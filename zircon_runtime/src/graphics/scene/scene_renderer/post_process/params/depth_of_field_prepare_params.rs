use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    ProjectionMode, RenderDepthOfFieldSettings, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;

/// 景深准备阶段使用的相机深度与镜头快照，产生供后续合成读取的 CoC 和散景种子。
/// 布局与 `depth_of_field_prepare.wgsl` 对应，视口原点描述输入场景纹理中的区域。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::post_process) struct DepthOfFieldPrepareParams {
    pub(in crate::graphics::scene::scene_renderer::post_process) viewport: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) depth: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) lens: [f32; 4],
    pub(in crate::graphics::scene::scene_renderer::post_process) coc_output: [f32; 4],
}

impl DepthOfFieldPrepareParams {
    /// 在录制景深准备 pass 前，把同一视口的相机和镜头设置转换为 GPU 参数。
    /// `scene_origin` 必须对应输入颜色与深度纹理；是否启用该 pass 由调用者另行决定。
    pub(in crate::graphics::scene::scene_renderer::post_process) fn from_camera(
        viewport_size: UVec2,
        scene_origin: [u32; 2],
        camera: &ViewportCameraSnapshot,
        settings: RenderDepthOfFieldSettings,
    ) -> Self {
        let near = camera.z_near.max(0.001);
        let far = camera.z_far.max(near + 0.001);
        let max_radius = settings.max_blur_radius.max(0.0);

        Self {
            viewport: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                scene_origin[0],
                scene_origin[1],
            ],
            depth: [
                near,
                far,
                1.0 / (far - near).max(0.001),
                if matches!(camera.projection_mode, ProjectionMode::Perspective) {
                    1.0
                } else {
                    0.0
                },
            ],
            lens: [
                settings.focus_distance.max(0.0),
                settings.render_focus_range(),
                settings.aperture.max(0.0),
                settings.render_focal_length_mm(),
            ],
            coc_output: [
                max_radius,
                if max_radius > f32::EPSILON {
                    1.0 / max_radius
                } else {
                    0.0
                },
                0.0,
                1.0,
            ],
        }
    }
}

/// 景深缺少有效光圈或模糊半径时，执行入口改为清除准备输出，避免复用旧帧数据。
pub(in crate::graphics::scene::scene_renderer::post_process) fn depth_of_field_prepare_enabled(
    settings: RenderDepthOfFieldSettings,
) -> bool {
    settings.aperture > f32::EPSILON && settings.max_blur_radius > f32::EPSILON
}

#[cfg(test)]
#[path = "tests/depth_of_field_prepare_params.rs"]
mod tests;
