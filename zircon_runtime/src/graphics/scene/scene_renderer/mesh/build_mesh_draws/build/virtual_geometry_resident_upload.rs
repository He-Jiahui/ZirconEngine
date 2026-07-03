use std::collections::BTreeMap;

use crate::core::framework::render::{
    RenderVirtualGeometryDebugSnapshot, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryPagePayload, RenderVirtualGeometryPagePayloadVertex,
};
use crate::graphics::scene::gpu_scene::{
    GpuScene, GpuSceneVirtualGeometryUploadReport, GpuVirtualGeometryClusterWord,
    GpuVirtualGeometryPage, GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX,
    GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
};

pub(super) fn upload_virtual_geometry_resident_payloads(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    gpu_scene: &mut GpuScene,
    virtual_geometry_enabled: bool,
    snapshot: Option<&RenderVirtualGeometryDebugSnapshot>,
) -> GpuSceneVirtualGeometryUploadReport {
    let Some(snapshot) = snapshot.filter(|_| virtual_geometry_enabled) else {
        return gpu_scene.upload_virtual_geometry_resident_buffers(device, queue, &[], &[]);
    };

    let (page_rows, cluster_words) = virtual_geometry_payload_rows_from_snapshot(snapshot);
    gpu_scene.upload_virtual_geometry_resident_buffers(device, queue, &page_rows, &cluster_words)
}

fn virtual_geometry_payload_rows_from_snapshot(
    snapshot: &RenderVirtualGeometryDebugSnapshot,
) -> (
    Vec<GpuVirtualGeometryPage>,
    Vec<GpuVirtualGeometryClusterWord>,
) {
    let mut pages = Vec::new();
    let mut cluster_words = Vec::new();
    let mut uploaded_pages = BTreeMap::<u32, GpuVirtualGeometryPage>::new();
    let payload_by_page = snapshot
        .resident_page_payloads
        .iter()
        .map(|payload| (payload.page_id, payload))
        .collect::<BTreeMap<_, _>>();

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
    uploaded_pages: &mut BTreeMap<u32, GpuVirtualGeometryPage>,
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
mod tests {
    use super::*;
    use crate::core::framework::render::RenderVirtualGeometryExecutionSegment;
    use crate::core::math::{Vec3, Vec4};

    #[test]
    fn virtual_geometry_page_rows_follow_submission_slots() {
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            execution_segments: vec![
                segment(30, Some(2), RenderVirtualGeometryExecutionState::Resident),
                segment(
                    40,
                    Some(0),
                    RenderVirtualGeometryExecutionState::PendingUpload,
                ),
                segment(50, None, RenderVirtualGeometryExecutionState::Resident),
                segment(60, Some(1), RenderVirtualGeometryExecutionState::Resident),
            ],
            ..RenderVirtualGeometryDebugSnapshot::default()
        };

        let (rows, cluster_words) = virtual_geometry_payload_rows_from_snapshot(&snapshot);

        assert_eq!(rows.len(), 3);
        assert!(cluster_words.is_empty());
        assert_eq!(rows[0], GpuVirtualGeometryPage::default());
        assert_eq!(
            rows[1],
            GpuVirtualGeometryPage::new(0, 0, 60, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT)
        );
        assert_eq!(
            rows[2],
            GpuVirtualGeometryPage::new(0, 0, 30, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT)
        );
    }

    #[test]
    fn virtual_geometry_cluster_words_follow_resident_page_payloads() {
        let snapshot = RenderVirtualGeometryDebugSnapshot {
            resident_page_payloads: vec![
                RenderVirtualGeometryPagePayload::new(
                    30,
                    vec![
                        vertex(
                            Vec3::new(1.0, 2.0, 3.0),
                            Vec3::new(0.0, 1.0, 0.0),
                            Vec4::new(1.0, 0.0, 0.0, 1.0),
                        ),
                        vertex(
                            Vec3::new(4.0, 5.0, 6.0),
                            Vec3::new(0.0, 0.0, 1.0),
                            Vec4::new(0.0, 1.0, 0.0, -1.0),
                        ),
                    ],
                ),
                RenderVirtualGeometryPagePayload::new(
                    60,
                    vec![vertex(
                        Vec3::new(7.0, 8.0, 9.0),
                        Vec3::new(1.0, 0.0, 0.0),
                        Vec4::new(0.0, 0.0, 1.0, 1.0),
                    )],
                ),
            ],
            execution_segments: vec![
                segment(30, Some(2), RenderVirtualGeometryExecutionState::Resident),
                segment(60, Some(1), RenderVirtualGeometryExecutionState::Resident),
                segment(30, Some(0), RenderVirtualGeometryExecutionState::Resident),
            ],
            ..RenderVirtualGeometryDebugSnapshot::default()
        };

        let (rows, cluster_words) = virtual_geometry_payload_rows_from_snapshot(&snapshot);

        assert_eq!(rows.len(), 3);
        assert_eq!(
            rows[0],
            GpuVirtualGeometryPage::new(0, 2, 30, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT)
        );
        assert_eq!(
            rows[2],
            GpuVirtualGeometryPage::new(0, 2, 30, GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT),
            "repeated resident submissions for the same page should share cluster word payload"
        );
        assert_eq!(
            rows[1],
            GpuVirtualGeometryPage::new(
                2 * GPU_VIRTUAL_GEOMETRY_CLUSTER_WORDS_PER_VERTEX,
                1,
                60,
                GPU_VIRTUAL_GEOMETRY_PAGE_FLAG_RESIDENT,
            )
        );
        assert_eq!(cluster_words.len(), 12);
        assert_eq!(cluster_words[0].values, [1.0, 2.0, 3.0, 1.0]);
        assert_eq!(cluster_words[1].values, [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(cluster_words[2].values, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(cluster_words[4].values, [4.0, 5.0, 6.0, 1.0]);
        assert_eq!(cluster_words[8].values, [7.0, 8.0, 9.0, 1.0]);
    }

    fn segment(
        page_id: u32,
        submission_slot: Option<u32>,
        state: RenderVirtualGeometryExecutionState,
    ) -> RenderVirtualGeometryExecutionSegment {
        RenderVirtualGeometryExecutionSegment {
            original_index: 0,
            instance_index: None,
            entity: 0,
            page_id,
            draw_ref_index: 0,
            submission_index: submission_slot,
            draw_ref_rank: submission_slot,
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            submission_slot,
            state,
            lineage_depth: 0,
            lod_level: 0,
            frontier_rank: 0,
        }
    }

    fn vertex(
        position: Vec3,
        normal: Vec3,
        tangent: Vec4,
    ) -> RenderVirtualGeometryPagePayloadVertex {
        RenderVirtualGeometryPagePayloadVertex {
            position,
            normal,
            tangent,
        }
    }
}
