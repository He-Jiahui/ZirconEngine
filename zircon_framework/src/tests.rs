use zircon_math::UVec2;

use crate::{
    input::{InputButton, InputEvent, InputEventRecord, InputSnapshot},
    render::{
        CapturedFrame, FrameHistoryHandle, RenderFrameworkError, RenderPipelineHandle,
        RenderQualityProfile, RenderStats, RenderViewportDescriptor, RenderViewportHandle,
        RenderingBackendInfo,
    },
    scene::{LevelSummary, Mobility, WorldHandle},
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
    let level = LevelSummary {
        handle: WorldHandle::new(42),
        entity_count: 5,
        selected_entity: Some(3),
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
    assert_eq!(level.handle.get(), 42);
    assert_eq!(Mobility::default(), Mobility::Dynamic);
    assert_eq!(
        RenderFrameworkError::UnknownPipeline { pipeline: 9 },
        RenderFrameworkError::UnknownPipeline { pipeline: 9 }
    );

    let history = FrameHistoryHandle::new(19);
    assert_eq!(history.raw(), 19);
}
