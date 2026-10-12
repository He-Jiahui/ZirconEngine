#[path = "../../hybrid_gi_neutral_readback_outputs.rs"]
mod hybrid_gi_neutral_readback_outputs;
#[path = "../../hybrid_gi_plugin_renderer_outputs.rs"]
mod hybrid_gi_plugin_renderer_outputs;
#[path = "../../outputs/hybrid_gi_readback_outputs.rs"]
mod hybrid_gi_readback_outputs;
#[path = "../../outputs/runtime_prepare_collector.rs"]
mod runtime_prepare_collector;
#[path = "../../outputs/scene_prepare_resources.rs"]
mod scene_prepare_resources;
#[cfg(test)]
#[path = "tests/mod.rs"]
mod scene_renderer_hybrid_gi;

pub(in crate::hybrid_gi::renderer) use hybrid_gi_plugin_renderer_outputs::plugin_renderer_outputs_from_gpu_readback;
pub(crate) use runtime_prepare_collector::runtime_prepare_collector;
