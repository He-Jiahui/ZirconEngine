use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiFrame, UiPoint},
    surface::{
        UiHitTestEntry, UiHitTestGrid, UiSurfaceFrame, UiSurfaceFrameDomainGenerations,
        UiVirtualPointerPosition,
    },
};

use super::{
    RuntimeUiInputPublication, RuntimeUiInputQueryAdmission, RuntimeUiInputQueryRejectReason,
};
use crate::core::math::UVec2;

#[test]
fn query_returns_only_cell_candidates_in_topmost_surface_order() {
    let viewport = UVec2::new(640, 360);
    let mut publication = RuntimeUiInputPublication::default();

    let report = publication.publish(
        viewport,
        3,
        [
            frame(1, &[UiFrame::new(0.0, 0.0, 40.0, 40.0)]),
            frame(1, &[UiFrame::new(300.0, 100.0, 40.0, 40.0)]),
            frame(1, &[UiFrame::new(0.0, 0.0, 40.0, 40.0)]),
        ],
    );

    assert!(report.full_rebuild);
    assert_eq!(report.patched_surface_count, 3);
    let overlap = publication
        .query(viewport, UiPoint::new(12.0, 12.0), UiPoint::new(12.0, 12.0))
        .published()
        .unwrap();
    assert_eq!(overlap.candidate_count(), 2);
    assert_eq!(publication.candidate_surface(overlap, 0), Some(2));
    assert_eq!(publication.candidate_surface(overlap, 1), Some(0));
    assert_eq!(publication.candidate_surface(overlap, 2), None);

    let empty = publication
        .query(
            viewport,
            UiPoint::new(180.0, 180.0),
            UiPoint::new(180.0, 180.0),
        )
        .published()
        .unwrap();
    assert_eq!(empty.candidate_count(), 0);
    assert_eq!(publication.candidate_surface(empty, 0), None);
}

#[test]
fn hit_generation_patch_moves_only_the_changed_surface_footprint() {
    let viewport = UVec2::new(640, 360);
    let mut publication = RuntimeUiInputPublication::default();
    let stable = frame(1, &[UiFrame::new(300.0, 100.0, 40.0, 40.0)]);
    publication.publish(
        viewport,
        2,
        [
            frame(1, &[UiFrame::new(0.0, 0.0, 40.0, 40.0)]),
            Arc::clone(&stable),
        ],
    );

    let report = publication.publish(
        viewport,
        2,
        [frame(2, &[UiFrame::new(500.0, 280.0, 40.0, 40.0)]), stable],
    );

    assert!(!report.full_rebuild);
    assert_eq!(report.patched_surface_count, 1);
    assert_eq!(report.visited_entry_count, 1);
    assert_eq!(
        publication
            .query(viewport, UiPoint::new(12.0, 12.0), UiPoint::new(12.0, 12.0),)
            .published()
            .unwrap()
            .candidate_count(),
        0
    );
    let moved = publication
        .query(
            viewport,
            UiPoint::new(512.0, 292.0),
            UiPoint::new(512.0, 292.0),
        )
        .published()
        .unwrap();
    assert_eq!(publication.candidate_surface(moved, 0), Some(0));

    let stable_report = publication.publish(
        viewport,
        2,
        [
            frame(2, &[UiFrame::new(500.0, 280.0, 40.0, 40.0)]),
            frame(1, &[UiFrame::new(300.0, 100.0, 40.0, 40.0)]),
        ],
    );
    assert_eq!(stable_report.patched_surface_count, 0);
    assert_eq!(stable_report.visited_entry_count, 0);
}

#[test]
fn hit_generation_patch_reuses_cell_stamps_and_surface_footprint_allocation() {
    let viewport = UVec2::new(640, 360);
    let mut publication = RuntimeUiInputPublication::default();
    publication.publish(
        viewport,
        1,
        [frame(1, &[UiFrame::new(0.0, 0.0, 320.0, 180.0)])],
    );
    let first_capacities = publication.patch_scratch_capacities_for_test(0);

    publication.publish(
        viewport,
        1,
        [frame(2, &[UiFrame::new(320.0, 180.0, 320.0, 180.0)])],
    );

    assert_eq!(
        publication.patch_scratch_capacities_for_test(0),
        first_capacities
    );
}

#[test]
fn resize_query_maps_hit_coordinates_but_preserves_the_physical_pointer() {
    let published_viewport = UVec2::new(640, 360);
    let mut publication = RuntimeUiInputPublication::default();
    publication.publish(
        published_viewport,
        1,
        [frame(1, &[UiFrame::new(0.0, 0.0, 40.0, 40.0)])],
    );

    let query = publication
        .query(
            UVec2::new(1280, 720),
            UiPoint::new(24.0, 24.0),
            UiPoint::new(20.0, 20.0),
        )
        .published()
        .unwrap();

    assert_eq!(publication.candidate_surface(query, 0), Some(0));
    let hit_test_query = query.hit_test_query();
    assert_eq!(hit_test_query.point, UiPoint::new(24.0, 24.0));
    assert_eq!(
        hit_test_query.virtual_pointer,
        Some(UiVirtualPointerPosition::new(
            UiPoint::new(12.0, 12.0),
            UiPoint::new(10.0, 10.0),
        ))
    );
}

#[test]
fn query_distinguishes_unpublished_from_invalid_input() {
    let viewport = UVec2::new(640, 360);
    let mut publication = RuntimeUiInputPublication::default();
    assert_eq!(
        publication.query(viewport, UiPoint::new(1.0, 1.0), UiPoint::new(1.0, 1.0)),
        RuntimeUiInputQueryAdmission::Unpublished
    );
    assert_eq!(
        publication.query(
            viewport,
            UiPoint::new(f32::NAN, 1.0),
            UiPoint::new(1.0, 1.0),
        ),
        RuntimeUiInputQueryAdmission::Rejected(RuntimeUiInputQueryRejectReason::NonFinitePointer)
    );

    publication.publish(viewport, 0, []);
    assert!(matches!(
        publication.query(viewport, UiPoint::new(1.0, 1.0), UiPoint::new(1.0, 1.0)),
        RuntimeUiInputQueryAdmission::Published(query) if query.candidate_count() == 0
    ));
    assert_eq!(
        publication.query(
            viewport,
            UiPoint::new(f32::NAN, 1.0),
            UiPoint::new(1.0, 1.0),
        ),
        RuntimeUiInputQueryAdmission::Rejected(RuntimeUiInputQueryRejectReason::NonFinitePointer)
    );
    assert_eq!(
        publication.query(UVec2::ZERO, UiPoint::new(1.0, 1.0), UiPoint::new(1.0, 1.0)),
        RuntimeUiInputQueryAdmission::Rejected(RuntimeUiInputQueryRejectReason::DegenerateViewport)
    );
}

fn frame(hit_generation: u64, frames: &[UiFrame]) -> Arc<UiSurfaceFrame> {
    let entries = frames
        .iter()
        .enumerate()
        .map(|(index, frame)| UiHitTestEntry {
            node_id: UiNodeId::new(index as u64 + 1),
            frame: *frame,
            clip_frame: *frame,
            z_index: 0,
            paint_order: index as u64,
            control_id: None,
            route_node_index: index as u32,
        })
        .collect::<Vec<_>>();
    Arc::new(UiSurfaceFrame {
        domain_generations: UiSurfaceFrameDomainGenerations {
            hit_test: hit_generation,
            ..UiSurfaceFrameDomainGenerations::default()
        },
        hit_grid: Arc::new(UiHitTestGrid {
            entries: entries.into(),
            ..UiHitTestGrid::default()
        }),
        ..UiSurfaceFrame::default()
    })
}
