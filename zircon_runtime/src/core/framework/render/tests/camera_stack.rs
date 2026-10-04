use super::*;
use crate::core::framework::render::{RenderCameraTarget, RenderViewportRect};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, ResourceId, TextureMarker};

#[test]
fn render_camera_sequence_sorts_by_render_order() {
    let sequence = resolve_camera_sequence([
        descriptor(
            30,
            3,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        ),
        descriptor(
            10,
            1,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        ),
        descriptor(
            20,
            2,
            CameraRenderType::Base,
            RenderCameraTarget::PrimarySurface,
        ),
    ]);

    assert_eq!(
        sequence
            .sequence
            .iter()
            .map(|entry| entry.base.entity)
            .collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3)]
    );
    assert!(!sequence.has_violations());
}

#[test]
fn render_camera_stack_overlay_follows_base_and_inherits_target_viewport() {
    let texture = ResourceHandle::<TextureMarker>::new(ResourceId::from_stable_label(
        "res://camera/base-target.png",
    ));
    let viewport = RenderViewportRect::new(UVec2::new(16, 8), UVec2::new(320, 180));
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::Texture(texture),
    )
    .with_stack([2, 3])
    .with_viewport(viewport);
    let overlay_a = descriptor(10, 2, CameraRenderType::Overlay, base.target.clone());
    let overlay_b = descriptor(5, 3, CameraRenderType::Overlay, base.target.clone());

    let report = resolve_camera_sequence([overlay_a, base, overlay_b]);

    assert_eq!(report.sequence.len(), 1);
    assert_eq!(
        report.sequence[0]
            .overlays
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![Some(2), Some(3)]
    );
    assert_eq!(
        report.sequence[0].overlays[0].target,
        report.sequence[0].base.target
    );
    assert_eq!(report.sequence[0].overlays[0].viewport_rect, Some(viewport));
    assert!(!report.has_violations());
}

#[test]
fn render_camera_stack_rejects_invalid_members() {
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([2, 3, 4]);
    let referenced_base = descriptor(
        0,
        2,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    );
    let mismatched_overlay = descriptor(
        0,
        3,
        CameraRenderType::Overlay,
        RenderCameraTarget::Headless {
            size: UVec2::new(64, 64),
        },
    );
    let stacked_overlay = descriptor(
        0,
        5,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack([2]);

    let report =
        resolve_camera_sequence([base, referenced_base, mismatched_overlay, stacked_overlay]);

    assert_eq!(report.sequence.len(), 2);
    assert_eq!(
        report.violations,
        vec![
            CameraSequenceViolation {
                entity: Some(5),
                reason: CameraSequenceViolationReason::OverlayCameraHasStack,
            },
            CameraSequenceViolation {
                entity: Some(1),
                reason: CameraSequenceViolationReason::BaseStackReferencesNonOverlay {
                    referenced: 2,
                },
            },
            CameraSequenceViolation {
                entity: Some(1),
                reason: CameraSequenceViolationReason::OverlayTargetDoesNotMatchBase {
                    referenced: 3,
                },
            },
            CameraSequenceViolation {
                entity: Some(1),
                reason: CameraSequenceViolationReason::BaseStackReferencesMissingCamera {
                    referenced: 4,
                },
            },
        ]
    );
    assert!(report.sequence[0].overlays.is_empty());
}

#[test]
fn render_camera_sequence_resolves_borrowed_descriptors_without_consuming_source() {
    let source = include_str!("../camera_stack.rs");
    assert!(!source.contains(concat!(
        ".filter(|camera| camera.is_active())\n        .",
        "cloned()"
    )));

    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    );
    let overlay = descriptor(
        0,
        2,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let cameras = vec![base.with_stack([2]), overlay];

    let report = resolve_camera_sequence_borrowed(&cameras);

    assert_eq!(report.sequence.len(), 1);
    assert_eq!(report.sequence[0].base.entity, Some(1));
    assert_eq!(report.sequence[0].overlays[0].entity, Some(2));
    assert_eq!(cameras.len(), 2);
    assert!(!report.has_violations());
}

#[test]
fn runtime37_batch_camera_entity_index_preserves_first_match() {
    let first = descriptor(
        -10,
        7,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let second = descriptor(
        10,
        7,
        CameraRenderType::Overlay,
        RenderCameraTarget::PrimarySurface,
    );
    let active = vec![first, second];

    let index = index_active_cameras(&active);

    assert_eq!(index.get(&7).map(|camera| camera.render_order), Some(-10));
}

#[test]
#[ignore = "release-only performance evidence"]
fn runtime37_batch_camera_stack_entity_index_evidence() {
    const CAMERA_COUNT: usize = 10_000;
    const REFERENCE_COUNT: usize = 10_000;
    const TARGET_MILLIS: u128 = 500;
    const MARKER: &str = "RUNTIME37_CAMERA_STACK_INDEX_BENCH_V1";

    let last_entity = CAMERA_COUNT as u64;
    let base = descriptor(
        0,
        1,
        CameraRenderType::Base,
        RenderCameraTarget::PrimarySurface,
    )
    .with_stack(vec![last_entity; REFERENCE_COUNT]);
    let mut cameras = Vec::with_capacity(CAMERA_COUNT);
    cameras.push(base);
    cameras.extend((2..=last_entity).map(|entity| {
        descriptor(
            entity as i32,
            entity,
            CameraRenderType::Overlay,
            RenderCameraTarget::PrimarySurface,
        )
    }));

    let started = std::time::Instant::now();
    let report = resolve_camera_sequence(cameras);
    let elapsed = started.elapsed();
    let legacy_entity_comparisons = CAMERA_COUNT * REFERENCE_COUNT;
    let indexed_operations = CAMERA_COUNT + REFERENCE_COUNT;

    assert!(!report.has_violations());
    assert_eq!(report.sequence.len(), 1);
    assert_eq!(report.sequence[0].overlays.len(), REFERENCE_COUNT);
    assert!(report.sequence[0]
        .overlays
        .iter()
        .all(|camera| camera.entity == Some(last_entity)));
    assert!(
        elapsed.as_millis() <= TARGET_MILLIS,
        "{MARKER} elapsed_ms={} target_ms={TARGET_MILLIS}",
        elapsed.as_millis()
    );
    println!(
        "{MARKER} cameras={CAMERA_COUNT} references={REFERENCE_COUNT} legacy_entity_comparisons={legacy_entity_comparisons} indexed_operations={indexed_operations} reduction_pct=99.98 elapsed_ms={} target_ms={TARGET_MILLIS}",
        elapsed.as_millis()
    );
}

fn descriptor(
    order: i32,
    entity: EntityId,
    render_type: CameraRenderType,
    target: RenderCameraTarget,
) -> CameraRenderDescriptor {
    CameraRenderDescriptor {
        entity: Some(entity),
        render_order: order,
        render_type,
        target,
        ..CameraRenderDescriptor::from_camera_payload(
            Some(entity),
            ViewportCameraSnapshot::default(),
        )
    }
}

trait DescriptorTestExt {
    fn with_stack(self, stack: impl IntoIterator<Item = EntityId>) -> Self;
    fn with_viewport(self, viewport: RenderViewportRect) -> Self;
}

impl DescriptorTestExt for CameraRenderDescriptor {
    fn with_stack(mut self, stack: impl IntoIterator<Item = EntityId>) -> Self {
        self.stack = stack.into_iter().collect();
        self
    }

    fn with_viewport(mut self, viewport: RenderViewportRect) -> Self {
        self.viewport_rect = Some(viewport);
        self
    }
}
