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

@group(4) @binding(0) var<storage, read_write> oit_layers: array<vec2<u32>>;
@group(4) @binding(1) var<storage, read_write> oit_counts: array<atomic<u32>>;
@group(4) @binding(2) var<uniform> oit_settings: OitSettings;

fn oit_draw(position: vec4<f32>, color: vec4<f32>) {
    if (color.a <= oit_settings.alpha_threshold) {
        return;
    }
    let physical_pixel = vec2<u32>(position.xy);
    let origin = vec2<u32>(oit_settings.viewport_origin_x, oit_settings.viewport_origin_y);
    if (any(physical_pixel < origin)) {
        return;
    }
    let pixel = physical_pixel - origin;
    if (pixel.x >= oit_settings.viewport_width || pixel.y >= oit_settings.viewport_height) {
        return;
    }
    let pixel_index = pixel.y * oit_settings.viewport_width + pixel.x;
    let slot = atomicAdd(&oit_counts[pixel_index], 1u);
    if (slot >= oit_settings.fragments_per_pixel) {
        return;
    }
    let layer_index = pixel_index * oit_settings.fragments_per_pixel + slot;
    oit_layers[layer_index] = vec2<u32>(pack4x8unorm(color), bitcast<u32>(position.z));
}
