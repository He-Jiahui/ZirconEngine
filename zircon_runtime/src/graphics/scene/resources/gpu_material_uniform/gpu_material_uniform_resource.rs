use crate::core::framework::render::{
    RenderMaterialPropertyUniformPayload, RenderMaterialTextureTransform,
};
use crate::graphics::scene::resources::MaterialRuntime;
use wgpu::util::DeviceExt;

pub(crate) const GPU_MATERIAL_UNIFORM_MIN_SIZE: usize = 144;

pub(crate) struct GpuMaterialUniformResource {
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) buffer: wgpu::Buffer,
    #[allow(dead_code)]
    pub(crate) payload_byte_len: u64,
    #[allow(dead_code)]
    pub(crate) buffer_byte_len: u64,
}

impl GpuMaterialUniformResource {
    pub(crate) fn binding_resource(&self) -> wgpu::BindingResource<'_> {
        self.buffer.as_entire_binding()
    }

    pub(crate) fn from_payload(
        device: &wgpu::Device,
        payload: &RenderMaterialPropertyUniformPayload,
    ) -> Self {
        let contents = padded_uniform_contents(payload);
        Self::from_contents(
            device,
            "zircon-material-property-uniform-buffer",
            &contents,
            payload.bytes.len() as u64,
        )
    }

    pub(crate) fn from_standard_material(
        device: &wgpu::Device,
        material: &MaterialRuntime,
    ) -> Self {
        let contents = standard_material_uniform_contents(material);
        Self::from_contents(
            device,
            "zircon-standard-material-uniform-buffer",
            &contents,
            contents.len() as u64,
        )
    }

    pub(crate) fn fallback_standard_material(device: &wgpu::Device) -> Self {
        let contents = fallback_standard_material_uniform_contents();
        Self::from_contents(
            device,
            "zircon-standard-material-fallback-uniform-buffer",
            &contents,
            contents.len() as u64,
        )
    }

    fn from_contents(
        device: &wgpu::Device,
        buffer_label: &'static str,
        contents: &[u8],
        payload_byte_len: u64,
    ) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(buffer_label),
            contents,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            buffer,
            payload_byte_len,
            buffer_byte_len: contents.len() as u64,
        }
    }
}

fn padded_uniform_contents(payload: &RenderMaterialPropertyUniformPayload) -> Vec<u8> {
    let mut contents = payload.bytes.clone();
    contents.resize(contents.len().max(GPU_MATERIAL_UNIFORM_MIN_SIZE), 0);
    contents
}

fn standard_material_uniform_contents(material: &MaterialRuntime) -> Vec<u8> {
    standard_material_uniform_contents_from_values(
        material.metallic,
        material.roughness,
        material.emissive.to_array(),
        material.unlit,
        material.taa_reactive_mask_strength,
        standard_material_texture_transforms(material),
        standard_material_texture_uv_channels(material),
    )
}

fn fallback_standard_material_uniform_contents() -> Vec<u8> {
    standard_material_uniform_contents_from_values(
        0.0,
        1.0,
        [0.0, 0.0, 0.0],
        false,
        0.0,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
    )
}

fn standard_material_uniform_contents_from_values(
    metallic: f32,
    roughness: f32,
    emissive: [f32; 3],
    unlit: bool,
    taa_reactive_mask_strength: f32,
    texture_transforms: [RenderMaterialTextureTransform; STANDARD_TEXTURE_TRANSFORM_COUNT],
    texture_uv_channels: [u32; STANDARD_TEXTURE_TRANSFORM_COUNT],
) -> Vec<u8> {
    let mut values = [0.0_f32; 36];
    values[0] = finite_or(metallic, 0.0).clamp(0.0, 1.0);
    values[1] = finite_or(roughness, 1.0).clamp(0.04, 1.0);
    values[2] = 1.0;
    values[3] = if unlit { 1.0 } else { 0.0 };
    values[4] = finite_or(emissive[0], 0.0).max(0.0);
    values[5] = finite_or(emissive[1], 0.0).max(0.0);
    values[6] = finite_or(emissive[2], 0.0).max(0.0);
    for (slot, transform) in texture_transforms.into_iter().enumerate() {
        let offset = (2 + slot) * 4;
        values[offset..offset + 4].copy_from_slice(&transform.as_uniform_vec4());
    }
    values[7] = material_uv_channel_scalar(texture_uv_channels[4]);
    values[28] = material_uv_channel_scalar(texture_uv_channels[0]);
    values[29] = material_uv_channel_scalar(texture_uv_channels[1]);
    values[30] = material_uv_channel_scalar(texture_uv_channels[2]);
    values[31] = material_uv_channel_scalar(texture_uv_channels[3]);
    values[32] = finite_or(taa_reactive_mask_strength, 0.0).clamp(0.0, 1.0);

    bytemuck::cast_slice(&values).to_vec()
}

const STANDARD_TEXTURE_TRANSFORM_COUNT: usize = 5;

fn standard_material_texture_transforms(
    material: &MaterialRuntime,
) -> [RenderMaterialTextureTransform; STANDARD_TEXTURE_TRANSFORM_COUNT] {
    [
        material.base_color_texture_transform,
        material.normal_texture_transform,
        material.metallic_roughness_texture_transform,
        material.occlusion_texture_transform,
        material.emissive_texture_transform,
    ]
}

fn standard_material_texture_uv_channels(
    material: &MaterialRuntime,
) -> [u32; STANDARD_TEXTURE_TRANSFORM_COUNT] {
    [
        material.base_color_texture_uv_channel,
        material.normal_texture_uv_channel,
        material.metallic_roughness_texture_uv_channel,
        material.occlusion_texture_uv_channel,
        material.emissive_texture_uv_channel,
    ]
}

fn material_uv_channel_scalar(channel: u32) -> f32 {
    channel.min(1) as f32
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::RenderMaterialTextureTransform;

    use super::{standard_material_uniform_contents_from_values, STANDARD_TEXTURE_TRANSFORM_COUNT};

    #[test]
    fn standard_material_uniform_packs_pbr_scalars_without_property_schema_offsets() {
        let bytes = standard_material_uniform_contents_from_values(
            1.4,
            0.0,
            [0.25, -1.0, 2.0],
            true,
            1.4,
            [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
            [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        );

        assert_eq!(bytes.len(), 144);
        assert_eq!(f32_at(&bytes, 0), 1.0);
        assert_eq!(f32_at(&bytes, 4), 0.04);
        assert_eq!(f32_at(&bytes, 8), 1.0);
        assert_eq!(f32_at(&bytes, 12), 1.0);
        assert_eq!(f32_at(&bytes, 16), 0.25);
        assert_eq!(f32_at(&bytes, 20), 0.0);
        assert_eq!(f32_at(&bytes, 24), 2.0);
        assert_eq!(f32_at(&bytes, 128), 1.0);
    }

    #[test]
    fn standard_material_uniform_packs_per_slot_texture_transforms() {
        let bytes = standard_material_uniform_contents_from_values(
            0.5,
            0.5,
            [0.0, 0.0, 0.0],
            false,
            0.25,
            [
                transform([2.0, 3.0], [0.25, 0.5]),
                transform([4.0, 5.0], [0.125, 0.25]),
                transform([6.0, 7.0], [0.75, 0.875]),
                transform([8.0, 9.0], [1.25, 1.5]),
                transform([f32::NAN, 11.0], [f32::INFINITY, -0.25]),
            ],
            [1, 0, 2, u32::MAX, 1],
        );

        assert_eq!(vec4_at(&bytes, 32), [2.0, 3.0, 0.25, 0.5]);
        assert_eq!(vec4_at(&bytes, 48), [4.0, 5.0, 0.125, 0.25]);
        assert_eq!(vec4_at(&bytes, 64), [6.0, 7.0, 0.75, 0.875]);
        assert_eq!(vec4_at(&bytes, 80), [8.0, 9.0, 1.25, 1.5]);
        assert_eq!(vec4_at(&bytes, 96), [1.0, 11.0, 0.0, -0.25]);
        assert_eq!(f32_at(&bytes, 28), 1.0);
        assert_eq!(vec4_at(&bytes, 112), [1.0, 0.0, 1.0, 1.0]);
        assert_eq!(vec4_at(&bytes, 128), [0.25, 0.0, 0.0, 0.0]);
    }

    fn transform(scale: [f32; 2], offset: [f32; 2]) -> RenderMaterialTextureTransform {
        RenderMaterialTextureTransform { scale, offset }
    }

    fn vec4_at(bytes: &[u8], offset: usize) -> [f32; 4] {
        [
            f32_at(bytes, offset),
            f32_at(bytes, offset + 4),
            f32_at(bytes, offset + 8),
            f32_at(bytes, offset + 12),
        ]
    }

    fn f32_at(bytes: &[u8], offset: usize) -> f32 {
        f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }
}
