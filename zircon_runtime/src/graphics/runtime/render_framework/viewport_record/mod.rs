mod camera_history_key;
mod capture;
mod descriptor;
mod generation;
mod history;
mod motion_vector_camera;
mod new;
mod particle_previous_sprites;
mod pipeline;
mod product_reports;
mod quality_profile;
mod runtime_states;
mod surface;
mod temporal_frame_index;
mod viewport_record;

pub(in crate::graphics::runtime::render_framework) use camera_history_key::ViewportCameraHistoryKey;
pub(in crate::graphics::runtime::render_framework) use viewport_record::ViewportRecord;
