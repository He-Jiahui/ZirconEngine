@group(0) @binding(0) var glyph_atlas: texture_2d_array<f32>;
@group(0) @binding(1) var glyph_atlas_sampler: sampler;
struct GlyphAtlasViewport {
    size_px: vec2<f32>,
    _padding: vec2<f32>,
};
@group(0) @binding(2) var<uniform> glyph_atlas_viewport: GlyphAtlasViewport;

struct GlyphAtlasVertexIn {
    @location(0) screen_rect_px: vec4<f32>,
    @location(1) uv_rect: vec4<f32>,
    @location(2) foreground_color: vec4<f32>,
    @location(3) background_color: vec4<f32>,
    @location(4) page_index: u32,
};

struct GlyphAtlasVertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) foreground_color: vec4<f32>,
    @location(2) background_color: vec4<f32>,
    @location(3) @interpolate(flat) page_index: u32,
};

fn glyph_atlas_quad_corner(vertex_index: u32) -> vec2<f32> {
    switch vertex_index {
        case 0u, 3u: { return vec2<f32>(0.0, 0.0); }
        case 1u: { return vec2<f32>(1.0, 0.0); }
        case 2u, 4u: { return vec2<f32>(1.0, 1.0); }
        default: { return vec2<f32>(0.0, 1.0); }
    }
}

@vertex
fn vs_main(input: GlyphAtlasVertexIn, @builtin(vertex_index) vertex_index: u32) -> GlyphAtlasVertexOut {
    var out: GlyphAtlasVertexOut;
    let corner = glyph_atlas_quad_corner(vertex_index);
    let position_px = input.screen_rect_px.xy + corner * input.screen_rect_px.zw;
    let viewport = max(glyph_atlas_viewport.size_px, vec2<f32>(1.0, 1.0));
    out.position = vec4<f32>(
        (position_px.x / viewport.x) * 2.0 - 1.0,
        1.0 - (position_px.y / viewport.y) * 2.0,
        0.0,
        1.0,
    );
    out.uv = mix(input.uv_rect.xy, input.uv_rect.zw, corner);
    out.foreground_color = input.foreground_color;
    out.background_color = input.background_color;
    out.page_index = input.page_index;
    return out;
}

fn glyph_atlas_sample(input: GlyphAtlasVertexOut) -> vec4<f32> {
    return textureSample(glyph_atlas, glyph_atlas_sampler, input.uv, i32(input.page_index));
}

fn glyph_atlas_colors(input: GlyphAtlasVertexOut) -> GlyphAtlasTextColors {
    return GlyphAtlasTextColors(input.foreground_color, input.background_color);
}

@fragment
fn fs_alpha_coverage(input: GlyphAtlasVertexOut) -> @location(0) vec4<f32> {
    return glyph_atlas_decode_alpha_coverage(glyph_atlas_sample(input), glyph_atlas_colors(input));
}

@fragment
fn fs_subpixel_rgb_coverage(input: GlyphAtlasVertexOut) -> @location(0) vec4<f32> {
    return glyph_atlas_decode_subpixel_rgb_coverage(glyph_atlas_sample(input), glyph_atlas_colors(input));
}

@fragment
fn fs_signed_distance_coverage(input: GlyphAtlasVertexOut) -> @location(0) vec4<f32> {
    return glyph_atlas_decode_signed_distance_coverage(glyph_atlas_sample(input), glyph_atlas_colors(input));
}

@fragment
fn fs_multi_channel_signed_distance_coverage(input: GlyphAtlasVertexOut) -> @location(0) vec4<f32> {
    return glyph_atlas_decode_multi_channel_signed_distance_coverage(glyph_atlas_sample(input), glyph_atlas_colors(input));
}

@fragment
fn fs_color_rgba(input: GlyphAtlasVertexOut) -> @location(0) vec4<f32> {
    return glyph_atlas_decode_color_rgba(glyph_atlas_sample(input), glyph_atlas_colors(input));
}
