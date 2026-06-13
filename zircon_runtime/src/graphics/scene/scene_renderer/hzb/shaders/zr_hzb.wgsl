fn zr_hzb_mip_for_radius(radius_pixels: f32, mip_count: u32) -> u32 {
    if (mip_count <= 1u) {
        return 0u;
    }
    let conservative_radius = max(radius_pixels, 1.0);
    let mip = u32(floor(log2(conservative_radius)));
    return min(mip, mip_count - 1u);
}

fn zr_hzb_load_furthest(uv: vec2<f32>, mip_level: u32) -> f32 {
    let mip_count = textureNumLevels(previous_hzb);
    let safe_mip = min(mip_level, mip_count - 1u);
    let mip_size = max(textureDimensions(previous_hzb, safe_mip), vec2<u32>(1u, 1u));
    let clamped_uv = clamp(uv, vec2<f32>(0.0, 0.0), vec2<f32>(0.999999, 0.999999));
    let coord = vec2<i32>(min(vec2<u32>(clamped_uv * vec2<f32>(mip_size)), mip_size - vec2<u32>(1u, 1u)));
    return textureLoad(previous_hzb, coord, safe_mip).r;
}
