use super::super::super::super::post_process_params::PostProcessParams;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

/// 为调用者的持久参数槽生成提交前上传；上传与相应节点命令须进入同一提交事务。
/// 不同参数生产者应选不同槽，同一槽在提交前的多份上传会覆盖早先值。
pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn post_process_params_upload(
    buffer: &wgpu::Buffer,
    params: &PostProcessParams,
) -> WgpuBufferUploadBatch {
    WgpuBufferUpload::from_bytes(buffer.clone(), 0, bytemuck::bytes_of(params)).into()
}

#[cfg(test)]
#[path = "tests/pass_params_buffer.rs"]
mod tests;
