use super::super::params::post_process_params::PostProcessParams;

/// 按后处理参数生产者持有独立的持久 uniform 槽，供同一提交中的拆分 pass 使用。
/// 独立槽避免不同效果的提交前上传互相覆盖；字段与各执行入口选择的缓冲须一起维护。
pub(in crate::graphics::scene::scene_renderer::post_process) struct PostProcessPassParameterBuffers
{
    pub(in crate::graphics::scene::scene_renderer::post_process) blur: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_blur: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) scene_composite: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_pyramid: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_pyramid_coarse:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_resolve: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) specular_occlusion: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process: wgpu::Buffer,
}

impl PostProcessPassParameterBuffers {
    pub(in crate::graphics::scene::scene_renderer::post_process) fn new(
        device: &wgpu::Device,
    ) -> Self {
        Self {
            blur: create_parameter_buffer(device, "zircon-blur-params"),
            depth_of_field: create_parameter_buffer(device, "zircon-depth-of-field-params"),
            motion_blur: create_parameter_buffer(device, "zircon-motion-blur-params"),
            scene_composite: create_parameter_buffer(device, "zircon-scene-composite-params"),
            reflection_pyramid: create_parameter_buffer(
                device,
                "zircon-screen-space-reflection-reflection-pyramid-params",
            ),
            reflection_pyramid_coarse: create_parameter_buffer(
                device,
                "zircon-screen-space-reflection-reflection-pyramid-coarse-params",
            ),
            reflection_resolve: create_parameter_buffer(
                device,
                "zircon-screen-space-reflection-resolve-params",
            ),
            specular_occlusion: create_parameter_buffer(
                device,
                "zircon-screen-space-reflection-specular-occlusion-params",
            ),
            post_process: create_parameter_buffer(device, "zircon-post-process-pass-params"),
        }
    }
}

fn create_parameter_buffer(device: &wgpu::Device, label: &'static str) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: std::mem::size_of::<PostProcessParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
#[path = "tests/post_process_pass_parameter_buffers.rs"]
mod tests;
