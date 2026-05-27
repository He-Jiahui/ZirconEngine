//! Runtime-owned picking contracts shared by editor tools, dev tools, and backends.

mod backend;
mod debug_feed;
mod hit_data;
mod hit_record;
mod hit_target;
mod hover_map;
mod pickable;
mod pipeline;
mod pointer_button;
mod pointer_event;
mod pointer_event_state;
mod pointer_hits;
mod pointer_id;
mod pointer_input;
mod pointer_location;
mod pointer_phase;
mod primitive_backend;
mod ray;
mod ray_map;
mod report;
mod schedule_label;
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
    run_picking_pipeline, PickingPipelineInput, PickingPipelineOutput, PickingPipelineStageReport,
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
