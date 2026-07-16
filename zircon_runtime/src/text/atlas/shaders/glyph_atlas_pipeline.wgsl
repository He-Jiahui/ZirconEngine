@group(0) @binding(0) var glyph_atlas: texture_2d_array<f32>;
@group(0) @binding(1) var glyph_atlas_sampler: sampler;

struct GlyphAtlasVertexIn {
    @location(0) position_ndc: vec2<f32>,
    @location(1) uv: vec2<f32>,
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

@vertex
fn vs_main(input: GlyphAtlasVertexIn) -> GlyphAtlasVertexOut {
    var out: GlyphAtlasVertexOut;
    out.position = vec4<f32>(input.position_ndc, 0.0, 1.0);
    out.uv = input.uv;
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
