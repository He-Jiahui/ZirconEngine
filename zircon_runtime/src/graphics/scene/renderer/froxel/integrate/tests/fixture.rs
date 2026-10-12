use wgpu::util::DeviceExt;

use crate::core::framework::render::{GpuLightData, GpuLightType};
use crate::core::math::{Mat4, UVec2, Vec3, Vec4};
use crate::graphics::scene::scene_renderer::lighting::light_grid_builder::{
    build_light_grid, LightGridProjection, LightGridViewInfo,
};
use crate::graphics::scene::scene_renderer::shadow::atlas::SHADOW_ATLAS_COMPARE_FUNCTION;
use crate::graphics::scene::scene_renderer::shadow::slot::{
    GpuShadowGlobals, GpuShadowSlot, GPU_SHADOW_SLOT_FLAG_VALID,
};

use super::super::FroxelViewReconstruction;

pub(super) const TEST_GRID: [u32; 3] = [16, 8, 8];
pub(super) const TEST_OUTPUT: [u32; 2] = [16, 8];
pub(super) const READBACK_BYTES_PER_ROW: u32 = 256;
pub(super) const TEST_SHADOW_OCCLUDER_DEPTH: f32 = 0.25;
pub(super) const TEST_SHADOWED_RECEIVER_DEPTH: f32 = 0.5;

pub(super) struct LightingResources {
    pub(super) light_buffer: wgpu::Buffer,
    pub(super) params_buffer: wgpu::Buffer,
    pub(super) zbins_buffer: wgpu::Buffer,
    pub(super) tile_masks_buffer: wgpu::Buffer,
}

pub(super) fn test_froxel_view() -> FroxelViewReconstruction {
    FroxelViewReconstruction::perspective(
        Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 0.1, 20.0).inverse(),
        Vec3::ZERO,
        Vec3::NEG_Z,
    )
}

pub(super) fn create_lighting_resources(device: &wgpu::Device) -> LightingResources {
    let light = GpuLightData {
        color_intensity: [1.0, 0.82, 0.45, 80.0],
        direction_type: [0.0, 0.0, -1.0, GpuLightType::Directional.as_f32_bits()],
        shadow_slot_layer: [0, u32::MAX, 1, 1],
        shadow_params: [1.0, 0.0, 0.0, 1.0],
        cookie_misc: [0, 0, 1, 0],
        ..GpuLightData::default()
    };
    let projection = Mat4::perspective_rh(90.0_f32.to_radians(), 2.0, 0.1, 20.0);
    let light_grid = build_light_grid(
        &[light],
        &LightGridViewInfo {
            viewport_size: UVec2::from_array(TEST_OUTPUT),
            world_to_view: Mat4::IDENTITY,
            view_to_clip: projection,
            projection: LightGridProjection::Perspective,
            z_near: 0.1,
            z_far: 20.0,
        },
    );
    assert_eq!(light_grid.stats.light_count, 1);
    assert_eq!(light_grid.stats.non_empty_tile_count, 2);
    assert_eq!(
        light_grid.stats.non_empty_zbin_count,
        light_grid.stats.zbin_count
    );
    let light_buffer = create_buffer(
        device,
        "volumetric-chain-lights",
        &[light],
        wgpu::BufferUsages::STORAGE,
    );
    let params_buffer = create_buffer(
        device,
        "volumetric-chain-light-grid-params",
        &[light_grid.params],
        wgpu::BufferUsages::UNIFORM,
    );
    let zbins_buffer = create_buffer(
        device,
        "volumetric-chain-light-zbins",
        &light_grid.zbins,
        wgpu::BufferUsages::STORAGE,
    );
    let tile_masks_buffer = create_buffer(
        device,
        "volumetric-chain-light-tile-masks",
        &light_grid.tile_masks,
        wgpu::BufferUsages::STORAGE,
    );
    LightingResources {
        light_buffer,
        params_buffer,
        zbins_buffer,
        tile_masks_buffer,
    }
}

pub(super) fn create_shadow_resources(
    device: &wgpu::Device,
    receiver_depth: f32,
) -> (
    wgpu::Texture,
    wgpu::TextureView,
    wgpu::Sampler,
    wgpu::Buffer,
    wgpu::Buffer,
) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("volumetric-chain-shadow-atlas"),
        size: wgpu::Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("volumetric-chain-shadow-sampler"),
        compare: Some(SHADOW_ATLAS_COMPARE_FUNCTION),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let left_half_projection = Mat4::from_cols(
        Vec4::new(2.0, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 1.0, 0.0, 0.0),
        Vec4::ZERO,
        Vec4::new(1.0, 0.0, receiver_depth, 1.0),
    );
    let slot = GpuShadowSlot {
        view_proj: left_half_projection.to_cols_array_2d(),
        atlas_scale_bias: [1.0, 1.0, 0.0, 0.0],
        params: [
            0.0,
            0.0,
            1.0 / 16.0,
            f32::from_bits(GPU_SHADOW_SLOT_FLAG_VALID),
        ],
    };
    let slots = create_buffer(
        device,
        "volumetric-chain-shadow-slots",
        &[slot],
        wgpu::BufferUsages::STORAGE,
    );
    let globals = create_buffer(
        device,
        "volumetric-chain-shadow-globals",
        &[GpuShadowGlobals::disabled(16, 16)],
        wgpu::BufferUsages::UNIFORM,
    );
    (texture, view, sampler, slots, globals)
}

pub(super) fn clear_shadow_atlas(encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("volumetric-chain-clear-shadow-atlas"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

pub(super) fn write_shadow_occluder_depth(
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("volumetric-chain-write-shadow-occluder-depth"),
        color_attachments: &[],
        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(TEST_SHADOW_OCCLUDER_DEPTH),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}

pub(super) fn create_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    label: &str,
    values: &[T],
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(values),
        usage,
    })
}

pub(super) fn create_rgba16f_3d_texture(device: &wgpu::Device, label: &str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: TEST_GRID[0],
            height: TEST_GRID[1],
            depth_or_array_layers: TEST_GRID[2],
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

pub(super) fn d3_view_descriptor(label: &str) -> wgpu::TextureViewDescriptor<'_> {
    wgpu::TextureViewDescriptor {
        label: Some(label),
        dimension: Some(wgpu::TextureViewDimension::D3),
        ..Default::default()
    }
}
