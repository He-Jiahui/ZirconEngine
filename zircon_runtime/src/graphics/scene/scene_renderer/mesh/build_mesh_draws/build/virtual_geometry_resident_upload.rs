use std::collections::HashMap;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPagePayloadVertex,
};
use crate::graphics::scene::gpu_scene::{
    GpuScene, GpuScenePreparedVirtualGeometryUpload, GpuVirtualGeometryClusterWord,
    GpuVirtualGeometryPage, GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX,
    GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
};

pub(super) fn upload_virtual_geometry_resident_payloads(
    device: &wgpu::Device,
    gpu_scene: &mut GpuScene,
    virtual_geometry_enabled: bool,
    snapshot: Option<&Arc<RenderVirtualGeometryDebugSnapshot>>,
) -> GpuScenePreparedVirtualGeometryUpload {
    let Some(snapshot) = snapshot.filter(|_| virtual_geometry_enabled) else {
        return gpu_scene.prepare_virtual_geometry_resident_buffers(device, Vec::new(), Vec::new());
    };

    let (page_rows, cluster_words) = virtual_geometry_payload_rows_from_snapshot(snapshot);
    gpu_scene.prepare_virtual_geometry_resident_buffers(device, page_rows, cluster_words)
}

fn virtual_geometry_payload_rows_from_snapshot(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
) -> (
    Vec<GpuVirtualGeometryPage>,
    Vec<GpuVirtualGeometryClusterWord>,
) {
    let mut pages = Vec::with_capacity(snapshot.execution_segments.len());
    let mut cluster_words = Vec::new();
    // Segment order owns output ordering; page lookup only needs stable expected-O(1) dedup.
    let mut uploaded_pages =
        HashMap::<u32, GpuVirtualGeometryPage>::with_capacity(snapshot.execution_segments.len());
    let payload_by_page = snapshot
        .resident_page_payloads
        .iter()
        .map(|payload| (payload.page_id, payload))
        .collect::<HashMap<_, _>>();

    for segment in &snapshot.execution_segments {
        if segment.state != RenderVirtualGeometryExecutionState::Resident {
            continue;
        }
        let Some(slot) = segment.submission_slot else {
            continue;
        };
        let slot_index = slot as usize;
        if pages.len() <= slot_index {
            pages.resize(slot_index + 1, GpuVirtualGeometryPage::default());
        }
        let page_row = resident_gpu_page_row(
            segment.page_id,
            payload_by_page.get(&segment.page_id).copied(),
            &mut uploaded_pages,
            &mut cluster_words,
        );
        pages[slot_index] = page_row;
    }

    (pages, cluster_words)
}

fn resident_gpu_page_row(
    page_id: u32,
    payload: Option<&RenderVirtualGeometryPagePayload>,
    uploaded_pages: &mut HashMap<u32, GpuVirtualGeometryPage>,
    cluster_words: &mut Vec<GpuVirtualGeometryClusterWord>,
) -> GpuVirtualGeometryPage {
    if let Some(page) = uploaded_pages.get(&page_id).copied() {
        return page;
    }

    let page = payload
        .map(|payload| {
            let cluster_base_word = saturated_u32_len(cluster_words.len());
            append_page_payload_cluster_words(payload, cluster_words);
            GpuVirtualGeometryPage::new(
                cluster_base_word,
                saturated_u32_len(payload.vertices.len()),
                page_id,
                GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
            )
        })
        // 保留 resident page 槽位但以零顶点数标记缺失 payload，让 shader 回退到普通 mesh 输入。
        .unwrap_or_else(|| {
            GpuVirtualGeometryPage::new(0, 0, page_id, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT)
        });
    uploaded_pages.insert(page_id, page);
    page
}

fn append_page_payload_cluster_words(
    payload: &RenderVirtualGeometryPagePayload,
    cluster_words: &mut Vec<GpuVirtualGeometryClusterWord>,
) {
    for vertex in &payload.vertices {
        append_vertex_payload_cluster_words(*vertex, cluster_words);
    }
}

fn append_vertex_payload_cluster_words(
    vertex: RenderVirtualGeometryPagePayloadVertex,
    cluster_words: &mut Vec<GpuVirtualGeometryClusterWord>,
) {
    cluster_words.extend_from_slice(&[
        GpuVirtualGeometryClusterWord {
            values: [vertex.position.x, vertex.position.y, vertex.position.z, 1.0],
        },
        GpuVirtualGeometryClusterWord {
            values: [vertex.normal.x, vertex.normal.y, vertex.normal.z, 0.0],
        },
        GpuVirtualGeometryClusterWord {
            values: vertex.tangent.to_array(),
        },
        GpuVirtualGeometryClusterWord { values: [0.0; 4] },
    ]);
}

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

#[cfg(test)]
#[path = "tests/virtual_geometry_resident_upload.rs"]
mod tests;
