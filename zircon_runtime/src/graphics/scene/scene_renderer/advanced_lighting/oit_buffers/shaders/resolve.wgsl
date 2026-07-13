struct OitSettings {
    viewport_width: u32,
    viewport_height: u32,
    viewport_origin_x: u32,
    viewport_origin_y: u32,
    fragments_per_pixel: u32,
    sorted_fragment_max_count: u32,
    alpha_threshold: f32,
    _padding: u32,
}

@group(0) @binding(0) var<storage, read> oit_layers: array<vec2<u32>>;
@group(0) @binding(1) var<storage, read> oit_counts: array<u32>;
@group(0) @binding(2) var<uniform> oit_settings: OitSettings;

const OIT_MAX_SORTED_FRAGMENTS: u32 = 32u;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn blend_front_to_back(accumulated: vec4<f32>, color: vec4<f32>) -> vec4<f32> {
    let alpha = clamp(color.a, 0.0, 1.0);
    let remaining = 1.0 - accumulated.a;
    return vec4<f32>(
        accumulated.rgb + remaining * clamp(color.rgb, vec3<f32>(0.0), vec3<f32>(1.0)) * alpha,
        accumulated.a + remaining * alpha,
    );
}

@fragment
fn fs_main(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let physical_pixel = vec2<u32>(position.xy);
    let origin = vec2<u32>(oit_settings.viewport_origin_x, oit_settings.viewport_origin_y);
    if (any(physical_pixel < origin)) {
        discard;
    }
    let pixel = physical_pixel - origin;
    if (pixel.x >= oit_settings.viewport_width || pixel.y >= oit_settings.viewport_height) {
        discard;
    }
    let pixel_index = pixel.y * oit_settings.viewport_width + pixel.x;
    let fragment_count = min(oit_counts[pixel_index], oit_settings.fragments_per_pixel);
    if (fragment_count == 0u) {
        discard;
    }

    let sorted_limit = min(oit_settings.sorted_fragment_max_count, OIT_MAX_SORTED_FRAGMENTS);
    var sorted_layers: array<vec2<u32>, 32>;
    var sorted_count = 0u;
    var overflow = vec4<f32>(0.0);
    let base = pixel_index * oit_settings.fragments_per_pixel;
    for (var i = 0u; i < fragment_count; i += 1u) {
        let candidate = oit_layers[base + i];
        if (sorted_count < sorted_limit) {
            var j = sorted_count;
            while (j > 0u && candidate.y < sorted_layers[j - 1u].y) {
                sorted_layers[j] = sorted_layers[j - 1u];
                j -= 1u;
            }
            sorted_layers[j] = candidate;
            sorted_count += 1u;
        } else if (sorted_limit > 0u && candidate.y < sorted_layers[sorted_limit - 1u].y) {
            overflow = blend_front_to_back(
                overflow,
                unpack4x8unorm(sorted_layers[sorted_limit - 1u].x),
            );
            var j = sorted_limit - 1u;
            while (j > 0u && candidate.y < sorted_layers[j - 1u].y) {
                sorted_layers[j] = sorted_layers[j - 1u];
                j -= 1u;
            }
            sorted_layers[j] = candidate;
        } else {
            overflow = blend_front_to_back(overflow, unpack4x8unorm(candidate.x));
        }
    }

    var color = vec4<f32>(0.0);
    for (var i = 0u; i < sorted_count; i += 1u) {
        color = blend_front_to_back(color, unpack4x8unorm(sorted_layers[i].x));
    }
    color = blend_front_to_back(color, overflow);
    return color;
}
