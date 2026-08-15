use std::collections::HashMap;
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};

use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfPageBuildRequest, HybridGiGlobalSdfSceneState, HybridGiMeshSdfObject,
    GLOBAL_SDF_MAX_PAGE_CANDIDATES, GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT,
};

pub(super) const GLOBAL_SDF_PAGE_CELLS_PER_EDGE: usize = 8;
pub(super) const GLOBAL_SDF_PAGE_VOXEL_COUNT: usize = GLOBAL_SDF_PAGE_CELLS_PER_EDGE
    * GLOBAL_SDF_PAGE_CELLS_PER_EDGE
    * GLOBAL_SDF_PAGE_CELLS_PER_EDGE;
const GLOBAL_SDF_MAX_OBJECT_PAYLOADS: usize = 8;
const GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS: usize = 4 * 1024 * 1024;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GlobalSdfGpuDispatchParams {
    pub(super) page_count: u32,
    pub(super) object_count: u32,
    pub(super) payload_count: u32,
    pub(super) candidate_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GlobalSdfGpuPage {
    pub(super) world_min_and_cell_size: [f32; 4],
    pub(super) atlas_slot: u32,
    pub(super) candidate_offset: u32,
    pub(super) candidate_count: u32,
    pub(super) _padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GlobalSdfGpuObject {
    pub(super) world_min_and_mode: [f32; 4],
    pub(super) world_max_and_padding: [f32; 4],
    pub(super) payload_offset: u32,
    pub(super) payload_count: u32,
    pub(super) _padding: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GlobalSdfGpuMeshPayload {
    pub(super) local_min_and_distance_min: [f32; 4],
    pub(super) local_max_and_distance_max: [f32; 4],
    pub(super) dimensions_and_voxel_offset: [u32; 4],
    pub(super) world_to_local: [[f32; 4]; 4],
    pub(super) distance_scale_and_padding: [f32; 4],
}

pub(super) struct GlobalSdfGpuBuildInputs {
    pub(super) requests: Vec<HybridGiGlobalSdfPageBuildRequest>,
    pub(super) params: GlobalSdfGpuDispatchParams,
    pub(super) pages: Vec<GlobalSdfGpuPage>,
    pub(super) objects: Vec<GlobalSdfGpuObject>,
    pub(super) payloads: Vec<GlobalSdfGpuMeshPayload>,
    pub(super) voxels: Vec<u32>,
    pub(super) candidates: Vec<u32>,
    pub(super) dispositions: Vec<GlobalSdfPageBuildDisposition>,
    pub(super) stats: GlobalSdfGpuBuildStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi::renderer) struct GlobalSdfGpuBuildStats {
    pub(in crate::hybrid_gi::renderer) dispatched_page_count: usize,
    pub(in crate::hybrid_gi::renderer) deferred_page_count: usize,
    pub(in crate::hybrid_gi::renderer) candidate_overflow_page_count: usize,
    /// Buffers created for the encoded page build: params, page, object, payload, voxel,
    /// candidates, and completion.
    pub(in crate::hybrid_gi::renderer) transient_buffer_creation_count: usize,
    /// One build bind group is created for every encoded page-build batch.
    pub(in crate::hybrid_gi::renderer) transient_bind_group_creation_count: usize,
    pub(in crate::hybrid_gi::renderer) transient_parameter_upload_byte_count: u64,
    pub(in crate::hybrid_gi::renderer) transient_page_upload_byte_count: u64,
    pub(in crate::hybrid_gi::renderer) transient_mesh_upload_byte_count: u64,
    pub(in crate::hybrid_gi::renderer) transient_completion_upload_byte_count: u64,
    pub(in crate::hybrid_gi::renderer) transient_upload_byte_count: u64,
}

impl GlobalSdfGpuBuildStats {
    pub(in crate::hybrid_gi::renderer) fn deferred_by_readback_backpressure(
        request_count: usize,
    ) -> Self {
        Self {
            deferred_page_count: request_count,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GlobalSdfPageBuildDispositionKind {
    Build,
    TerminalFallback,
    DeferredPageBudget,
    DeferredUploadBudget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct GlobalSdfPageBuildDisposition {
    pub(super) request: HybridGiGlobalSdfPageBuildRequest,
    pub(super) kind: GlobalSdfPageBuildDispositionKind,
}

pub(super) fn pack_global_sdf_build_inputs(
    scene: &HybridGiGlobalSdfSceneState,
    objects: &[HybridGiMeshSdfObject],
    requests: &[HybridGiGlobalSdfPageBuildRequest],
    page_budget: usize,
) -> GlobalSdfGpuBuildInputs {
    let object_indices_by_stable_key = objects
        .iter()
        .enumerate()
        .map(|(index, object)| (object.stable_instance_key(), index))
        .collect::<HashMap<_, _>>();
    let mut ready_voxel_counts = HashMap::<usize, Option<usize>>::new();
    let mut dispositions = Vec::with_capacity(requests.len());
    let mut complete_pages = Vec::<(HybridGiGlobalSdfPageBuildRequest, Vec<usize>)>::new();
    let mut candidate_overflow_page_count = 0;
    for request in requests.iter().copied() {
        if request.atlas_slot() >= GLOBAL_SDF_MAX_RESIDENT_PAGE_COUNT as u32 {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
            continue;
        }
        if scene.clipmap_uses_voxel_fallback(request.key().clipmap_id()) {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
            continue;
        }
        if scene.page_has_candidate_overflow(request.key()) {
            candidate_overflow_page_count = candidate_overflow_page_count.saturating_add(1);
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
            continue;
        }
        let Some(candidate_keys) = scene.page_candidate_keys(request.key()) else {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
            continue;
        };
        let mut page_candidates = Vec::with_capacity(candidate_keys.len());
        let mut complete = true;
        for stable_instance_key in candidate_keys {
            let Some(&object_index) = object_indices_by_stable_key.get(stable_instance_key) else {
                complete = false;
                break;
            };
            let object = &objects[object_index];
            if !object.participates_in_global_sdf() {
                complete = false;
                break;
            }
            // Keep non-ready contributors in the page relation. Dropping one here
            // would make the remaining objects produce a sampleable page that
            // incorrectly bypasses the contributor's typed voxel fallback.
            let ready_voxel_count = *ready_voxel_counts
                .entry(object_index)
                .or_insert_with(|| ready_object_voxel_count(object));
            if ready_voxel_count.is_none() {
                complete = false;
                break;
            }
            page_candidates.push(object_index);
        }
        if complete && !page_candidates.is_empty() {
            complete_pages.push((request, page_candidates));
        } else {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
        }
    }

    let mut selected_pages = Vec::new();
    let mut selected_objects = vec![false; objects.len()];
    let mut selected_voxel_count = 0_usize;
    for (request, page_candidates) in complete_pages {
        let page_voxel_count = page_candidates
            .iter()
            .copied()
            .filter_map(|index| ready_voxel_counts[index])
            .sum::<usize>();
        if page_voxel_count > GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::TerminalFallback,
            });
            continue;
        }
        if selected_pages.len() >= page_budget {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::DeferredPageBudget,
            });
            continue;
        }
        let additional_voxels = page_candidates
            .iter()
            .copied()
            .filter(|index| !selected_objects[*index])
            .filter_map(|index| ready_voxel_counts[index])
            .sum::<usize>();
        if selected_voxel_count.saturating_add(additional_voxels)
            > GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS
        {
            dispositions.push(GlobalSdfPageBuildDisposition {
                request,
                kind: GlobalSdfPageBuildDispositionKind::DeferredUploadBudget,
            });
            continue;
        }
        selected_voxel_count += additional_voxels;
        for object_index in page_candidates.iter().copied() {
            selected_objects[object_index] = true;
        }
        let disposition_index = dispositions.len();
        dispositions.push(GlobalSdfPageBuildDisposition {
            request,
            kind: GlobalSdfPageBuildDispositionKind::Build,
        });
        selected_pages.push((request, page_candidates, disposition_index));
    }

    let mut gpu_object_indices = vec![None; objects.len()];
    let mut gpu_objects = Vec::new();
    let mut payloads = Vec::new();
    let mut voxels = Vec::new();
    for (object_index, selected) in selected_objects.into_iter().enumerate() {
        if !selected {
            continue;
        }
        let object = &objects[object_index];
        let gpu_index = gpu_objects.len() as u32;
        if let Some(gpu_object) = pack_ready_object(object, &mut payloads, &mut voxels) {
            gpu_object_indices[object_index] = Some(gpu_index);
            gpu_objects.push(gpu_object);
        }
    }

    let mut packed_requests = Vec::with_capacity(selected_pages.len());
    let mut pages = Vec::with_capacity(selected_pages.len());
    let mut candidates = Vec::new();
    for (request, page_object_indices, disposition_index) in selected_pages {
        let Some(bounds) = scene.page_bounds(request.key()) else {
            dispositions[disposition_index].kind =
                GlobalSdfPageBuildDispositionKind::TerminalFallback;
            continue;
        };
        let expected_candidate_count = page_object_indices.len();
        let gpu_candidates = page_object_indices
            .into_iter()
            .filter_map(|index| gpu_object_indices[index])
            .collect::<Vec<_>>();
        if gpu_candidates.len() != expected_candidate_count {
            dispositions[disposition_index].kind =
                GlobalSdfPageBuildDispositionKind::TerminalFallback;
            continue;
        }
        packed_requests.push(request);
        let candidate_offset = candidates.len() as u32;
        candidates.extend(gpu_candidates);
        let page_extent = bounds.max[0] - bounds.min[0];
        pages.push(GlobalSdfGpuPage {
            world_min_and_cell_size: [
                bounds.min[0],
                bounds.min[1],
                bounds.min[2],
                page_extent / GLOBAL_SDF_PAGE_CELLS_PER_EDGE as f32,
            ],
            atlas_slot: request.atlas_slot(),
            candidate_offset,
            candidate_count: candidates.len() as u32 - candidate_offset,
            _padding: 0,
        });
    }

    let deferred_page_count = dispositions
        .iter()
        .filter(|disposition| {
            matches!(
                disposition.kind,
                GlobalSdfPageBuildDispositionKind::DeferredPageBudget
                    | GlobalSdfPageBuildDispositionKind::DeferredUploadBudget
            )
        })
        .count();
    let transient_resource_stats = global_sdf_transient_resource_stats(
        pages.len(),
        gpu_objects.len(),
        payloads.len(),
        voxels.len(),
        candidates.len(),
    );
    let dispatched_page_count = pages.len();

    GlobalSdfGpuBuildInputs {
        params: GlobalSdfGpuDispatchParams {
            page_count: pages.len() as u32,
            object_count: gpu_objects.len() as u32,
            payload_count: payloads.len() as u32,
            candidate_count: candidates.len() as u32,
        },
        requests: packed_requests,
        pages,
        objects: gpu_objects,
        payloads,
        voxels,
        candidates,
        dispositions,
        stats: GlobalSdfGpuBuildStats {
            dispatched_page_count,
            deferred_page_count,
            candidate_overflow_page_count,
            ..transient_resource_stats
        },
    }
}

fn global_sdf_transient_resource_stats(
    page_count: usize,
    object_count: usize,
    payload_count: usize,
    voxel_count: usize,
    candidate_count: usize,
) -> GlobalSdfGpuBuildStats {
    if page_count == 0 {
        return GlobalSdfGpuBuildStats::default();
    }
    let transient_parameter_upload_byte_count = size_of::<GlobalSdfGpuDispatchParams>() as u64;
    let transient_page_upload_byte_count =
        (page_count * size_of::<GlobalSdfGpuPage>() + candidate_count * size_of::<u32>()) as u64;
    let transient_mesh_upload_byte_count = (object_count.max(1) * size_of::<GlobalSdfGpuObject>()
        + payload_count.max(1) * size_of::<GlobalSdfGpuMeshPayload>()
        + voxel_count * size_of::<u32>()) as u64;
    let transient_completion_upload_byte_count = (page_count * size_of::<u32>()) as u64;
    GlobalSdfGpuBuildStats {
        transient_buffer_creation_count: 7,
        transient_bind_group_creation_count: 1,
        transient_parameter_upload_byte_count,
        transient_page_upload_byte_count,
        transient_mesh_upload_byte_count,
        transient_completion_upload_byte_count,
        transient_upload_byte_count: transient_parameter_upload_byte_count
            .saturating_add(transient_page_upload_byte_count)
            .saturating_add(transient_mesh_upload_byte_count)
            .saturating_add(transient_completion_upload_byte_count),
        ..GlobalSdfGpuBuildStats::default()
    }
}

fn ready_object_voxel_count(object: &HybridGiMeshSdfObject) -> Option<usize> {
    if !object.distance_scale().is_finite() || object.distance_scale() <= 0.0 {
        return None;
    }
    let ready = object.asset_state().ready_payloads()?;
    if ready.is_empty() || ready.len() > GLOBAL_SDF_MAX_OBJECT_PAYLOADS {
        return None;
    }
    let voxel_count = ready.iter().try_fold(0_usize, |total, payload| {
        total.checked_add(payload.voxels.len())
    })?;
    (voxel_count <= GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS).then_some(voxel_count)
}

fn pack_ready_object(
    object: &HybridGiMeshSdfObject,
    payloads: &mut Vec<GlobalSdfGpuMeshPayload>,
    voxels: &mut Vec<u32>,
) -> Option<GlobalSdfGpuObject> {
    let payload_offset = payloads.len();
    let ready = object.asset_state().ready_payloads()?;
    let required_voxels = ready_object_voxel_count(object)?;
    if object.distance_scale() <= 0.0
        || voxels.len().saturating_add(required_voxels) > GLOBAL_SDF_MAX_UPLOAD_VOXEL_WORDS
    {
        return None;
    }
    for payload in ready {
        let voxel_offset = voxels.len() as u32;
        voxels.extend(payload.voxels.iter().map(|value| i32::from(*value) as u32));
        payloads.push(GlobalSdfGpuMeshPayload {
            local_min_and_distance_min: [
                payload.local_bounds.min[0],
                payload.local_bounds.min[1],
                payload.local_bounds.min[2],
                payload.distance_range[0],
            ],
            local_max_and_distance_max: [
                payload.local_bounds.max[0],
                payload.local_bounds.max[1],
                payload.local_bounds.max[2],
                payload.distance_range[1],
            ],
            dimensions_and_voxel_offset: [
                payload.dimensions[0],
                payload.dimensions[1],
                payload.dimensions[2],
                voxel_offset,
            ],
            world_to_local: object.world_to_local(),
            distance_scale_and_padding: [object.distance_scale(), 0.0, 0.0, 0.0],
        });
    }
    let payload_count = payloads.len() - payload_offset;
    let bounds = object.bounds();
    Some(GlobalSdfGpuObject {
        world_min_and_mode: [bounds.min[0], bounds.min[1], bounds.min[2], 1.0],
        world_max_and_padding: [bounds.max[0], bounds.max[1], bounds.max[2], 0.0],
        payload_offset: payload_offset as u32,
        payload_count: payload_count as u32,
        _padding: [0; 2],
    })
}

#[cfg(test)]
mod tests;
