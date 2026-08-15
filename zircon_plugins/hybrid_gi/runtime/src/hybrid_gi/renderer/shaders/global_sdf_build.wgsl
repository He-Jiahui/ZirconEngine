const PAGE_CELLS_PER_EDGE: u32 = 8u;
const PAGE_VOXEL_COUNT: u32 = 512u;
const MAX_PAGE_CANDIDATES: u32 = 32u;
const MAX_OBJECT_PAYLOADS: u32 = 8u;

struct DispatchParams {
    page_count: u32,
    object_count: u32,
    payload_count: u32,
    candidate_count: u32,
}

struct PageDescriptor {
    world_min_and_cell_size: vec4<f32>,
    atlas_slot: u32,
    candidate_offset: u32,
    candidate_count: u32,
    padding: u32,
}

struct ObjectDescriptor {
    world_min_and_mode: vec4<f32>,
    world_max_and_padding: vec4<f32>,
    payload_offset: u32,
    payload_count: u32,
    padding0: u32,
    padding1: u32,
}

struct MeshPayload {
    local_min_and_distance_min: vec4<f32>,
    local_max_and_distance_max: vec4<f32>,
    dimensions_and_voxel_offset: vec4<u32>,
    world_to_local: mat4x4<f32>,
    distance_scale_and_padding: vec4<f32>,
}

@group(0) @binding(0) var<uniform> params: DispatchParams;
@group(0) @binding(1) var<storage, read> pages: array<PageDescriptor>;
@group(0) @binding(2) var<storage, read> objects: array<ObjectDescriptor>;
@group(0) @binding(3) var<storage, read> payloads: array<MeshPayload>;
@group(0) @binding(4) var<storage, read> mesh_voxels: array<u32>;
@group(0) @binding(5) var<storage, read> page_candidates: array<u32>;
@group(0) @binding(6) var<storage, read_write> page_atlas: array<u32>;
@group(0) @binding(7) var<storage, read_write> page_completions: array<u32>;

fn voxel_distance(payload: MeshPayload, coordinate: vec3<u32>) -> f32 {
    let dimensions = payload.dimensions_and_voxel_offset.xyz;
    let index = payload.dimensions_and_voxel_offset.w
        + coordinate.x
        + dimensions.x * (coordinate.y + dimensions.y * coordinate.z);
    let encoded = bitcast<i32>(mesh_voxels[index]);
    return clamp(f32(encoded) / 32767.0, -1.0, 1.0)
        * payload.local_max_and_distance_max.w;
}

fn sample_mesh_payload(payload: MeshPayload, world_position: vec3<f32>) -> f32 {
    let local_position = (payload.world_to_local * vec4<f32>(world_position, 1.0)).xyz;
    let local_min = payload.local_min_and_distance_min.xyz;
    let local_max = payload.local_max_and_distance_max.xyz;
    let clamped_position = clamp(local_position, local_min, local_max);
    let outside_distance = length(local_position - clamped_position);
    let dimensions = payload.dimensions_and_voxel_offset.xyz;
    let grid = (clamped_position - local_min) / max(local_max - local_min, vec3<f32>(1.0e-6))
        * vec3<f32>(dimensions) - vec3<f32>(0.5);
    let base = vec3<i32>(floor(grid));
    let fraction = fract(grid);
    var samples = array<f32, 8>();
    for (var corner = 0u; corner < 8u; corner += 1u) {
        let offset = vec3<i32>(i32(corner & 1u), i32((corner >> 1u) & 1u), i32((corner >> 2u) & 1u));
        let coordinate = vec3<u32>(clamp(base + offset, vec3<i32>(0), vec3<i32>(dimensions) - vec3<i32>(1)));
        samples[corner] = voxel_distance(payload, coordinate);
    }
    let x00 = mix(samples[0], samples[1], fraction.x);
    let x10 = mix(samples[2], samples[3], fraction.x);
    let x01 = mix(samples[4], samples[5], fraction.x);
    let x11 = mix(samples[6], samples[7], fraction.x);
    let local_distance = mix(mix(x00, x10, fraction.y), mix(x01, x11, fraction.y), fraction.z)
        + outside_distance;
    return local_distance * payload.distance_scale_and_padding.x;
}

fn sample_object(object: ObjectDescriptor, world_position: vec3<f32>) -> f32 {
    if (object.world_min_and_mode.w < 0.5 || object.payload_count == 0u) {
        return 1.0e20;
    }
    var distance = 1.0e20;
    let payload_count = min(object.payload_count, MAX_OBJECT_PAYLOADS);
    for (var payload_index = 0u; payload_index < payload_count; payload_index += 1u) {
        let source_index = object.payload_offset + payload_index;
        if (source_index < params.payload_count) {
            distance = min(distance, sample_mesh_payload(payloads[source_index], world_position));
        }
    }
    return distance;
}

@compute @workgroup_size(64, 1, 1)
fn cs_build_global_sdf(@builtin(global_invocation_id) id: vec3<u32>) {
    let invocation = id.x;
    let page_index = invocation / PAGE_VOXEL_COUNT;
    if (page_index >= params.page_count) {
        return;
    }
    let cell_index = invocation % PAGE_VOXEL_COUNT;
    let page = pages[page_index];
    let cell_coordinate = vec3<u32>(
        cell_index % PAGE_CELLS_PER_EDGE,
        (cell_index / PAGE_CELLS_PER_EDGE) % PAGE_CELLS_PER_EDGE,
        cell_index / (PAGE_CELLS_PER_EDGE * PAGE_CELLS_PER_EDGE),
    );
    let world_position = page.world_min_and_cell_size.xyz
        + (vec3<f32>(cell_coordinate) + vec3<f32>(0.5)) * page.world_min_and_cell_size.w;
    var distance = page.world_min_and_cell_size.w * f32(PAGE_CELLS_PER_EDGE);
    let candidate_count = min(page.candidate_count, MAX_PAGE_CANDIDATES);
    for (var candidate = 0u; candidate < candidate_count; candidate += 1u) {
        let candidate_index = page.candidate_offset + candidate;
        if (candidate_index >= params.candidate_count) {
            break;
        }
        let object_index = page_candidates[candidate_index];
        if (object_index < params.object_count) {
            distance = min(distance, sample_object(objects[object_index], world_position));
        }
    }
    page_atlas[page.atlas_slot * PAGE_VOXEL_COUNT + cell_index] = bitcast<u32>(distance);
    if (cell_index == 0u) {
        page_completions[page_index] = 1u;
    }
}
