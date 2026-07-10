struct HybridGiTemporalResolveParams {
    viewport_and_flags: vec4<u32>,
    blend_and_rejection: vec4<f32>,
};

@group(0) @binding(0)
var<storage, read> hybrid_gi_trace_words: array<u32>;
@group(0) @binding(1) var scene_velocity_tex: texture_2d<f32>;
@group(0) @binding(2) var previous_lighting_tex: texture_2d<f32>;
@group(0) @binding(3) var previous_temporal_metadata_tex: texture_2d<f32>;
@group(0) @binding(4) var<uniform> temporal_params: HybridGiTemporalResolveParams;

const HYBRID_GI_TRACE_SCHEDULE_MAGIC: u32 = 0x48474954u;
const HYBRID_GI_HZB_TRACE_MAGIC: u32 = 0x48475a42u;
const TRACE_DEPTH_SOURCE_VALID_FLAG: u32 = 1u;
const TRACE_HZB_TILE_GRID_EXTENT: u32 = 8u;
const TRACE_HZB_TILE_WORD_OFFSET: u32 = 64u;
const TRACE_HZB_TILE_WORD_COUNT: u32 = 7u;
const TRACE_HZB_TILE_HIT_FLAG: u32 = 1u << 8u;
const TRACE_SURFACE_CACHE_HIT_FLAG: u32 = 1u << 10u;
const TRACE_VOXEL_FALLBACK_FLAG: u32 = 1u << 11u;
const TRACE_RADIANCE_VALID_FLAG: u32 = 1u << 12u;
const DEPTH_Q24_MAX: f32 = 16777215.0;
const SCENE_SIGNATURE_MASK: u32 = 1023u;
const SCENE_SIGNATURE_SCALE: f32 = 1.0 / 1023.0;
const TRACE_SOURCE_NONE: f32 = 0.0;
const TRACE_SOURCE_SURFACE_CACHE: f32 = 1.0;
const TRACE_SOURCE_VOXEL: f32 = 2.0;
const TRACE_SOURCE_SCREEN_HIT: f32 = 3.0;
const TRACE_SOURCE_DEPTH_FALLBACK: f32 = 4.0;
const RESET_CONFIDENCE: f32 = 0.25;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct CurrentGiSample {
    radiance: vec3<f32>,
    depth: f32,
    source: f32,
    signature: f32,
    valid: f32,
};

struct HybridGiTemporalResolveOutput {
    @location(0) lighting: vec4<f32>,
    @location(1) temporal_metadata: vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -3.0),
        vec2<f32>(3.0, 1.0),
        vec2<f32>(-1.0, 1.0),
    );

    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    output.uv = positions[vertex_index] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return output;
}

fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32(packed & 0xffu) / 255.0;
    let g = f32((packed >> 8u) & 0xffu) / 255.0;
    let b = f32((packed >> 16u) & 0xffu) / 255.0;
    let a = f32((packed >> 24u) & 0xffu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

fn trace_tile_coord(uv: vec2<f32>) -> vec2<u32> {
    return min(
        vec2<u32>(clamp(uv, vec2<f32>(0.0), vec2<f32>(0.999999)) *
            f32(TRACE_HZB_TILE_GRID_EXTENT)),
        vec2<u32>(TRACE_HZB_TILE_GRID_EXTENT - 1u),
    );
}

fn viewport_size() -> vec2<u32> {
    return max(temporal_params.viewport_and_flags.xy, vec2<u32>(1u));
}

fn clamp_coord(coord: vec2<i32>, size: vec2<u32>) -> vec2<i32> {
    return clamp(
        coord,
        vec2<i32>(0),
        vec2<i32>(i32(size.x) - 1, i32(size.y) - 1),
    );
}

fn luminance(rgb: vec3<f32>) -> f32 {
    return dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
}

fn current_scene_signature() -> f32 {
    return f32(hybrid_gi_trace_words[49] & SCENE_SIGNATURE_MASK) * SCENE_SIGNATURE_SCALE;
}

fn normalized_support_signature(signature: u32) -> f32 {
    return f32(signature & SCENE_SIGNATURE_MASK) * SCENE_SIGNATURE_SCALE;
}

fn invalid_current_sample() -> CurrentGiSample {
    return CurrentGiSample(
        vec3<f32>(0.0),
        1.0,
        TRACE_SOURCE_NONE,
        current_scene_signature(),
        0.0,
    );
}

fn current_gi_sample(uv: vec2<f32>) -> CurrentGiSample {
    let trace_magic = hybrid_gi_trace_words[0];
    let packet_count = hybrid_gi_trace_words[1];
    let packed_depth_rgba = hybrid_gi_trace_words[6];
    let valid_flag = hybrid_gi_trace_words[7];
    let valid_depth_source =
        trace_magic == HYBRID_GI_TRACE_SCHEDULE_MAGIC &&
        packet_count > 0u &&
        valid_flag == TRACE_DEPTH_SOURCE_VALID_FLAG;
    let valid_hzb_trace =
        trace_magic == HYBRID_GI_TRACE_SCHEDULE_MAGIC &&
        packet_count == TRACE_HZB_TILE_GRID_EXTENT * TRACE_HZB_TILE_GRID_EXTENT &&
        hybrid_gi_trace_words[10] == HYBRID_GI_HZB_TRACE_MAGIC &&
        hybrid_gi_trace_words[19] == 1u &&
        hybrid_gi_trace_words[20] == TRACE_HZB_TILE_GRID_EXTENT &&
        hybrid_gi_trace_words[21] ==
            TRACE_HZB_TILE_GRID_EXTENT * TRACE_HZB_TILE_GRID_EXTENT;
    let fallback_signature = current_scene_signature();

    if (valid_hzb_trace) {
        let tile_coord = trace_tile_coord(uv);
        let tile_index = tile_coord.y * TRACE_HZB_TILE_GRID_EXTENT + tile_coord.x;
        let tile_word_offset =
            TRACE_HZB_TILE_WORD_OFFSET + tile_index * TRACE_HZB_TILE_WORD_COUNT;
        let packed_radiance = hybrid_gi_trace_words[tile_word_offset];
        let depth = clamp(
            f32(hybrid_gi_trace_words[tile_word_offset + 1u]) / DEPTH_Q24_MAX,
            0.0,
            1.0,
        );
        let tile_flags = hybrid_gi_trace_words[tile_word_offset + 3u];
        let support_signature = hybrid_gi_trace_words[tile_word_offset + 6u];
        let signature = select(
            fallback_signature,
            normalized_support_signature(support_signature),
            support_signature != 0u,
        );
        if ((tile_flags & TRACE_RADIANCE_VALID_FLAG) != 0u) {
            let radiance = unpack_rgba8(packed_radiance).rgb;
            if ((tile_flags & TRACE_SURFACE_CACHE_HIT_FLAG) != 0u) {
                return CurrentGiSample(
                    radiance,
                    depth,
                    TRACE_SOURCE_SURFACE_CACHE,
                    signature,
                    1.0,
                );
            }
            if ((tile_flags & TRACE_VOXEL_FALLBACK_FLAG) != 0u) {
                return CurrentGiSample(
                    radiance * 0.8,
                    depth,
                    TRACE_SOURCE_VOXEL,
                    signature,
                    1.0,
                );
            }
        }
        if ((tile_flags & TRACE_HZB_TILE_HIT_FLAG) != 0u) {
            return CurrentGiSample(
                vec3<f32>(0.08, 0.01, 0.0),
                depth,
                TRACE_SOURCE_SCREEN_HIT,
                signature,
                1.0,
            );
        }
        return CurrentGiSample(
            vec3<f32>(0.015, 0.0, 0.0),
            depth,
            TRACE_SOURCE_NONE,
            signature,
            0.0,
        );
    }

    if (valid_depth_source) {
        return CurrentGiSample(
            unpack_rgba8(packed_depth_rgba).rgb * 0.25,
            clamp(f32(hybrid_gi_trace_words[4]) / DEPTH_Q24_MAX, 0.0, 1.0),
            TRACE_SOURCE_DEPTH_FALLBACK,
            fallback_signature,
            1.0,
        );
    }
    return invalid_current_sample();
}

fn reproject_history_pixel(coord: vec2<u32>, velocity: vec2<f32>, size: vec2<u32>) -> vec2<f32> {
    return (vec2<f32>(coord) + vec2<f32>(0.5)) - velocity * vec2<f32>(size);
}

fn history_pixel_is_inside(history_pixel: vec2<f32>, size: vec2<u32>) -> bool {
    return history_pixel.x >= 0.5 &&
        history_pixel.y >= 0.5 &&
        history_pixel.x < f32(size.x) - 0.5 &&
        history_pixel.y < f32(size.y) - 0.5;
}

fn temporal_history_weight(
    current: CurrentGiSample,
    history: vec4<f32>,
    history_metadata: vec4<f32>,
    history_pixel: vec2<f32>,
    velocity: vec2<f32>,
    size: vec2<u32>,
) -> f32 {
    if (temporal_params.viewport_and_flags.z == 0u ||
        current.valid == 0.0 ||
        !history_pixel_is_inside(history_pixel, size)) {
        return 0.0;
    }
    let depth_matches =
        abs(history_metadata.x - current.depth) <= temporal_params.blend_and_rejection.z;
    let source_matches = abs(history_metadata.y - current.source) < 0.25;
    let signature_matches = abs(history_metadata.z - current.signature) < 0.00075;
    if (!depth_matches || !source_matches || !signature_matches) {
        return 0.0;
    }

    let motion_acceptance = clamp(
        1.0 - length(velocity) * temporal_params.blend_and_rejection.y,
        0.0,
        1.0,
    );
    let luma_delta = abs(luminance(current.radiance) - luminance(history.rgb));
    let luma_threshold = max(temporal_params.blend_and_rejection.w, 0.000001);
    let lighting_change_rejection = smoothstep(
        luma_threshold,
        luma_threshold * 4.0,
        luma_delta,
    );
    return clamp(
        temporal_params.blend_and_rejection.x *
            motion_acceptance *
            (1.0 - lighting_change_rejection) *
            clamp(history_metadata.w, 0.0, 1.0),
        0.0,
        0.95,
    );
}

fn next_history_confidence(previous_confidence: f32, weight: f32) -> f32 {
    if (weight <= 0.0) {
        return RESET_CONFIDENCE;
    }
    return clamp(previous_confidence + 0.125, RESET_CONFIDENCE, 1.0);
}

@fragment
fn fs_main(input: VertexOutput) -> HybridGiTemporalResolveOutput {
    let size = viewport_size();
    let coord = min(
        vec2<u32>(u32(input.position.x), u32(input.position.y)),
        size - vec2<u32>(1u),
    );
    let current = current_gi_sample(input.uv);
    let velocity = textureLoad(scene_velocity_tex, vec2<i32>(coord), 0).xy;
    let history_pixel = reproject_history_pixel(coord, velocity, size);
    let history_coord = clamp_coord(
        vec2<i32>(round(history_pixel - vec2<f32>(0.5))),
        size,
    );
    let history = textureLoad(previous_lighting_tex, history_coord, 0);
    let history_metadata = textureLoad(previous_temporal_metadata_tex, history_coord, 0);
    let weight = temporal_history_weight(
        current,
        history,
        history_metadata,
        history_pixel,
        velocity,
        size,
    );
    let clamped_history = clamp(
        history.rgb,
        max(current.radiance - vec3<f32>(0.25), vec3<f32>(0.0)),
        current.radiance + vec3<f32>(0.25),
    );
    let resolved = mix(current.radiance, clamped_history, weight);
    let confidence = next_history_confidence(history_metadata.w, weight);

    var output: HybridGiTemporalResolveOutput;
    output.lighting = vec4<f32>(resolved, 1.0);
    output.temporal_metadata = vec4<f32>(
        current.depth,
        current.source,
        current.signature,
        confidence,
    );
    return output;
}
