use std::ops::Range;
use std::sync::Arc;

use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderFrame;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::super::ScenePostProcessResources;
use super::super::encode_hybrid_gi_probes::encode_hybrid_gi_probes;
use super::super::encode_hybrid_gi_trace_regions::encode_hybrid_gi_trace_regions;
use super::super::encode_reflection_probes::encode_reflection_probes;

/// 将有效 GPU 数据前缀放入一份共享不可变 payload，并返回与每个缓冲读取范围一致的计数。
/// 零条目不上传，shader 必须以计数限制读取，不能依赖未更新的尾部内容。
pub(in crate::graphics::scene::scene_renderer::post_process::resources) fn prepare_scene_data_uploads(
    resources: &ScenePostProcessResources,
    frame: &ViewportRenderFrame,
    viewport_size: UVec2,
    reflection_probes_enabled: bool,
    hybrid_global_illumination_enabled: bool,
) -> (u32, u32, u32, WgpuBufferUploadBatch) {
    let (reflection_probes, reflection_probe_count) =
        encode_reflection_probes(&frame.extract, viewport_size, reflection_probes_enabled);
    let (hybrid_gi_probes, hybrid_gi_probe_count) =
        encode_hybrid_gi_probes(frame, viewport_size, hybrid_global_illumination_enabled);
    let (hybrid_gi_trace_regions, hybrid_gi_trace_region_count) =
        encode_hybrid_gi_trace_regions(frame, viewport_size, hybrid_global_illumination_enabled);

    let reflection_probe_bytes =
        bytemuck::cast_slice(&reflection_probes[..reflection_probe_count as usize]);
    let hybrid_gi_probe_bytes =
        bytemuck::cast_slice(&hybrid_gi_probes[..hybrid_gi_probe_count as usize]);
    let hybrid_gi_trace_region_bytes =
        bytemuck::cast_slice(&hybrid_gi_trace_regions[..hybrid_gi_trace_region_count as usize]);
    let payload_byte_len = reflection_probe_bytes
        .len()
        .saturating_add(hybrid_gi_probe_bytes.len())
        .saturating_add(hybrid_gi_trace_region_bytes.len());
    if payload_byte_len == 0 {
        return (
            reflection_probe_count,
            hybrid_gi_probe_count,
            hybrid_gi_trace_region_count,
            WgpuBufferUploadBatch::new(),
        );
    }

    let mut payload = Vec::with_capacity(payload_byte_len);
    let reflection_probe_range = append_payload_bytes(&mut payload, reflection_probe_bytes);
    let hybrid_gi_probe_range = append_payload_bytes(&mut payload, hybrid_gi_probe_bytes);
    let hybrid_gi_trace_region_range =
        append_payload_bytes(&mut payload, hybrid_gi_trace_region_bytes);
    let payload: Arc<[u8]> = payload.into();
    let mut uploads = WgpuBufferUploadBatch::new();
    push_non_empty_upload(
        &mut uploads,
        &resources.reflection_probe_buffer,
        Arc::clone(&payload),
        reflection_probe_range,
    );
    push_non_empty_upload(
        &mut uploads,
        &resources.hybrid_gi_probe_buffer,
        Arc::clone(&payload),
        hybrid_gi_probe_range,
    );
    push_non_empty_upload(
        &mut uploads,
        &resources.hybrid_gi_trace_region_buffer,
        payload,
        hybrid_gi_trace_region_range,
    );

    (
        reflection_probe_count,
        hybrid_gi_probe_count,
        hybrid_gi_trace_region_count,
        uploads,
    )
}

// 保存各目的缓冲在共享 payload 中的精确来源范围，后续上传只复制所属片段。
fn append_payload_bytes(payload: &mut Vec<u8>, bytes: &[u8]) -> Range<usize> {
    let start = payload.len();
    payload.extend_from_slice(bytes);
    start..payload.len()
}

// 空前缀无需清旧缓冲，消费者计数阻止读取；有效范围持有 Arc，直到提交上传事务消费。
fn push_non_empty_upload(
    uploads: &mut WgpuBufferUploadBatch,
    buffer: &wgpu::Buffer,
    payload: Arc<[u8]>,
    source_range: Range<usize>,
) {
    if source_range.is_empty() {
        return;
    }
    uploads.push(
        WgpuBufferUpload::new(buffer.clone(), 0, payload, source_range)
            .expect("prepared post-process upload range must fit its immutable payload"),
    );
}

#[cfg(test)]
#[path = "tests/prepare_scene_data_uploads.rs"]
mod tests;
