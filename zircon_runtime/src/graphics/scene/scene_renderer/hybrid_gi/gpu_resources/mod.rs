mod buffer_helpers;
mod execute_prepare;
mod gpu_pending_probe_input;
mod gpu_resident_probe_input;
mod gpu_trace_region_input;
mod hybrid_gi_completion_params;
mod hybrid_gi_gpu_resources;
mod new;
mod seed_quantization;

pub(in crate::graphics::scene::scene_renderer) use hybrid_gi_gpu_resources::HybridGiGpuResources;
