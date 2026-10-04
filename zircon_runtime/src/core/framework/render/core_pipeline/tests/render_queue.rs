use super::RenderQueueValue;
use crate::core::framework::render::{CorePipelineKind, RenderMaterialAlphaMode, RenderPhase};

#[test]
fn render_queue_defaults_follow_alpha_mode() {
    assert_eq!(
        RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Opaque),
        RenderQueueValue::GEOMETRY
    );
    assert_eq!(
        RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Mask { cutoff: 0.5 }),
        RenderQueueValue::ALPHA_TEST
    );
    assert_eq!(
        RenderQueueValue::from_alpha_mode(&RenderMaterialAlphaMode::Blend),
        RenderQueueValue::TRANSPARENT
    );
}

#[test]
fn render_queue_segments_map_to_core_pipeline_phases() {
    assert_eq!(
        RenderQueueValue::BACKGROUND.phase(CorePipelineKind::Core3d),
        RenderPhase::Opaque3d
    );
    assert_eq!(
        RenderQueueValue::GEOMETRY.phase(CorePipelineKind::Core2d),
        RenderPhase::Opaque2d
    );
    assert_eq!(
        RenderQueueValue::ALPHA_TEST.phase(CorePipelineKind::Core3d),
        RenderPhase::AlphaMask3d
    );
    assert_eq!(
        RenderQueueValue::GEOMETRY_LAST.phase(CorePipelineKind::Core2d),
        RenderPhase::AlphaMask2d
    );
    assert_eq!(
        RenderQueueValue::new(RenderQueueValue::GEOMETRY_LAST.raw() + 1)
            .phase(CorePipelineKind::Core3d),
        RenderPhase::Transparent3d
    );
    assert_eq!(
        RenderQueueValue::TRANSPARENT.phase(CorePipelineKind::Core2d),
        RenderPhase::Transparent2d
    );
    assert_eq!(
        RenderQueueValue::OVERLAY.phase(CorePipelineKind::Core3d),
        RenderPhase::Overlay
    );
}

#[test]
fn authored_unity_queue_values_override_alpha_mode() {
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 2_900),
        RenderQueueValue::new(2_900)
    );
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 2_900)
            .phase(CorePipelineKind::Core3d),
        RenderPhase::Transparent3d
    );
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Blend, 2_000)
            .phase(CorePipelineKind::Core2d),
        RenderPhase::Opaque2d
    );
}

#[test]
fn authored_queue_offsets_are_clamped_to_material_window() {
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Blend, -10),
        RenderQueueValue::new(2_990)
    );
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, -500),
        RenderQueueValue::new(1_900)
    );
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 500),
        RenderQueueValue::new(2_100)
    );
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, 0),
        RenderQueueValue::GEOMETRY
    );
}

#[test]
fn authored_queue_maximum_offset_clamps_without_narrowing() {
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, i32::MAX),
        RenderQueueValue::new(2_100)
    );
}

#[test]
fn authored_queue_minimum_offset_clamps_without_narrowing() {
    assert_eq!(
        RenderQueueValue::from_authored_queue(&RenderMaterialAlphaMode::Opaque, i32::MIN),
        RenderQueueValue::new(1_900)
    );
}

#[test]
fn deserialized_queue_value_clamps_to_the_sort_key_domain() {
    let queue: RenderQueueValue = serde_json::from_value(serde_json::json!(u16::MAX))
        .expect("queue value should deserialize");

    assert_eq!(queue, RenderQueueValue::MAX);
}
