mod hybrid_gi_neutral_readback_outputs;
mod hybrid_gi_plugin_renderer_outputs;
mod hybrid_gi_readback_outputs;
mod runtime_prepare_collector;
mod scene_prepare_resources;
#[cfg(test)]
mod scene_renderer_hybrid_gi;

pub(in crate::hybrid_gi::renderer) use hybrid_gi_plugin_renderer_outputs::plugin_renderer_outputs_from_gpu_readback;
pub(crate) use runtime_prepare_collector::runtime_prepare_collector;
