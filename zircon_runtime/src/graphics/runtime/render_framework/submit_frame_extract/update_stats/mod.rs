mod base_stats;
mod hybrid_gi_stats;
mod particle_stats;
mod quality_profile;
mod update;
mod virtual_geometry_stats;

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) use update::update_stats;
