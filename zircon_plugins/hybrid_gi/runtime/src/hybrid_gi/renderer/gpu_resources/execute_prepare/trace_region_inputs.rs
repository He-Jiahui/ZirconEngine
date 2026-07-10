use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};

use super::super::gpu_trace_region_input::GpuTraceRegionInput;
use super::probe_quantization::scheduled_trace_region_scene_data_by_id;

pub(super) fn trace_region_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
) -> Vec<GpuTraceRegionInput> {
    scheduled_trace_region_scene_data_by_id(resolve_runtime, &prepare.scheduled_trace_region_ids)
        .into_iter()
        .map(|(region_id, scene_data)| {
            trace_region_input_from_runtime_scene_data(region_id, scene_data)
        })
        .collect()
}

fn trace_region_input_from_runtime_scene_data(
    region_id: u32,
    scene_data: HybridGiResolveTraceRegionSceneData,
) -> GpuTraceRegionInput {
    GpuTraceRegionInput {
        region_id,
        center_x_q: scene_data.center_x_q(),
        center_y_q: scene_data.center_y_q(),
        center_z_q: scene_data.center_z_q(),
        radius_q: scene_data.radius_q(),
        coverage_q: scene_data.coverage_q(),
        rt_lighting_rgb: pack_rgb8(scene_data.rt_lighting_rgb()),
        _padding1: 0,
    }
}

fn pack_rgb8(rgb: [u8; 3]) -> u32 {
    u32::from(rgb[0]) | (u32::from(rgb[1]) << 8) | (u32::from(rgb[2]) << 16)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn trace_region_inputs_project_only_prepared_runtime_scene_data() {
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids: vec![9, 9, 404],
            ..HybridGiPrepareFrame::default()
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(BTreeMap::from([(
                9,
                HybridGiResolveTraceRegionSceneData::new(2016, 2048, 2080, 144, 96, [16, 32, 64]),
            )]))
            .build();

        let inputs = trace_region_inputs(&prepare, Some(&runtime));

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].region_id, 9);
        assert_eq!(inputs[0].center_x_q, 2016);
        assert_eq!(inputs[0].center_z_q, 2080);
        assert_eq!(inputs[0].radius_q, 144);
        assert_eq!(inputs[0].coverage_q, 96);
        assert_eq!(inputs[0].rt_lighting_rgb, pack_rgb8([16, 32, 64]));
    }
}
