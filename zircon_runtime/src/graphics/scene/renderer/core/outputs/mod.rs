#[path = "../../../core/outputs/reset_last_runtime_outputs.rs"]
mod reset_last_runtime_outputs;
#[path = "../../../core/outputs/store_last_runtime_outputs.rs"]
mod store_last_runtime_outputs;
#[path = "../../../../core/outputs/take_last_hybrid_gi_readback_outputs.rs"]
mod take_last_hybrid_gi_readback_outputs;
#[path = "../../../../outputs/take_last_particle_gpu_readback_outputs.rs"]
mod take_last_particle_gpu_readback_outputs;
#[path = "../../../../take_last_virtual_geometry_readback_outputs.rs"]
mod take_last_virtual_geometry_readback_outputs;

pub(in crate::graphics::scene::scene_renderer::core) use reset_last_runtime_outputs::reset_last_runtime_outputs;
pub(in crate::graphics::scene::scene_renderer::core) use store_last_runtime_outputs::store_last_runtime_outputs;
