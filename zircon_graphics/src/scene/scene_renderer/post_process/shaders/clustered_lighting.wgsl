struct ClusterParams {
    viewport_and_clusters: vec4<u32>,
    counts: vec4<u32>,
    strengths: vec4<f32>,
};

struct ClusterLight {
    direction: vec4<f32>,
    color_intensity: vec4<f32>,
};

@group(0) @binding(0) var<uniform> params: ClusterParams;
@group(0) @binding(1) var<storage, read> lights: array<ClusterLight>;
@group(0) @binding(2) var<storage, read_write> cluster_buffer: array<vec4<f32>>;

@compute @workgroup_size(8, 8, 1)
fn cs_main(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let cluster_dims = params.viewport_and_clusters.zw;
    if (invocation_id.x >= cluster_dims.x || invocation_id.y >= cluster_dims.y) {
        return;
    }

    let cluster_index = invocation_id.y * cluster_dims.x + invocation_id.x;
    let light_count = params.counts.x;
    if (light_count == 0u) {
        cluster_buffer[cluster_index] = vec4<f32>(0.0, 0.0, 0.0, 0.0);
        return;
    }

    let cluster_uv =
        (vec2<f32>(invocation_id.xy) + vec2<f32>(0.5, 0.5)) / max(vec2<f32>(cluster_dims), vec2<f32>(1.0, 1.0));
    let screen = cluster_uv * 2.0 - vec2<f32>(1.0, 1.0);
    var accumulated = vec3<f32>(0.0, 0.0, 0.0);
    var total_weight = 0.0;

    for (var i = 0u; i < light_count; i = i + 1u) {
        let light = lights[i];
        let light_dir = normalize(-light.direction.xyz);
        let surface_dir = normalize(vec3<f32>(screen.x, -screen.y, 1.0));
        let response = clamp(0.35 + 0.65 * dot(surface_dir, light_dir), 0.0, 1.0);
        let edge = clamp(1.0 - length(screen) * 0.65, 0.25, 1.0);
        let weight = response * edge;
        accumulated = accumulated + light.color_intensity.rgb * light.color_intensity.a * weight;
        total_weight = total_weight + weight;
    }

    let normalized = select(
        vec3<f32>(0.0, 0.0, 0.0),
        accumulated / max(total_weight, 0.001),
        total_weight > 0.0,
    );
    let intensity = clamp(length(normalized) * params.strengths.x, 0.0, 1.0);
    cluster_buffer[cluster_index] = vec4<f32>(normalized, intensity);
}
