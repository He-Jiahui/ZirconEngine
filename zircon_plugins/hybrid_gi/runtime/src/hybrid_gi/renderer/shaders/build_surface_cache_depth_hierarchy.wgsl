@group(0) @binding(0)
var parent_depth: texture_2d<f32>;

@group(0) @binding(1)
var target_depth: texture_storage_2d<rgba8unorm, write>;

fn reduce_parent_depth(parent_coord: vec2<i32>) -> vec4<f32> {
    let parent_size = vec2<i32>(textureDimensions(parent_depth));
    let clamped_coord = clamp(parent_coord, vec2<i32>(0), parent_size - vec2<i32>(1));
    return textureLoad(parent_depth, clamped_coord, 0);
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let target_size = textureDimensions(target_depth);
    if (global_id.x >= target_size.x || global_id.y >= target_size.y) {
        return;
    }

    let parent_origin = vec2<i32>(global_id.xy * 2u);
    let parent_samples = array<vec4<f32>, 4>(
        reduce_parent_depth(parent_origin),
        reduce_parent_depth(parent_origin + vec2<i32>(1, 0)),
        reduce_parent_depth(parent_origin + vec2<i32>(0, 1)),
        reduce_parent_depth(parent_origin + vec2<i32>(1, 1)),
    );
    var min_depth = 1.0;
    var max_depth = 0.0;
    var valid_count = 0u;
    for (var sample_index = 0u; sample_index < 4u; sample_index = sample_index + 1u) {
        let sample = parent_samples[sample_index];
        if (sample.a <= 0.5) {
            continue;
        }
        min_depth = min(min_depth, sample.r);
        max_depth = max(max_depth, max(sample.r, sample.g));
        valid_count = valid_count + 1u;
    }

    let target_coord = vec2<i32>(global_id.xy);
    if (valid_count == 0u) {
        textureStore(target_depth, target_coord, vec4<f32>(1.0, 1.0, 0.0, 0.0));
        return;
    }
    textureStore(
        target_depth,
        target_coord,
        vec4<f32>(min_depth, max_depth, max_depth - min_depth, 1.0),
    );
}
