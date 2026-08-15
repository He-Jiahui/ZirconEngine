use std::collections::BTreeSet;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use super::super::super::super::buffer_helpers::create_u32_storage_buffer;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use crate::hybrid_gi::renderer::{HybridGiGpuResources, HybridGiScenePrepareResourcesSnapshot};

const DEFAULT_RAYS_PER_TRACE_TILE: u32 = 8;
const MIN_RAYS_PER_TRACE_TILE: u32 = 4;
const MAX_RAYS_PER_TRACE_TILE: u32 = 16;
const PROBE_TRACE_TILE_WORDS_PER_RECORD: usize = 4;
const PROBE_TRACE_TILE_GENERATION_WORKGROUP_SIZE: [u32; 3] = [8, 8, 1];
const PROBE_TRACE_TILE_INDIRECT_ARG_WORD_COUNT: usize = 4;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ProbeTraceTileGenerationParams {
    record_count: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
}

pub(super) struct ScenePrepareProbeTraceTileResources {
    pub(super) probe_trace_tile_seed_buffer: Option<wgpu::Buffer>,
    pub(super) probe_trace_tile_params_buffer: Option<wgpu::Buffer>,
    pub(super) probe_trace_tile_buffer: Option<wgpu::Buffer>,
    pub(super) probe_trace_indirect_args_buffer: Option<wgpu::Buffer>,
    pub(super) probe_trace_tile_word_count: usize,
    pub(super) probe_trace_tile_record_count: usize,
    pub(super) probe_trace_indirect_arg_word_count: usize,
}

pub(super) fn store_scene_prepare_probe_trace_tiles(
    snapshot: &mut HybridGiScenePrepareResourcesSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
    tracing_budget: Option<u32>,
) {
    let tiles = probe_trace_tiles(snapshot, inputs, tracing_budget);
    let dispatch = probe_trace_dispatch(tiles.len());
    snapshot.store_probe_trace_tiles(tiles, dispatch);
}

pub(super) fn scene_prepare_probe_trace_tile_resources(
    resources: &HybridGiGpuResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
) -> ScenePrepareProbeTraceTileResources {
    let words = probe_trace_tile_words(snapshot);
    if words.is_empty() {
        return ScenePrepareProbeTraceTileResources {
            probe_trace_tile_seed_buffer: None,
            probe_trace_tile_params_buffer: None,
            probe_trace_tile_buffer: None,
            probe_trace_indirect_args_buffer: None,
            probe_trace_tile_word_count: 0,
            probe_trace_tile_record_count: 0,
            probe_trace_indirect_arg_word_count: 0,
        };
    }

    let probe_trace_tile_record_count = snapshot.probe_trace_tiles().len();
    let probe_trace_tile_seed_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-probe-trace-tile-seeds",
        &words,
        wgpu::BufferUsages::STORAGE,
    );
    let probe_trace_tile_params_buffer = create_probe_trace_tile_generation_params_buffer(
        device,
        probe_trace_tile_record_count as u32,
    );
    let probe_trace_tile_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-probe-trace-tiles",
        &vec![0; words.len()],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let probe_trace_indirect_args_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-probe-trace-indirect-args",
        &vec![0; PROBE_TRACE_TILE_INDIRECT_ARG_WORD_COUNT],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDIRECT,
    );
    encode_probe_trace_tile_generation(
        resources,
        device,
        encoder,
        &probe_trace_tile_params_buffer,
        &probe_trace_tile_seed_buffer,
        &probe_trace_tile_buffer,
        &probe_trace_indirect_args_buffer,
        probe_trace_tile_record_count,
    );
    ScenePrepareProbeTraceTileResources {
        probe_trace_tile_seed_buffer: Some(probe_trace_tile_seed_buffer),
        probe_trace_tile_params_buffer: Some(probe_trace_tile_params_buffer),
        probe_trace_tile_buffer: Some(probe_trace_tile_buffer),
        probe_trace_indirect_args_buffer: Some(probe_trace_indirect_args_buffer),
        probe_trace_tile_word_count: words.len(),
        probe_trace_tile_record_count,
        probe_trace_indirect_arg_word_count: PROBE_TRACE_TILE_INDIRECT_ARG_WORD_COUNT,
    }
}

fn create_probe_trace_tile_generation_params_buffer(
    device: &wgpu::Device,
    record_count: u32,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-hybrid-gi-scene-prepare-probe-trace-tile-generation-params"),
        contents: bytemuck::bytes_of(&ProbeTraceTileGenerationParams {
            record_count,
            _pad0: 0,
            _pad1: 0,
            _pad2: 0,
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn encode_probe_trace_tile_generation(
    resources: &HybridGiGpuResources,
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    params_buffer: &wgpu::Buffer,
    seed_buffer: &wgpu::Buffer,
    output_buffer: &wgpu::Buffer,
    indirect_args_buffer: &wgpu::Buffer,
    record_count: usize,
) {
    if record_count == 0 {
        return;
    }

    let bind_group = create_probe_trace_tile_generation_bind_group(
        device,
        &resources.probe_trace_tile_generation_bind_group_layout,
        params_buffer,
        seed_buffer,
        output_buffer,
        indirect_args_buffer,
    );
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("HybridGiGenerateProbeTraceTilesPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(&resources.probe_trace_tile_generation_pipeline);
    pass.set_bind_group(0, &bind_group, &[]);
    pass.dispatch_workgroups(
        1,
        1,
        record_count as u32 / PROBE_TRACE_TILE_GENERATION_WORKGROUP_SIZE[2],
    );
}

fn create_probe_trace_tile_generation_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    seed_buffer: &wgpu::Buffer,
    output_buffer: &wgpu::Buffer,
    indirect_args_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-generate-probe-trace-tiles-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: seed_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: indirect_args_buffer.as_entire_binding(),
            },
        ],
    })
}

fn probe_trace_tiles(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
    tracing_budget: Option<u32>,
) -> Vec<(u32, u32, u32, u32)> {
    let rays_per_trace_tile = rays_per_trace_tile(tracing_budget);
    let mut tiles = snapshot
        .voxel_clipmap_cell_occupancy_counts()
        .iter()
        .filter(|(_, _, occupancy)| *occupancy > 0)
        .map(|(clipmap_id, cell_id, occupancy)| {
            (
                0,
                *clipmap_id,
                *cell_id,
                occupancy.saturating_mul(rays_per_trace_tile),
            )
        })
        .collect::<Vec<_>>();

    if tiles.is_empty() {
        tiles = surface_cache_trace_tiles(snapshot, inputs, rays_per_trace_tile);
    }

    tiles
        .into_iter()
        .enumerate()
        .map(|(tile_id, (_, probe_id, trace_region_id, ray_count))| {
            (tile_id as u32, probe_id, trace_region_id, ray_count.max(1))
        })
        .collect()
}

fn surface_cache_trace_tiles(
    snapshot: &HybridGiScenePrepareResourcesSnapshot,
    inputs: &HybridGiPrepareExecutionInputs,
    rays_per_trace_tile: u32,
) -> Vec<(u32, u32, u32, u32)> {
    let known_slots = inputs
        .scene_card_capture_requests
        .iter()
        .map(|request| (request.atlas_slot_id, request.card_id, request.page_id))
        .chain(
            inputs
                .scene_surface_cache_page_contents
                .iter()
                .map(|page| (page.atlas_slot_id, page.owner_card_id, page.page_id)),
        )
        .collect::<BTreeSet<_>>();

    snapshot
        .occupied_atlas_slots()
        .iter()
        .filter_map(|slot_id| {
            known_slots
                .iter()
                .find(|(known_slot_id, _, _)| known_slot_id == slot_id)
                .map(|(atlas_slot_id, card_id, _)| {
                    (0, *card_id, *atlas_slot_id, rays_per_trace_tile)
                })
        })
        .collect()
}

fn rays_per_trace_tile(tracing_budget: Option<u32>) -> u32 {
    tracing_budget
        .map(|budget| (budget / 2).clamp(MIN_RAYS_PER_TRACE_TILE, MAX_RAYS_PER_TRACE_TILE))
        .unwrap_or(DEFAULT_RAYS_PER_TRACE_TILE)
}

fn probe_trace_dispatch(tile_count: usize) -> [u32; 3] {
    if tile_count == 0 {
        [0; 3]
    } else {
        [1, 1, tile_count as u32]
    }
}

fn probe_trace_tile_words(snapshot: &HybridGiScenePrepareResourcesSnapshot) -> Vec<u32> {
    let mut words =
        Vec::with_capacity(snapshot.probe_trace_tiles().len() * PROBE_TRACE_TILE_WORDS_PER_RECORD);
    for &(tile_id, probe_id, trace_region_id, ray_count) in snapshot.probe_trace_tiles() {
        words.extend([tile_id, probe_id, trace_region_id, ray_count]);
    }
    words
}

#[cfg(test)]
mod tests {
    use crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest;
    use zircon_runtime::core::math::Vec3;

    use super::*;

    #[test]
    fn surface_cache_trace_tile_ray_count_scales_with_hybrid_gi_quality_budget() {
        let snapshot = HybridGiScenePrepareResourcesSnapshot::new(
            1,
            Vec::new(),
            vec![3],
            Vec::new(),
            4,
            0,
            (32, 8),
            (0, 0),
            0,
        );
        let mut inputs = HybridGiPrepareExecutionInputs::default();
        inputs.scene_card_capture_requests = vec![HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 22,
            atlas_slot_id: 3,
            capture_slot_id: 0,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
        }];

        for (quality_tracing_budget, expected_ray_count) in
            [(Some(8), 4), (Some(16), 8), (Some(32), 16), (None, 8)]
        {
            let tiles = probe_trace_tiles(&snapshot, &inputs, quality_tracing_budget);
            assert_eq!(tiles.len(), 1);
            assert_eq!(tiles[0].3, expected_ray_count);
        }
    }

    #[test]
    fn probe_trace_tile_generation_pipeline_is_device_owned_not_frame_created() {
        let frame_source = include_str!("scene_prepare_trace_tiles.rs");
        let construct_source = include_str!("../../../new/construct/construct.rs");

        let layout_factory = ["create_probe_trace_tile_generation_", "bind_group_layout"].concat();
        assert!(construct_source.contains(&layout_factory));
        let pipeline_factory = ["create_probe_trace_tile_generation_", "pipeline"].concat();
        let frame_pipeline_use = [
            "pass.set_pipeline(&resources.",
            "probe_trace_tile_generation_pipeline);",
        ]
        .concat();
        let frame_layout_factory = ["fn ", &layout_factory, "("].concat();
        let frame_pipeline_factory = ["fn ", &pipeline_factory, "("].concat();

        assert!(construct_source.contains(
            "let probe_trace_tile_generation_bind_group_layout =\n            create_probe_trace_tile_generation_bind_group_layout(device);"
        ));
        assert!(construct_source.contains(
            "let probe_trace_tile_generation_pipeline = create_probe_trace_tile_generation_pipeline("
        ));
        assert!(construct_source.contains("probe_trace_tile_generation_bind_group_layout,"));
        assert!(construct_source.contains("probe_trace_tile_generation_pipeline,"));
        assert!(frame_source.contains(&frame_pipeline_use));
        assert!(!frame_source.contains(&frame_layout_factory));
        assert!(!frame_source.contains(&frame_pipeline_factory));
    }
}
