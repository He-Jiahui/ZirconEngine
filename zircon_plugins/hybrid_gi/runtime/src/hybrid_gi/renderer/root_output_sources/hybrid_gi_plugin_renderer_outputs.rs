use crate::hybrid_gi::renderer::HybridGiGpuReadback;
use zircon_runtime::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderPluginRendererOutputs,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;

use super::hybrid_gi_readback_outputs::HybridGiReadbackOutputs;

pub(in crate::hybrid_gi::renderer) fn plugin_renderer_outputs_from_gpu_readback(
    readback: Option<HybridGiGpuReadback>,
) -> RenderPluginRendererOutputs {
    let mut readback_outputs = HybridGiReadbackOutputs::default();
    readback_outputs.store_gpu_readback(readback);

    RenderPluginRendererOutputs {
        hybrid_gi: readback_outputs.take_neutral_readback_outputs(),
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

pub(crate) fn runtime_prepare_renderer_outputs(
    context: &RuntimePrepareCollectorContext<'_>,
) -> RenderPluginRendererOutputs {
    // Keep the collector honest: mirror only neutral provider sidebands and do
    // not fabricate a concrete HybridGiGpuReadback when none exists.
    plugin_renderer_outputs_from_hybrid_gi_readback(
        context.prepared_hybrid_gi_readback_outputs().clone(),
    )
}

#[cfg(test)]
mod tests {
    use crate::hybrid_gi::renderer::{HybridGiGpuReadback, HybridGiScenePrepareResourcesSnapshot};

    use super::*;

    #[test]
    fn plugin_renderer_outputs_package_gpu_readback_under_hybrid_gi() {
        let mut scene_prepare = HybridGiScenePrepareResourcesSnapshot::new(
            1,
            vec![9],
            vec![1],
            vec![3],
            8,
            4,
            (64, 32),
            (16, 16),
            2,
        );
        scene_prepare.store_voxel_resource_samples(
            vec![(9, [70, 80, 90, 255])],
            vec![(9, 0b1011)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );

        let outputs = plugin_renderer_outputs_from_gpu_readback(Some(HybridGiGpuReadback::new(
            vec![(5, 7)],
            vec![11],
            vec![21],
            vec![(11, [1, 2, 3])],
            vec![(11, [7, 8, 9])],
            Some(scene_prepare),
        )));

        assert_eq!(outputs.hybrid_gi.cache_entries[0].key, 5);
        assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![11]);
        assert_eq!(outputs.hybrid_gi.completed_trace_region_ids, vec![21]);
        assert_eq!(
            outputs.hybrid_gi.scene_prepare.voxel_occupancy_masks[0].occupancy_mask,
            0b1011
        );
        assert!(outputs.virtual_geometry.is_empty());
        assert!(outputs.particles.is_empty());
        assert!(!outputs.is_empty());
    }

    #[test]
    fn runtime_prepare_renderer_outputs_do_not_fabricate_hybrid_gi_readbacks() {
        let outputs = plugin_renderer_outputs_from_hybrid_gi_readback(
            RenderHybridGiReadbackOutputs::default(),
        );

        assert!(outputs.is_empty());
        assert!(outputs.hybrid_gi.is_empty());
    }

    #[test]
    fn runtime_prepare_renderer_outputs_package_prepared_hybrid_gi_sideband() {
        let outputs =
            plugin_renderer_outputs_from_hybrid_gi_readback(RenderHybridGiReadbackOutputs {
                completed_probe_ids: vec![31, 32],
                ..RenderHybridGiReadbackOutputs::default()
            });

        assert_eq!(outputs.hybrid_gi.completed_probe_ids, vec![31, 32]);
        assert!(outputs.virtual_geometry.is_empty());
        assert!(outputs.particles.is_empty());
    }
}
