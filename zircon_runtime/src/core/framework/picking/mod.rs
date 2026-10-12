//! Runtime-owned picking contracts shared by editor tools, dev tools, and backends.

#[path = "backend.rs"]
mod backend;
#[path = "debug_feed.rs"]
mod debug_feed;
#[path = "hit_data.rs"]
mod hit_data;
#[path = "hit_record.rs"]
mod hit_record;
#[path = "hit_target.rs"]
mod hit_target;
#[path = "hover_map.rs"]
mod hover_map;
#[path = "pickable.rs"]
mod pickable;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "pointer_button.rs"]
mod pointer_button;
#[path = "pointer_event.rs"]
mod pointer_event;
#[path = "pointer_event_state.rs"]
mod pointer_event_state;
#[path = "pointer_hits.rs"]
mod pointer_hits;
#[path = "pointer_id.rs"]
mod pointer_id;
#[path = "pointer_input.rs"]
mod pointer_input;
#[path = "pointer_location.rs"]
mod pointer_location;
#[path = "pointer_phase.rs"]
mod pointer_phase;
#[path = "primitive_backend.rs"]
mod primitive_backend;
#[path = "ray.rs"]
mod ray;
#[path = "ray_map.rs"]
mod ray_map;
#[path = "report.rs"]
mod report;
#[path = "schedule_label.rs"]
mod schedule_label;
#[path = "settings.rs"]
mod settings;

pub use backend::{PickingBackend, PickingBackendCapability, PickingBackendInfo};
pub use debug_feed::{
    PickingDebugFeed, PickingDebugMetric, PickingDebugMetricKind, PickingDebugPointerRow,
};
pub use hit_data::HitData;
pub use hit_record::HitRecord;
pub use hit_target::{HitTarget, PickingAxis, PickingTargetPriority};
pub use hover_map::PickingHoverMap;
pub use pickable::Pickable;
pub use pipeline::{
    resolve_picking_outputs, run_picking_pipeline, PickingPipelineInput, PickingPipelineOutput,
    PickingPipelineStageReport,
};
pub use pointer_button::PointerButton;
pub use pointer_event::{PickingEventKind, PickingEventLabel, PickingPointerEvent};
pub use pointer_event_state::PickingEventState;
pub use pointer_hits::{hovered_hits_for_pointer, sorted_hits_for_pointer, PointerHits};
pub use pointer_id::PointerId;
pub use pointer_input::{PointerAction, PointerInput, PointerScrollUnit};
pub use pointer_location::PointerLocation;
pub use pointer_phase::PointerPhase;
pub use primitive_backend::{PickingPrimitive, PickingPrimitiveShape, PrimitivePickingBackend};
pub use ray::{ray_from_viewport_point, PointerRay};
pub use ray_map::{CameraRaySource, RayId, RayMap};
pub use report::{PickingPipelineReport, PickingPointerPipelineReport};
pub use schedule_label::PickingScheduleLabel;
pub use settings::PickingSettings;

#[cfg(test)]
pub(crate) use pointer_hits::{reset_sorted_hit_projection_metrics, sorted_hit_projection_metrics};
