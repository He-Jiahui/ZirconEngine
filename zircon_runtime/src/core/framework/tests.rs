use crate::core::math::UVec2;

use super::{
    animation::{AnimationParameterValue, AnimationPlaybackSettings, AnimationTrackPath},
    input::{InputButton, InputEvent, InputEventRecord, InputSnapshot},
    physics::{PhysicsCombineRule, PhysicsMaterialMetadata, PhysicsSettings},
    render::{
        CapturedFrame, FrameHistoryHandle, RenderFrameworkError, RenderPipelineHandle,
        RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
        RenderingBackendInfo,
    },
    scene::{ComponentPropertyPath, EntityPath, LevelSummary, Mobility, WorldHandle},
};

#[test]
fn framework_contract_types_are_constructible() {
    let viewport = RenderViewportHandle::new(7);
    let pipeline = RenderPipelineHandle::new(11);
    let descriptor = RenderViewportDescriptor::new(UVec2::new(320, 240));
    let profile =
        RenderQualityProfile::new("editor-high").with_pipeline_asset(RenderPipelineHandle::new(11));
    let frame = CapturedFrame::new(320, 240, vec![0; 320 * 240 * 4], 3);
    let stats = RenderStats::default();
    let backend = RenderingBackendInfo {
        backend_name: "wgpu".into(),
        supports_runtime_preview: true,
        supports_shared_texture_viewports: true,
    };
    let input = InputSnapshot {
        cursor_position: [12.0, 24.0],
        pressed_buttons: vec![InputButton::MouseLeft],
        wheel_accumulator: 1.0,
    };
    let event = InputEventRecord {
        sequence: 1,
        timestamp_millis: 2,
        event: InputEvent::ButtonPressed(InputButton::MouseRight),
    };
    let entity_path = EntityPath::parse("Root/Hero").unwrap();
    let property_path = ComponentPropertyPath::parse("Transform.translation").unwrap();
    let track_path = AnimationTrackPath::new(entity_path.clone(), property_path.clone());
    let playback = AnimationPlaybackSettings::default();
    let material = PhysicsMaterialMetadata::default();
    let physics = PhysicsSettings::default();
    let level = LevelSummary {
        handle: WorldHandle::new(42),
        entity_count: 5,
        active_camera: Some(4),
    };

    assert_eq!(viewport.raw(), 7);
    assert_eq!(pipeline.raw(), 11);
    assert_eq!(descriptor.size, UVec2::new(320, 240));
    assert_eq!(
        profile.pipeline_override,
        Some(RenderPipelineHandle::new(11))
    );
    assert_eq!(frame.generation, 3);
    assert_eq!(stats.active_viewports, 0);
    assert_eq!(backend.backend_name, "wgpu");
    assert_eq!(input.cursor_position, [12.0, 24.0]);
    assert_eq!(event.sequence, 1);
    assert_eq!(entity_path.to_string(), "Root/Hero");
    assert_eq!(property_path.to_string(), "Transform.translation");
    assert_eq!(track_path.to_string(), "Root/Hero:Transform.translation");
    assert_eq!(AnimationParameterValue::Trigger, AnimationParameterValue::Trigger);
    assert!(playback.enabled && playback.property_tracks);
    assert_eq!(material.friction_combine, PhysicsCombineRule::Average);
    assert_eq!(physics.fixed_hz, 60);
    assert_eq!(level.handle.get(), 42);
    assert_eq!(Mobility::default(), Mobility::Dynamic);
    assert_eq!(
        RenderFrameworkError::UnknownPipeline { pipeline: 9 },
        RenderFrameworkError::UnknownPipeline { pipeline: 9 }
    );

    let history = FrameHistoryHandle::new(19);
    assert_eq!(history.raw(), 19);
}
