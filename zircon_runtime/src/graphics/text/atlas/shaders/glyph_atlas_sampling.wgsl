struct GlyphAtlasTextColors {
    foreground: vec4<f32>,
    background: vec4<f32>,
};

fn glyph_atlas_decode_alpha_coverage(sample: vec4<f32>, colors: GlyphAtlasTextColors) -> vec4<f32> {
    let coverage = clamp(sample.r, 0.0, 1.0);
    return vec4<f32>(colors.foreground.rgb, colors.foreground.a * coverage);
}

fn glyph_atlas_decode_subpixel_rgb_coverage(sample: vec4<f32>, colors: GlyphAtlasTextColors) -> vec4<f32> {
    let coverage = clamp(sample.rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    let foreground_alpha = clamp(colors.foreground.a, 0.0, 1.0);
    let rgb = mix(colors.background.rgb, colors.foreground.rgb, coverage * foreground_alpha);
    let alpha = foreground_alpha * max(max(coverage.r, coverage.g), coverage.b);
    return vec4<f32>(rgb, alpha);
}

fn glyph_atlas_distance_coverage(distance: f32) -> f32 {
    let width = max(fwidth(distance), 0.0001);
    return smoothstep(0.5 - width, 0.5 + width, distance);
}

fn glyph_atlas_decode_signed_distance_coverage(sample: vec4<f32>, colors: GlyphAtlasTextColors) -> vec4<f32> {
    let coverage = glyph_atlas_distance_coverage(sample.r);
    return vec4<f32>(colors.foreground.rgb, colors.foreground.a * coverage);
}

fn glyph_atlas_median_rgb(value: vec3<f32>) -> f32 {
    return max(min(value.r, value.g), min(max(value.r, value.g), value.b));
}

fn glyph_atlas_decode_multi_channel_signed_distance_coverage(sample: vec4<f32>, colors: GlyphAtlasTextColors) -> vec4<f32> {
    let coverage = glyph_atlas_distance_coverage(glyph_atlas_median_rgb(sample.rgb));
    return vec4<f32>(colors.foreground.rgb, colors.foreground.a * coverage);
}

fn glyph_atlas_decode_color_rgba(sample: vec4<f32>, colors: GlyphAtlasTextColors) -> vec4<f32> {
    return sample * colors.foreground;
}
