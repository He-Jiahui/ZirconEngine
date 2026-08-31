use std::ops::Range;
use std::sync::Arc;

use crate::core::math::UVec2;
use crate::graphics::types::ViewportRenderFrame;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::super::super::ScenePostProcessResources;
use super::super::encode_hybrid_gi_probes::encode_hybrid_gi_probes;
use super::super::encode_hybrid_gi_trace_regions::encode_hybrid_gi_trace_regions;
use super::super::encode_reflection_probes::encode_reflection_probes;

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

fn append_payload_bytes(payload: &mut Vec<u8>, bytes: &[u8]) -> Range<usize> {
    let start = payload.len();
    payload.extend_from_slice(bytes);
    start..payload.len()
}

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
mod tests {
    #[test]
    fn scene_data_uploads_share_one_exact_payload_and_skip_empty_targets() {
        let source = include_str!("prepare_scene_data_uploads.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("post-process scene-data upload source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(production.contains("Vec::with_capacity(payload_byte_len)"));
        assert_eq!(production.matches("let payload: Arc<[u8]>").count(), 1);
        assert_eq!(production.matches("push_non_empty_upload(").count(), 4);
        assert!(production.contains("if payload_byte_len == 0"));
        assert!(production.contains("if source_range.is_empty()"));
    }
}
