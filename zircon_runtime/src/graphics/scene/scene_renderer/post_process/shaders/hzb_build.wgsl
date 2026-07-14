struct HzbParams {
    target_size: vec2<u32>,
    target_mip_level: u32,
    _pad0: u32,
};

@group(0) @binding(0) var<uniform> hzb_params: HzbParams;
@group(0) @binding(1) var scene_depth_tex: texture_depth_2d;
@group(0) @binding(2) var source_hzb_tex: texture_2d<f32>;
@group(0) @binding(3) var target_hzb_tex: texture_storage_2d<rgba16float, write>;

struct HzbDepthRange {
    furthest: f32,
    closest: f32,
};

fn furthest_depth(a: f32, b: f32) -> f32 {
    return max(a, b);
}

fn closest_depth(a: f32, b: f32) -> f32 {
    return min(a, b);
}

fn load_depth_range_or_far(coord: vec2<i32>, size: vec2<i32>) -> HzbDepthRange {
    if (coord.x < 0 || coord.y < 0 || coord.x >= size.x || coord.y >= size.y) {
        return HzbDepthRange(1.0, 1.0);
    }
    let depth = textureLoad(scene_depth_tex, coord, 0);
    return HzbDepthRange(depth, depth);
}

fn load_hzb_range_or_far(coord: vec2<i32>, size: vec2<i32>) -> HzbDepthRange {
    if (coord.x < 0 || coord.y < 0 || coord.x >= size.x || coord.y >= size.y) {
        return HzbDepthRange(1.0, 1.0);
    }
    let parent_range = textureLoad(source_hzb_tex, coord, 0);
    return HzbDepthRange(parent_range.x, parent_range.y);
}

fn combine_depth_ranges(a: HzbDepthRange, b: HzbDepthRange) -> HzbDepthRange {
    return HzbDepthRange(
        furthest_depth(a.furthest, b.furthest),
        closest_depth(a.closest, b.closest),
    );
}

fn reduce_depth_quad(base: vec2<i32>, size: vec2<i32>) -> HzbDepthRange {
    var depth_range = load_depth_range_or_far(base, size);
    depth_range = combine_depth_ranges(
        depth_range,
        load_depth_range_or_far(base + vec2<i32>(1, 0), size),
    );
    depth_range = combine_depth_ranges(
        depth_range,
        load_depth_range_or_far(base + vec2<i32>(0, 1), size),
    );
    return combine_depth_ranges(
        depth_range,
        load_depth_range_or_far(base + vec2<i32>(1, 1), size),
    );
}

fn reduce_hzb_quad(base: vec2<i32>, size: vec2<i32>) -> HzbDepthRange {
    var depth_range = load_hzb_range_or_far(base, size);
    depth_range = combine_depth_ranges(
        depth_range,
        load_hzb_range_or_far(base + vec2<i32>(1, 0), size),
    );
    depth_range = combine_depth_ranges(
        depth_range,
        load_hzb_range_or_far(base + vec2<i32>(0, 1), size),
    );
    return combine_depth_ranges(
        depth_range,
        load_hzb_range_or_far(base + vec2<i32>(1, 1), size),
    );
}

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let target_coord_u = global_id.xy;
    if (target_coord_u.x >= hzb_params.target_size.x ||
        target_coord_u.y >= hzb_params.target_size.y) {
        return;
    }

    let base = vec2<i32>(target_coord_u * 2u);
    var depth_range: HzbDepthRange;
    if (hzb_params.target_mip_level == 0u) {
        depth_range = reduce_depth_quad(base, vec2<i32>(textureDimensions(scene_depth_tex)));
    } else {
        depth_range = reduce_hzb_quad(base, vec2<i32>(textureDimensions(source_hzb_tex)));
    }
    textureStore(
        target_hzb_tex,
        vec2<i32>(target_coord_u),
        vec4<f32>(
            depth_range.furthest,
            depth_range.closest,
            max(0.0, depth_range.furthest - depth_range.closest),
            1.0,
        ),
    );
}
