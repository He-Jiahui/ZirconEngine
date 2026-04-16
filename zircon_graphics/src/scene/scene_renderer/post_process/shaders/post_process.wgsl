struct PostProcessParams {
    viewport_and_clusters: vec4<u32>,
    feature_flags: vec4<u32>,
    hybrid_gi_counts: vec4<u32>,
    blends: vec4<f32>,
    grading: vec4<f32>,
    tint_and_probe: vec4<f32>,
    hybrid_gi_color_and_intensity: vec4<f32>,
    baked_color_and_intensity: vec4<f32>,
};

struct ReflectionProbe {
    screen_uv_and_radius: vec4<f32>,
    color_and_intensity: vec4<f32>,
};

struct HybridGiProbe {
    slot_and_budget: vec4<f32>,
    irradiance_and_weight: vec4<f32>,
};

@group(0) @binding(0) var scene_color_tex: texture_2d<f32>;
@group(0) @binding(1) var ambient_occlusion_tex: texture_2d<f32>;
@group(0) @binding(2) var history_scene_color_tex: texture_2d<f32>;
@group(0) @binding(3) var bloom_tex: texture_2d<f32>;
@group(0) @binding(4) var<uniform> params: PostProcessParams;
@group(0) @binding(5) var<storage, read> cluster_buffer: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read> reflection_probe_buffer: array<ReflectionProbe>;
@group(0) @binding(7) var<storage, read> hybrid_gi_probe_buffer: array<HybridGiProbe>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(3.0, 1.0)
    );
    var output: VertexOutput;
    output.clip_position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn apply_color_grading(color: vec3<f32>) -> vec3<f32> {
    let exposure = params.grading.x;
    let contrast = params.grading.y;
    let saturation = params.grading.z;
    let gamma = params.grading.w;
    var graded = color * exposure;
    let luma = dot(graded, vec3<f32>(0.2126, 0.7152, 0.0722));
    graded = mix(vec3<f32>(luma), graded, saturation);
    graded = ((graded - vec3<f32>(0.5)) * contrast) + vec3<f32>(0.5);
    graded = max(graded, vec3<f32>(0.0));
    graded = pow(graded, vec3<f32>(1.0 / max(gamma, 0.001)));
    return graded * params.tint_and_probe.xyz;
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let viewport_size = params.viewport_and_clusters.xy;
    let cluster_dims = params.viewport_and_clusters.zw;
    let coord = min(vec2<u32>(position.xy), viewport_size - vec2<u32>(1u, 1u));
    let coord_i32 = vec2<i32>(coord);
    let uv = (vec2<f32>(coord) + vec2<f32>(0.5)) / vec2<f32>(viewport_size);

    let scene_color = textureLoad(scene_color_tex, coord_i32, 0);
    var color = scene_color.rgb;

    if (params.feature_flags.x != 0u) {
        let ao = textureLoad(ambient_occlusion_tex, coord_i32, 0).r;
        let ao_factor = max(ao * ao, 0.12);
        color = color * ao_factor;
    }

    if (params.feature_flags.y != 0u) {
        let tile_size = 16u;
        let tile = min(coord / vec2<u32>(tile_size, tile_size), cluster_dims - vec2<u32>(1u, 1u));
        let cluster_index = tile.y * cluster_dims.x + tile.x;
        let cluster = cluster_buffer[cluster_index];
        color = color * (1.0 + cluster.a * params.blends.y);
        color = color + cluster.rgb * cluster.a * params.blends.z;
    }

    if (params.feature_flags.z != 0u) {
        let history = textureLoad(history_scene_color_tex, coord_i32, 0).rgb;
        color = mix(color, history, params.blends.x);
    }

    if (params.blends.w > 0.0) {
        let bloom = textureLoad(bloom_tex, coord_i32, 0).rgb;
        color = color + bloom * params.blends.w;
    }

    if (params.feature_flags.w > 0u) {
        for (var probe_index = 0u; probe_index < params.feature_flags.w; probe_index = probe_index + 1u) {
            let probe = reflection_probe_buffer[probe_index];
            let probe_uv = probe.screen_uv_and_radius.xy;
            let radius = max(probe.screen_uv_and_radius.z, 0.0001);
            let distance = distance(uv, probe_uv);
            let falloff = max(1.0 - distance / radius, 0.0);
            let influence = falloff * falloff * probe.color_and_intensity.w;
            color = color + probe.color_and_intensity.rgb * influence * params.tint_and_probe.w;
        }
    }

    if (params.hybrid_gi_counts.x > 0u && params.hybrid_gi_color_and_intensity.w > 0.0) {
        var gi_light = vec3<f32>(0.0);
        for (var probe_index = 0u; probe_index < params.hybrid_gi_counts.x; probe_index = probe_index + 1u) {
            let probe = hybrid_gi_probe_buffer[probe_index];
            let slot = probe.slot_and_budget.x;
            let slot_phase = fract((slot + 1.0) * 0.173 + uv.x * 0.37 + uv.y * 0.29);
            let probe_weight = mix(0.65, 1.0, slot_phase) * probe.irradiance_and_weight.w;
            gi_light = gi_light + probe.irradiance_and_weight.rgb * probe_weight;
        }

        let trace_boost = 1.0 + f32(params.hybrid_gi_counts.y) * 0.15;
        let probe_count = max(f32(params.hybrid_gi_counts.x), 1.0);
        let indirect_light =
            (gi_light / probe_count)
            * params.hybrid_gi_color_and_intensity.w
            * trace_boost;
        color = color + indirect_light;
    }

    if (params.baked_color_and_intensity.w > 0.0) {
        color = color + params.baked_color_and_intensity.rgb * params.baked_color_and_intensity.w;
    }

    color = apply_color_grading(color);
    return vec4<f32>(color, scene_color.a);
}
