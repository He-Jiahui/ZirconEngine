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
        stable_instance_key: 0,
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

fn vertex(position: Vec3, normal: Vec3, tangent: Vec4) -> RenderVirtualGeometryPagePayloadVertex {
    RenderVirtualGeometryPagePayloadVertex {
        position,
        normal,
        tangent,
    }
}
