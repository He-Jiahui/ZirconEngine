use std::sync::Arc;

use crate::core::framework::render::{EnvironmentBrdfLutFormat, CANONICAL_ENVIRONMENT_PBR_RECIPE};
use crate::graphics::types::GraphicsError;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

use super::payloads::{
    black_alpha_one_rgba8_bytes, black_cube_rgba16float_bytes, black_rgba16float_bytes,
    black_rgba8_bytes, effect_lut_3d_rgba8_bytes, effect_lut_rgba8_bytes,
    irradiance_volume_black_rgba8_bytes, normal_rgba8_bytes, white_rgba8_bytes,
};

pub(super) const BLACK_CUBE_FACE_COUNT: u32 = 6;
pub(super) const EFFECT_LUT_WIDTH: u32 = 64;
pub(super) const EFFECT_LUT_3D_SIZE: u32 = 2;
pub(super) const IRRADIANCE_VOLUME_FALLBACK_HEIGHT: u32 = 2;
pub(super) const IRRADIANCE_VOLUME_FALLBACK_DEPTH: u32 = 3;
pub(super) const SYSTEM_TEXTURE_UPLOAD_COUNT: usize = 10;
pub(super) const SYSTEM_TEXTURE_UPLOAD_BYTES: u64 = 16_768;

#[derive(Clone)]
pub(super) struct SystemTextureResources {
    black_cube_texture: wgpu::Texture,
    black_cube_view: wgpu::TextureView,
    brdf_lut_texture: wgpu::Texture,
    brdf_lut_view: wgpu::TextureView,
    black_rgba8_texture: wgpu::Texture,
    black_rgba8_view: wgpu::TextureView,
    black_alpha_one_rgba8_texture: wgpu::Texture,
    black_alpha_one_rgba8_view: wgpu::TextureView,
    white_rgba8_texture: wgpu::Texture,
    white_rgba8_view: wgpu::TextureView,
    white_rgba8_srgb_view: wgpu::TextureView,
    normal_rgba8_texture: wgpu::Texture,
    normal_rgba8_view: wgpu::TextureView,
    black_rgba16float_texture: wgpu::Texture,
    black_rgba16float_view: wgpu::TextureView,
    black_rgba16float_array_view: wgpu::TextureView,
    irradiance_volume_black_texture: wgpu::Texture,
    irradiance_volume_black_view: wgpu::TextureView,
    effect_lut_texture: wgpu::Texture,
    effect_lut_view: wgpu::TextureView,
    effect_lut_3d_texture: wgpu::Texture,
    effect_lut_3d_view: wgpu::TextureView,
    linear_clamp_sampler: wgpu::Sampler,
}

pub(super) struct PreparedSystemTextureResources {
    pub(super) resources: SystemTextureResources,
    pub(super) uploads: WgpuTextureUploadBatch,
    pub(super) upload_count: usize,
    pub(super) upload_bytes: u64,
}

impl SystemTextureResources {
    pub(super) fn prepare(
        device: &wgpu::Device,
        brdf_lut_payload: Arc<[u8]>,
    ) -> Result<PreparedSystemTextureResources, GraphicsError> {
        let black_cube_texture = create_texture(
            device,
            "zircon-system-black-cube",
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: BLACK_CUBE_FACE_COUNT,
            },
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba16Float,
        );
        let black_cube_view = black_cube_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-system-black-cube-view"),
            format: Some(wgpu::TextureFormat::Rgba16Float),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: Some(1),
            base_array_layer: 0,
            array_layer_count: Some(BLACK_CUBE_FACE_COUNT),
        });
        let brdf_lut_recipe = CANONICAL_ENVIRONMENT_PBR_RECIPE.brdf_lut_recipe();
        let brdf_lut_texture = create_texture(
            device,
            "zircon-system-environment-brdf-lut",
            wgpu::Extent3d {
                width: brdf_lut_recipe.width(),
                height: brdf_lut_recipe.height(),
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D2,
            environment_brdf_lut_wgpu_format(brdf_lut_recipe.output_format()),
        );
        let brdf_lut_view = brdf_lut_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let (black_rgba8_texture, black_rgba8_view) = create_texture_and_default_view(
            device,
            "zircon-system-black-rgba8",
            one_texel_extent(),
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let (black_alpha_one_rgba8_texture, black_alpha_one_rgba8_view) =
            create_texture_and_default_view(
                device,
                "zircon-system-black-alpha-one-rgba8",
                one_texel_extent(),
                wgpu::TextureDimension::D2,
                wgpu::TextureFormat::Rgba8Unorm,
            );
        let white_rgba8_texture = create_texture_with_view_formats(
            device,
            "zircon-system-white-rgba8",
            one_texel_extent(),
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
            &[wgpu::TextureFormat::Rgba8UnormSrgb],
        );
        let white_rgba8_view = white_rgba8_texture.create_view(&Default::default());
        let white_rgba8_srgb_view = white_rgba8_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("zircon-system-white-rgba8-srgb-view"),
            format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            ..Default::default()
        });
        let (normal_rgba8_texture, normal_rgba8_view) = create_texture_and_default_view(
            device,
            "zircon-system-default-normal-rgba8",
            one_texel_extent(),
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let black_rgba16float_texture = create_texture(
            device,
            "zircon-system-black-rgba16float",
            one_texel_extent(),
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba16Float,
        );
        let black_rgba16float_view =
            black_rgba16float_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let black_rgba16float_array_view =
            black_rgba16float_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("zircon-system-black-rgba16float-array-view"),
                format: Some(wgpu::TextureFormat::Rgba16Float),
                dimension: Some(wgpu::TextureViewDimension::D2Array),
                usage: Some(wgpu::TextureUsages::TEXTURE_BINDING),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
            });
        let (irradiance_volume_black_texture, irradiance_volume_black_view) =
            create_texture_and_default_view(
                device,
                "zircon-system-irradiance-volume-black",
                wgpu::Extent3d {
                    width: 1,
                    height: IRRADIANCE_VOLUME_FALLBACK_HEIGHT,
                    depth_or_array_layers: IRRADIANCE_VOLUME_FALLBACK_DEPTH,
                },
                wgpu::TextureDimension::D3,
                wgpu::TextureFormat::Rgba8Unorm,
            );
        let (effect_lut_texture, effect_lut_view) = create_texture_and_default_view(
            device,
            "zircon-system-effect-lut",
            wgpu::Extent3d {
                width: EFFECT_LUT_WIDTH,
                height: 1,
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D2,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let (effect_lut_3d_texture, effect_lut_3d_view) = create_texture_and_default_view(
            device,
            "zircon-system-effect-lut-3d",
            wgpu::Extent3d {
                width: EFFECT_LUT_3D_SIZE,
                height: EFFECT_LUT_3D_SIZE,
                depth_or_array_layers: EFFECT_LUT_3D_SIZE,
            },
            wgpu::TextureDimension::D3,
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let linear_clamp_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-system-linear-clamp-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });

        let mut uploads = WgpuTextureUploadBatch::new();
        let mut upload_count = 0_usize;
        let mut upload_bytes = 0_u64;
        push_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &black_cube_texture,
            TextureCopyRegion::new(1, 1).with_depth_or_array_layers(BLACK_CUBE_FACE_COUNT),
            8,
            1,
            black_cube_rgba16float_bytes(),
        )?;
        push_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &brdf_lut_texture,
            TextureCopyRegion::new(brdf_lut_recipe.width(), brdf_lut_recipe.height()),
            brdf_lut_recipe.width() * brdf_lut_recipe.output_format().texel_size_bytes(),
            brdf_lut_recipe.height(),
            brdf_lut_payload,
        )?;
        push_solid_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &black_rgba8_texture,
            4,
            black_rgba8_bytes(),
        )?;
        push_solid_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &black_alpha_one_rgba8_texture,
            4,
            black_alpha_one_rgba8_bytes(),
        )?;
        push_solid_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &white_rgba8_texture,
            4,
            white_rgba8_bytes(),
        )?;
        push_solid_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &normal_rgba8_texture,
            4,
            normal_rgba8_bytes(),
        )?;
        push_solid_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &black_rgba16float_texture,
            8,
            black_rgba16float_bytes(),
        )?;
        push_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &irradiance_volume_black_texture,
            TextureCopyRegion::new(1, IRRADIANCE_VOLUME_FALLBACK_HEIGHT)
                .with_depth_or_array_layers(IRRADIANCE_VOLUME_FALLBACK_DEPTH),
            4,
            IRRADIANCE_VOLUME_FALLBACK_HEIGHT,
            irradiance_volume_black_rgba8_bytes(),
        )?;
        push_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &effect_lut_texture,
            TextureCopyRegion::new(EFFECT_LUT_WIDTH, 1),
            EFFECT_LUT_WIDTH * 4,
            1,
            effect_lut_rgba8_bytes(),
        )?;
        push_upload(
            &mut uploads,
            &mut upload_count,
            &mut upload_bytes,
            &effect_lut_3d_texture,
            TextureCopyRegion::new(EFFECT_LUT_3D_SIZE, EFFECT_LUT_3D_SIZE)
                .with_depth_or_array_layers(EFFECT_LUT_3D_SIZE),
            EFFECT_LUT_3D_SIZE * 4,
            EFFECT_LUT_3D_SIZE,
            effect_lut_3d_rgba8_bytes(),
        )?;
        debug_assert_eq!(upload_count, SYSTEM_TEXTURE_UPLOAD_COUNT);
        debug_assert_eq!(upload_bytes, SYSTEM_TEXTURE_UPLOAD_BYTES);

        Ok(PreparedSystemTextureResources {
            resources: Self {
                black_cube_texture,
                black_cube_view,
                brdf_lut_texture,
                brdf_lut_view,
                black_rgba8_texture,
                black_rgba8_view,
                black_alpha_one_rgba8_texture,
                black_alpha_one_rgba8_view,
                white_rgba8_texture,
                white_rgba8_view,
                white_rgba8_srgb_view,
                normal_rgba8_texture,
                normal_rgba8_view,
                black_rgba16float_texture,
                black_rgba16float_view,
                black_rgba16float_array_view,
                irradiance_volume_black_texture,
                irradiance_volume_black_view,
                effect_lut_texture,
                effect_lut_view,
                effect_lut_3d_texture,
                effect_lut_3d_view,
                linear_clamp_sampler,
            },
            uploads,
            upload_count,
            upload_bytes,
        })
    }

    pub(super) fn black_cube_texture(&self) -> &wgpu::Texture {
        &self.black_cube_texture
    }

    pub(super) fn black_cube_view(&self) -> &wgpu::TextureView {
        &self.black_cube_view
    }

    pub(super) fn brdf_lut_texture(&self) -> &wgpu::Texture {
        &self.brdf_lut_texture
    }

    pub(super) fn brdf_lut_view(&self) -> &wgpu::TextureView {
        &self.brdf_lut_view
    }

    pub(super) fn black_rgba8_texture(&self) -> &wgpu::Texture {
        &self.black_rgba8_texture
    }

    pub(super) fn black_rgba8_view(&self) -> &wgpu::TextureView {
        &self.black_rgba8_view
    }

    pub(super) fn black_alpha_one_rgba8_texture(&self) -> &wgpu::Texture {
        &self.black_alpha_one_rgba8_texture
    }

    pub(super) fn black_alpha_one_rgba8_view(&self) -> &wgpu::TextureView {
        &self.black_alpha_one_rgba8_view
    }

    pub(super) fn white_rgba8_texture(&self) -> &wgpu::Texture {
        &self.white_rgba8_texture
    }

    pub(super) fn white_rgba8_view(&self) -> &wgpu::TextureView {
        &self.white_rgba8_view
    }

    pub(super) fn white_rgba8_srgb_view(&self) -> &wgpu::TextureView {
        &self.white_rgba8_srgb_view
    }

    pub(super) fn normal_rgba8_texture(&self) -> &wgpu::Texture {
        &self.normal_rgba8_texture
    }

    pub(super) fn normal_rgba8_view(&self) -> &wgpu::TextureView {
        &self.normal_rgba8_view
    }

    pub(super) fn black_rgba16float_texture(&self) -> &wgpu::Texture {
        &self.black_rgba16float_texture
    }

    pub(super) fn black_rgba16float_view(&self) -> &wgpu::TextureView {
        &self.black_rgba16float_view
    }

    pub(super) fn black_rgba16float_array_view(&self) -> &wgpu::TextureView {
        &self.black_rgba16float_array_view
    }

    pub(super) fn irradiance_volume_black_texture(&self) -> &wgpu::Texture {
        &self.irradiance_volume_black_texture
    }

    pub(super) fn irradiance_volume_black_view(&self) -> &wgpu::TextureView {
        &self.irradiance_volume_black_view
    }

    pub(super) fn effect_lut_texture(&self) -> &wgpu::Texture {
        &self.effect_lut_texture
    }

    pub(super) fn effect_lut_view(&self) -> &wgpu::TextureView {
        &self.effect_lut_view
    }

    pub(super) fn effect_lut_3d_texture(&self) -> &wgpu::Texture {
        &self.effect_lut_3d_texture
    }

    pub(super) fn effect_lut_3d_view(&self) -> &wgpu::TextureView {
        &self.effect_lut_3d_view
    }

    pub(super) fn linear_clamp_sampler(&self) -> &wgpu::Sampler {
        &self.linear_clamp_sampler
    }
}

pub(super) const fn environment_brdf_lut_wgpu_format(
    format: EnvironmentBrdfLutFormat,
) -> wgpu::TextureFormat {
    match format {
        EnvironmentBrdfLutFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
    }
}

fn create_texture(
    device: &wgpu::Device,
    label: &'static str,
    size: wgpu::Extent3d,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    create_texture_with_view_formats(device, label, size, dimension, format, &[])
}

fn create_texture_with_view_formats(
    device: &wgpu::Device,
    label: &'static str,
    size: wgpu::Extent3d,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
    view_formats: &[wgpu::TextureFormat],
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats,
    })
}

const fn one_texel_extent() -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    }
}

fn create_texture_and_default_view(
    device: &wgpu::Device,
    label: &'static str,
    size: wgpu::Extent3d,
    dimension: wgpu::TextureDimension,
    format: wgpu::TextureFormat,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = create_texture(device, label, size, dimension, format);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn push_solid_upload(
    uploads: &mut WgpuTextureUploadBatch,
    upload_count: &mut usize,
    upload_bytes: &mut u64,
    texture: &wgpu::Texture,
    bytes_per_row: u32,
    payload: Arc<[u8]>,
) -> Result<(), GraphicsError> {
    push_upload(
        uploads,
        upload_count,
        upload_bytes,
        texture,
        TextureCopyRegion::new(1, 1),
        bytes_per_row,
        1,
        payload,
    )
}

#[allow(clippy::too_many_arguments)]
fn push_upload(
    uploads: &mut WgpuTextureUploadBatch,
    upload_count: &mut usize,
    upload_bytes: &mut u64,
    texture: &wgpu::Texture,
    region: TextureCopyRegion,
    bytes_per_row: u32,
    rows_per_image: u32,
    payload: Arc<[u8]>,
) -> Result<(), GraphicsError> {
    let payload_byte_len = payload.len();
    let upload = WgpuTextureUpload::new(
        texture.clone(),
        region,
        bytes_per_row,
        rows_per_image,
        payload,
        0..payload_byte_len,
    )
    .ok_or_else(|| {
        GraphicsError::WgpuValidation("system texture upload payload range is invalid".to_owned())
    })?;
    uploads.push(upload);
    *upload_count = upload_count.saturating_add(1);
    *upload_bytes = upload_bytes.saturating_add(payload_byte_len as u64);
    Ok(())
}
