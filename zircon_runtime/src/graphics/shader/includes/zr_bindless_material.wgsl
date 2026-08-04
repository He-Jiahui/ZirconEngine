// This module is injected only by the bindless material template variant. WGPU derives
// non-uniform resource indexing from `slot` dataflow after the generated header enables
// wgpu_binding_array. Shader code must sample through these wrappers instead of indexing the
// arrays directly.

@group(2) @binding(0) var zr_bindless_material_textures:
    binding_array<texture_2d<f32>, ZR_BINDLESS_MATERIAL_SLOT_CAPACITY>;
@group(2) @binding(1) var zr_bindless_material_samplers:
    binding_array<sampler, ZR_BINDLESS_MATERIAL_SLOT_CAPACITY>;

fn zr_bindless_material_slot(slot: u32) -> u32 {
    return min(slot, ZR_BINDLESS_MATERIAL_SLOT_CAPACITY - 1u);
}

fn zr_bindless_material_sample_bias(
    slot: u32,
    uv: vec2<f32>,
    mip_bias: f32,
) -> vec4<f32> {
    let sampled_slot = zr_bindless_material_slot(slot);
    return textureSampleBias(
        zr_bindless_material_textures[sampled_slot],
        zr_bindless_material_samplers[sampled_slot],
        uv,
        mip_bias,
    );
}
