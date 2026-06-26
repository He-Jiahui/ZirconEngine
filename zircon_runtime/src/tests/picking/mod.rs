use crate::core::framework::picking::{
    hovered_hits_for_pointer, ray_from_viewport_point, run_picking_pipeline,
    sorted_hits_for_pointer, CameraRaySource, HitData, HitRecord, HitTarget, Pickable, PickingAxis,
    PickingBackend, PickingDebugFeed, PickingDebugMetricKind, PickingEventKind, PickingEventLabel,
    PickingEventState, PickingHoverMap, PickingPipelineInput, PickingPipelineReport,
    PickingPrimitive, PickingScheduleLabel, PickingSettings, PointerAction, PointerButton,
    PointerHits, PointerId, PointerInput, PointerLocation, PointerScrollUnit,
    PrimitivePickingBackend, RayId, RayMap,
};
use crate::core::framework::render::{
    ProjectionMode, RenderViewportHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3};

mod diagnostics;
mod hits_and_hover;
mod pipeline;
mod pointer_events;
mod rays;

fn hit(target: HitTarget, depth: f32) -> HitRecord {
    HitRecord::new(target, HitData::new(99, depth, None, None))
}

fn pointer_location(pointer: PointerId, x: f32, y: f32) -> PointerLocation {
    PointerLocation::new(pointer, RenderViewportHandle::new(1), Vec2::new(x, y))
}

fn event_labels(
    events: &[crate::core::framework::picking::PickingPointerEvent],
) -> Vec<PickingEventLabel> {
    events.iter().map(|event| event.label()).collect()
}

fn test_camera(projection_mode: ProjectionMode) -> ViewportCameraSnapshot {
    ViewportCameraSnapshot {
        transform: Transform::looking_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y),
        projection_mode,
        fov_y_radians: 60.0_f32.to_radians(),
        ortho_size: 5.0,
        z_near: 0.1,
        z_far: 200.0,
        aspect_ratio: 16.0 / 9.0,
        ..ViewportCameraSnapshot::default()
    }
}
