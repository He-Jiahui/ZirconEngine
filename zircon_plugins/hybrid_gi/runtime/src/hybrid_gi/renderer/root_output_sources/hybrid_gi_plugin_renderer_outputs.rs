use crate::hybrid_gi::renderer::HybridGiGpuReadback;
use zircon_runtime::core::framework::render::{
    RenderHybridGiGlobalSdfStats, RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
};

use super::hybrid_gi_readback_outputs::HybridGiReadbackOutputs;

pub(in crate::hybrid_gi::renderer) fn plugin_renderer_outputs_from_gpu_readback(
    readback: Option<HybridGiGpuReadback>,
    global_sdf_stats: Option<RenderHybridGiGlobalSdfStats>,
) -> RenderPluginRendererOutputs {
    let mut readback_outputs = HybridGiReadbackOutputs::default();
    readback_outputs.store_gpu_readback(readback);
    let mut hybrid_gi = readback_outputs.take_neutral_readback_outputs();
    hybrid_gi.global_sdf_stats = global_sdf_stats;

    RenderPluginRendererOutputs {
        hybrid_gi,
        ..RenderPluginRendererOutputs::default()
    }
}

pub(in crate::hybrid_gi::renderer) fn plugin_renderer_outputs_from_hybrid_gi_readback(
    hybrid_gi: RenderHybridGiReadbackOutputs,
) -> RenderPluginRendererOutputs {
    RenderPluginRendererOutputs {
        hybrid_gi,
        ..RenderPluginRendererOutputs::default()
    }
}

#[cfg(test)]
#[path = "tests/hybrid_gi_plugin_renderer_outputs.rs"]
mod tests;
