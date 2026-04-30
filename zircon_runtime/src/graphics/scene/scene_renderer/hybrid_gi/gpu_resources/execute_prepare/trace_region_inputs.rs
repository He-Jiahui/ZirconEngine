use crate::core::framework::render::RenderHybridGiExtract;

use crate::graphics::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData,
};

use super::super::gpu_trace_region_input::GpuTraceRegionInput;
use super::probe_quantization::scheduled_trace_region_scene_data_by_id;

pub(super) fn trace_region_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
) -> Vec<GpuTraceRegionInput> {
    scheduled_trace_region_scene_data_by_id(
        resolve_runtime,
        extract,
        &prepare.scheduled_trace_region_ids,
    )
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
    use std::collections::{BTreeMap, BTreeSet};

    use crate::core::framework::render::{RenderHybridGiExtract, RenderHybridGiTraceRegion};
    use crate::graphics::types::{HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData};

    use super::super::trace_region_limits::MAX_GPU_TRACE_REGION_INPUTS;
    use super::*;

    #[test]
    fn trace_region_inputs_deduplicate_scheduled_live_payload_ids() {
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids: vec![40, 40],
            ..Default::default()
        };
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![trace_region(40)],
            ..Default::default()
        };

        let inputs = trace_region_inputs(&prepare, None, Some(&extract));

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].region_id, 40);
    }

    #[test]
    fn trace_region_inputs_limit_legacy_scheduled_trace_region_budget_before_tail_payload() {
        let live_tail_region_id = 10_000;
        let mut scheduled_trace_region_ids =
            (0..MAX_GPU_TRACE_REGION_INPUTS as u32).collect::<Vec<_>>();
        scheduled_trace_region_ids.push(live_tail_region_id);
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids,
            ..Default::default()
        };
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: (0..MAX_GPU_TRACE_REGION_INPUTS as u32)
                .chain(std::iter::once(live_tail_region_id))
                .map(trace_region)
                .collect(),
            ..Default::default()
        };

        let inputs = trace_region_inputs(&prepare, None, Some(&extract));

        assert_eq!(inputs.len(), MAX_GPU_TRACE_REGION_INPUTS);
        assert!(!inputs
            .iter()
            .any(|input| input.region_id == live_tail_region_id));
    }

    #[test]
    fn trace_region_inputs_prefers_runtime_scene_data_over_legacy_extract_payload() {
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids: vec![40],
            ..Default::default()
        };
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![trace_region(40)],
            ..Default::default()
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(BTreeMap::from([(
                40,
                HybridGiResolveTraceRegionSceneData::new(7, 11, 13, 17, 19, [23, 29, 31]),
            )]))
            .build();

        let inputs = trace_region_inputs(&prepare, Some(&runtime), Some(&extract));

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].center_x_q, 7);
        assert_eq!(inputs[0].center_y_q, 11);
        assert_eq!(inputs[0].center_z_q, 13);
        assert_eq!(inputs[0].radius_q, 17);
        assert_eq!(inputs[0].coverage_q, 19);
        assert_eq!(inputs[0].rt_lighting_rgb, pack_rgb8([23, 29, 31]));
    }

    #[test]
    fn trace_region_inputs_filters_legacy_backed_runtime_scene_data_when_runtime_has_scene_truth() {
        let legacy_region_id = 40;
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids: vec![legacy_region_id],
            ..Default::default()
        };
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![trace_region(legacy_region_id)],
            ..Default::default()
        };
        let runtime = runtime_scene_truth_with_trace_regions(BTreeMap::from([(
            legacy_region_id,
            runtime_trace_region_scene_data([240, 96, 48]),
        )]));

        let inputs = trace_region_inputs(&prepare, Some(&runtime), Some(&extract));

        assert!(
            inputs.is_empty(),
            "runtime scene truth must keep legacy-backed RenderHybridGiTraceRegion data out of GPU trace inputs"
        );
    }

    #[test]
    fn trace_region_inputs_keeps_runtime_only_region_when_legacy_payload_is_scheduled() {
        let legacy_region_id = 40;
        let runtime_only_region_id = 41;
        let prepare = HybridGiPrepareFrame {
            scheduled_trace_region_ids: vec![
                legacy_region_id,
                runtime_only_region_id,
                runtime_only_region_id,
            ],
            ..Default::default()
        };
        let extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![trace_region(legacy_region_id)],
            ..Default::default()
        };
        let runtime = runtime_scene_truth_with_trace_regions(BTreeMap::from([
            (
                legacy_region_id,
                runtime_trace_region_scene_data([240, 96, 48]),
            ),
            (
                runtime_only_region_id,
                runtime_trace_region_scene_data([32, 64, 240]),
            ),
        ]));

        let inputs = trace_region_inputs(&prepare, Some(&runtime), Some(&extract));

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].region_id, runtime_only_region_id);
        assert_eq!(
            inputs[0].rt_lighting_rgb,
            pack_rgb8([32, 64, 240]),
            "runtime-only trace region data should survive the legacy-backed id filter"
        );
    }

    fn runtime_scene_truth_with_trace_regions(
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::fixture()
            .with_trace_region_scene_data(trace_region_scene_data)
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                300,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.25, 0.45, 0.75], 0.5),
            )]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([300]))
            .build()
    }

    fn runtime_trace_region_scene_data(
        rt_lighting_rgb: [u8; 3],
    ) -> HybridGiResolveTraceRegionSceneData {
        HybridGiResolveTraceRegionSceneData::new(7, 11, 13, 17, 19, rt_lighting_rgb)
    }

    fn trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
        RenderHybridGiTraceRegion {
            region_id,
            ..Default::default()
        }
    }
}
