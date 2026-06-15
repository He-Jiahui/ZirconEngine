struct SceneUniform {
    view_proj: mat4x4<f32>,
    view_proj_unjittered: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    ambient_color: vec4<f32>,
    previous_view_proj_unjittered: mat4x4<f32>,
    motion_params: vec4<f32>,
    jitter_params: vec4<f32>,
};

struct HzbOcclusionCullParams {
    counts: vec4<u32>,
    values: vec4<f32>,
};

struct IndexedIndirectArgs {
    index_count: u32,
    instance_count: u32,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
};

struct HzbOcclusionCullStats {
    tested_arg_count: atomic<u32>,
    tested_instance_count: atomic<u32>,
    culled_arg_count: atomic<u32>,
    culled_instance_count: atomic<u32>,
};

struct IndirectCompactionBatchMetadata {
    source_arg_index: u32,
    visible_instance_base: u32,
    source_first_instance: u32,
    source_instance_count: u32,
    output_arg_base: u32,
    draw_count_index: u32,
};

struct IndirectDrawCount {
    value: atomic<u32>,
};

@group(0) @binding(0) var<uniform> scene: SceneUniform;
@group(1) @binding(0) var previous_hzb: texture_2d<f32>;
@group(1) @binding(1) var<uniform> cull_params: HzbOcclusionCullParams;
@group(1) @binding(2) var<storage, read> source_indirect_args: array<IndexedIndirectArgs>;
@group(1) @binding(3) var<storage, read> compaction_metadata: array<IndirectCompactionBatchMetadata>;
@group(1) @binding(4) var<storage, read_write> visible_instance_indices: array<u32>;
@group(1) @binding(5) var<storage, read_write> draw_counts: array<IndirectDrawCount>;
@group(1) @binding(6) var<storage, read_write> compacted_indirect_args: array<IndexedIndirectArgs>;
@group(1) @binding(7) var<storage, read_write> occlusion_stats: HzbOcclusionCullStats;

const GPU_PRIMITIVE_FLAG_VISIBLE: u32 = 1u;
const GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM: u32 = 4u;
const MIN_CLIP_W: f32 = 0.0001;

fn transform_scale_radius(world_from_local: mat4x4<f32>, radius: f32) -> f32 {
    let sx = length(world_from_local[0].xyz);
    let sy = length(world_from_local[1].xyz);
    let sz = length(world_from_local[2].xyz);
    return radius * max(max(sx, sy), max(sz, 0.0001));
}

fn instance_world_from_local(instance: ZrGpuInstanceData, primitive: ZrGpuPrimitiveData) -> mat4x4<f32> {
    if ((primitive.flags & GPU_PRIMITIVE_FLAG_HAS_PREVIOUS_TRANSFORM) != 0u) {
        return instance.prev_world_from_local;
    }
    return instance.world_from_local;
}

fn instance_is_conservatively_visible(instance_index: u32) -> bool {
    let instance = zr_gpu_scene_instance(instance_index);
    let primitive = zr_gpu_scene_primitive(instance);
    if ((primitive.flags & GPU_PRIMITIVE_FLAG_VISIBLE) == 0u) {
        return false;
    }

    let world_from_local = instance_world_from_local(instance, primitive);
    let world_center = world_from_local * vec4<f32>(primitive.bounds_center, 1.0);
    let world_radius = transform_scale_radius(world_from_local, primitive.bounds_radius) *
        max(cull_params.values.y, 1.0);
    let clip_center = scene.previous_view_proj_unjittered * world_center;
    if (clip_center.w <= MIN_CLIP_W) {
        return true;
    }

    let ndc_center = clip_center.xyz / clip_center.w;
    let radius_ndc = world_radius / max(clip_center.w, MIN_CLIP_W);
    if (ndc_center.x + radius_ndc < -1.0 ||
        ndc_center.x - radius_ndc > 1.0 ||
        ndc_center.y + radius_ndc < -1.0 ||
        ndc_center.y - radius_ndc > 1.0 ||
        ndc_center.z < 0.0 ||
        ndc_center.z > 1.0) {
        return true;
    }

    let hzb_size = vec2<f32>(max(textureDimensions(previous_hzb, 0u), vec2<u32>(1u, 1u)));
    let radius_pixels = radius_ndc * max(hzb_size.x, hzb_size.y);
    let mip = zr_hzb_mip_for_radius(radius_pixels, textureNumLevels(previous_hzb));
    let uv = ndc_center.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let hzb_furthest = zr_hzb_load_furthest(uv, mip);
    let conservative_nearest_depth = ndc_center.z - radius_ndc - cull_params.values.x;

    return conservative_nearest_depth <= hzb_furthest;
}

fn compact_visible_instances(metadata: IndirectCompactionBatchMetadata) -> u32 {
    var offset = 0u;
    var visible_count = 0u;
    loop {
        if (offset >= metadata.source_instance_count) {
            break;
        }
        let source_instance = metadata.source_first_instance + offset;
        if (instance_is_conservatively_visible(source_instance)) {
            visible_instance_indices[metadata.visible_instance_base + visible_count] = source_instance;
            visible_count = visible_count + 1u;
        }
        offset = offset + 1u;
    }
    return visible_count;
}

@compute @workgroup_size(64, 1, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let arg_index = global_id.x;
    if (arg_index >= cull_params.counts.x) {
        return;
    }

    let metadata = compaction_metadata[arg_index];
    let args = source_indirect_args[metadata.source_arg_index];
    if (args.instance_count == 0u) {
        return;
    }

    atomicAdd(&occlusion_stats.tested_arg_count, 1u);
    atomicAdd(&occlusion_stats.tested_instance_count, args.instance_count);

    let visible_count = compact_visible_instances(metadata);
    let culled_instance_count = args.instance_count - visible_count;
    if (culled_instance_count > 0u) {
        atomicAdd(&occlusion_stats.culled_instance_count, culled_instance_count);
    }
    if (visible_count == 0u) {
        atomicAdd(&occlusion_stats.culled_arg_count, 1u);
        return;
    }

    let output_arg_offset = atomicAdd(&draw_counts[metadata.draw_count_index].value, 1u);
    let output_arg_index = metadata.output_arg_base + output_arg_offset;
    var compacted_args = args;
    compacted_args.instance_count = visible_count;
    compacted_args.first_instance = metadata.visible_instance_base;
    compacted_indirect_args[output_arg_index] = compacted_args;
}
