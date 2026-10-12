#[path = "../../viewport_record/camera_history_key.rs"]
mod camera_history_key;
#[path = "capture.rs"]
mod capture;
#[path = "capture_mailbox.rs"]
mod capture_mailbox;
#[path = "construct.rs"]
mod construct;
#[path = "descriptor.rs"]
mod descriptor;
#[path = "generation.rs"]
mod generation;
#[path = "history.rs"]
mod history;
#[path = "../../viewport_record/motion_vector_camera.rs"]
mod motion_vector_camera;
#[path = "../../viewport_record/particle_previous_sprites.rs"]
mod particle_previous_sprites;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "product_reports.rs"]
mod product_reports;
#[path = "quality_profile.rs"]
mod quality_profile;
#[path = "runtime_states.rs"]
mod runtime_states;
#[path = "surface.rs"]
mod surface;
#[path = "../../viewport_record/temporal_frame_index.rs"]
mod temporal_frame_index;
#[path = "viewport_record.rs"]
mod viewport_record;
#[path = "../../viewport_record/visible_spatial_query.rs"]
mod visible_spatial_query;

pub(in crate::graphics::runtime::render_framework) use camera_history_key::ViewportCameraHistoryKey;
pub(in crate::graphics::runtime::render_framework) use viewport_record::ViewportRecord;
