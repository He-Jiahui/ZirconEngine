use crate::core::framework::render::{
    RenderMaterialPropertyUniformPayload, RenderMaterialTextureTransform,
    StandardPbrMaterialFeatures, SHADING_MODEL_GBUFFER_ALPHA_SCALE,
    STANDARD_MATERIAL_MIN_ROUGHNESS, STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
    STANDARD_PBR_DEFAULT_IOR, STANDARD_PBR_NO_ATTENUATION_DISTANCE,
};
use crate::graphics::scene::resources::MaterialRuntime;
use wgpu::util::DeviceExt;

/// Shared group-2 material binding floor for Standard-PBR's `data0..data15` ABI.
/// Custom property payloads are zero-padded to this size because they use the same layout.
pub(crate) const GPU_MATERIAL_UNIFORM_MIN_SIZE: usize = 256;

pub(crate) struct GpuMaterialUniformResource {
    pub(in crate::graphics::scene::resources) buffer: wgpu::Buffer,
    pub(crate) payload_byte_len: u64,
    pub(crate) buffer_byte_len: u64,
}

impl GpuMaterialUniformResource {
    pub(crate) const RETAINED_MATERIAL_UNIFORM_OWNER_COUNT: usize = 3;

    pub(crate) fn retained_material_uniform_owner_count(&self) -> usize {
        let _retained_material_uniform_owners =
            (&self.buffer, &self.payload_byte_len, &self.buffer_byte_len);
        Self::RETAINED_MATERIAL_UNIFORM_OWNER_COUNT
    }

    pub(crate) fn binding_resource(&self) -> wgpu::BindingResource<'_> {
        debug_assert_eq!(
            self.retained_material_uniform_owner_count(),
            Self::RETAINED_MATERIAL_UNIFORM_OWNER_COUNT,
            "GpuMaterialUniformResource must retain buffer and byte-length diagnostics while exposing uniform bindings",
        );
        self.buffer.as_entire_binding()
    }

    pub(crate) fn payload_byte_len(&self) -> u64 {
        self.payload_byte_len
    }

    pub(crate) fn buffer_byte_len(&self) -> u64 {
        self.buffer_byte_len
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

    // 材质绑定共享至少 256 字节的 ABI 下限；属性原始长度与补零后的缓冲长度分别保留，供诊断区分有效载荷与实际分配。
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

pub(crate) fn standard_material_uniform_contents(
    material: &MaterialRuntime,
) -> [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] {
    standard_material_uniform_contents_from_values_with_normal_details(
        material.metallic,
        material.roughness,
        material.occlusion_strength,
        material.emissive.to_array(),
        material.unlit,
        material.shading_model_id.value(),
        material.taa_reactive_mask_strength,
        material.subsurface_profile_index,
        material.alpha_cutoff,
        standard_material_texture_transforms(material),
        standard_material_texture_uv_channels(material),
        &material.advanced_features,
        material.normal_scale,
        material.clearcoat_normal_texture_transform,
        material.clearcoat_normal_texture_uv_channel,
        material.advanced_features.clearcoat_normal_scale,
    )
}

fn fallback_standard_material_uniform_contents() -> [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] {
    standard_material_uniform_contents_from_values(
        0.0,
        1.0,
        1.0,
        [0.0, 0.0, 0.0],
        false,
        2,
        0.0,
        0,
        None,
        [RenderMaterialTextureTransform::default(); STANDARD_TEXTURE_TRANSFORM_COUNT],
        [0; STANDARD_TEXTURE_TRANSFORM_COUNT],
        &StandardPbrMaterialFeatures::default(),
    )
}

fn standard_material_uniform_contents_from_values(
    metallic: f32,
    roughness: f32,
    occlusion_strength: f32,
    emissive: [f32; 3],
    unlit: bool,
    shading_model_id: u8,
    taa_reactive_mask_strength: f32,
    subsurface_profile_index: u32,
    alpha_cutoff: Option<f32>,
    texture_transforms: [RenderMaterialTextureTransform; STANDARD_TEXTURE_TRANSFORM_COUNT],
    texture_uv_channels: [u32; STANDARD_TEXTURE_TRANSFORM_COUNT],
    advanced_features: &StandardPbrMaterialFeatures,
) -> [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] {
    standard_material_uniform_contents_from_values_with_normal_scale(
        metallic,
        roughness,
        occlusion_strength,
        emissive,
        unlit,
        shading_model_id,
        taa_reactive_mask_strength,
        subsurface_profile_index,
        alpha_cutoff,
        texture_transforms,
        texture_uv_channels,
        advanced_features,
        1.0,
    )
}

fn standard_material_uniform_contents_from_values_with_normal_scale(
    metallic: f32,
    roughness: f32,
    occlusion_strength: f32,
    emissive: [f32; 3],
    unlit: bool,
    shading_model_id: u8,
    taa_reactive_mask_strength: f32,
    subsurface_profile_index: u32,
    alpha_cutoff: Option<f32>,
    texture_transforms: [RenderMaterialTextureTransform; STANDARD_TEXTURE_TRANSFORM_COUNT],
    texture_uv_channels: [u32; STANDARD_TEXTURE_TRANSFORM_COUNT],
    advanced_features: &StandardPbrMaterialFeatures,
    normal_scale: f32,
) -> [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] {
    standard_material_uniform_contents_from_values_with_normal_details(
        metallic,
        roughness,
        occlusion_strength,
        emissive,
        unlit,
        shading_model_id,
        taa_reactive_mask_strength,
        subsurface_profile_index,
        alpha_cutoff,
        texture_transforms,
        texture_uv_channels,
        advanced_features,
        normal_scale,
        RenderMaterialTextureTransform::default(),
        0,
        1.0,
    )
}

#[allow(clippy::too_many_arguments)]
fn standard_material_uniform_contents_from_values_with_normal_details(
    metallic: f32,
    roughness: f32,
    occlusion_strength: f32,
    emissive: [f32; 3],
    unlit: bool,
    shading_model_id: u8,
    taa_reactive_mask_strength: f32,
    subsurface_profile_index: u32,
    alpha_cutoff: Option<f32>,
    texture_transforms: [RenderMaterialTextureTransform; STANDARD_TEXTURE_TRANSFORM_COUNT],
    texture_uv_channels: [u32; STANDARD_TEXTURE_TRANSFORM_COUNT],
    advanced_features: &StandardPbrMaterialFeatures,
    normal_scale: f32,
    clearcoat_normal_transform: RenderMaterialTextureTransform,
    clearcoat_normal_uv_channel: u32,
    clearcoat_normal_scale: f32,
) -> [u8; GPU_MATERIAL_UNIFORM_MIN_SIZE] {
    let mut values = [0.0_f32; 64];
    values[0] = finite_or(metallic, 0.0).clamp(0.0, 1.0);
    values[1] = finite_or(roughness, 1.0).clamp(STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    values[2] = finite_or(occlusion_strength, 1.0).clamp(0.0, 1.0);
    values[3] = if unlit { 1.0 } else { 0.0 };
    values[4] = finite_or(emissive[0], 0.0).max(0.0);
    values[5] = finite_or(emissive[1], 0.0).max(0.0);
    values[6] = finite_or(emissive[2], 0.0).max(0.0);
    for (slot, transform) in texture_transforms.into_iter().enumerate() {
        let offset = (2 + slot) * 4;
        values[offset..offset + 4].copy_from_slice(&transform.as_uniform_vec4());
        let rotation_offset = 52 + slot * 2;
        values[rotation_offset..rotation_offset + 2]
            .copy_from_slice(&transform.as_uniform_rotation_sin_cos());
    }
    values[7] = finite_or(clearcoat_normal_transform.offset[1], 0.0);
    values[28] = material_uv_channel_mask(texture_uv_channels, clearcoat_normal_uv_channel);
    values[29] = finite_or(clearcoat_normal_transform.scale[0], 1.0);
    values[30] = finite_or(clearcoat_normal_transform.scale[1], 1.0);
    values[31] = finite_or(clearcoat_normal_transform.offset[0], 0.0);
    values[32] = finite_or(taa_reactive_mask_strength, 0.0).clamp(0.0, 1.0);
    values[33] = f32::from(shading_model_id) / SHADING_MODEL_GBUFFER_ALPHA_SCALE;
    values[34] = material_alpha_cutoff_scalar(alpha_cutoff);
    values[35] = subsurface_profile_index.min(255) as f32 / 255.0;
    values[36] = finite_or(advanced_features.clearcoat, 0.0).clamp(0.0, 1.0);
    values[37] = finite_or(
        advanced_features.clearcoat_perceptual_roughness,
        STANDARD_PBR_DEFAULT_CLEARCOAT_ROUGHNESS,
    )
    .clamp(STANDARD_MATERIAL_MIN_ROUGHNESS, 1.0);
    values[38] = finite_or(advanced_features.anisotropy_strength, 0.0).clamp(0.0, 1.0);
    values[39] = finite_or(advanced_features.anisotropy_rotation, 0.0);
    values[40] = finite_or(advanced_features.specular_transmission, 0.0).clamp(0.0, 1.0);
    values[41] = finite_or(advanced_features.diffuse_transmission, 0.0).clamp(0.0, 1.0);
    values[42] = finite_or(advanced_features.thickness, 0.0).max(0.0);
    values[43] = finite_or(advanced_features.ior, STANDARD_PBR_DEFAULT_IOR).max(1.0);
    values[44] = finite_or(advanced_features.attenuation_color[0], 1.0).clamp(0.0, 1.0);
    values[45] = finite_or(advanced_features.attenuation_color[1], 1.0).clamp(0.0, 1.0);
    values[46] = finite_or(advanced_features.attenuation_color[2], 1.0).clamp(0.0, 1.0);
    values[47] = finite_positive_or(
        advanced_features.attenuation_distance,
        STANDARD_PBR_NO_ATTENUATION_DISTANCE,
    );
    values[48] = finite_or(normal_scale, 1.0);
    let dielectric_f0 = advanced_features.dielectric_f0();
    values[49] = dielectric_f0;
    values[50] = finite_or(clearcoat_normal_scale, 1.0);
    let [clearcoat_rotation_cos, clearcoat_rotation_sin] =
        clearcoat_normal_transform.as_uniform_rotation_sin_cos();
    values[51] = clearcoat_rotation_cos;
    values[62] = clearcoat_rotation_sin;

    let mut bytes = [0_u8; GPU_MATERIAL_UNIFORM_MIN_SIZE];
    bytes.copy_from_slice(bytemuck::cast_slice(&values));
    bytes
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

fn material_uv_channel_mask(
    channels: [u32; STANDARD_TEXTURE_TRANSFORM_COUNT],
    clearcoat_normal_channel: u32,
) -> f32 {
    let mut mask = 0_u32;
    for (slot, channel) in channels.into_iter().enumerate() {
        if channel == 1 {
            mask |= 1_u32 << slot;
        }
    }
    if clearcoat_normal_channel == 1 {
        mask |= 1_u32 << STANDARD_TEXTURE_TRANSFORM_COUNT;
    }
    mask as f32
}

fn material_alpha_cutoff_scalar(alpha_cutoff: Option<f32>) -> f32 {
    alpha_cutoff
        .map(|cutoff| finite_or(cutoff, 0.0).clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

fn finite_positive_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

#[cfg(test)]
#[path = "tests/gpu_material_uniform_resource.rs"]
mod tests;
