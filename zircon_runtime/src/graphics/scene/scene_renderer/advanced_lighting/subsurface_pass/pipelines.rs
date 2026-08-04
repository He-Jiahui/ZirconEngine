use wgpu::util::DeviceExt;

use crate::core::framework::render::{
    SubsurfaceProfileData, ZR_SSS_MAX_PROFILES, resolve_subsurface_profile_table,
};
use crate::core::math::{Mat4, UVec2};
use crate::graphics::types::ViewportRenderRegion;

use super::{SSS_RECOMBINE_PIPELINE_LABEL, SSS_SCATTER_PIPELINE_LABEL, SSS_SETUP_PIPELINE_LABEL};

pub(super) const SETUP_SHADER: &str = include_str!("shaders/setup.wgsl");
pub(super) const SCATTER_SHADER: &str = include_str!("shaders/scatter.wgsl");
pub(super) const RECOMBINE_SHADER: &str = include_str!("shaders/recombine.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSubsurfaceParams {
    viewport_width: u32,
    viewport_height: u32,
    profile_count: u32,
    active_profile_mask: u32,
    inverse_view_projection: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSubsurfaceProfile {
    scatter_radius_and_world_scale: [f32; 4],
    falloff_and_profile_id: [f32; 4],
}

impl From<SubsurfaceProfileData> for GpuSubsurfaceProfile {
    fn from(profile: SubsurfaceProfileData) -> Self {
        Self {
            scatter_radius_and_world_scale: [
                profile.scatter_radius_rgb.x.max(0.001),
                profile.scatter_radius_rgb.y.max(0.001),
                profile.scatter_radius_rgb.z.max(0.001),
                profile.world_unit_scale.max(0.0),
            ],
            falloff_and_profile_id: [
                profile.falloff_rgb.x.max(0.0),
                profile.falloff_rgb.y.max(0.0),
                profile.falloff_rgb.z.max(0.0),
                profile.profile_id as f32,
            ],
        }
    }
}

pub(super) struct SubsurfacePipelines {
    target_format: wgpu::TextureFormat,
    setup_layout: wgpu::BindGroupLayout,
    setup: wgpu::ComputePipeline,
    scatter_layout: wgpu::BindGroupLayout,
    scatter: wgpu::ComputePipeline,
    recombine_layout: wgpu::BindGroupLayout,
    recombine: wgpu::RenderPipeline,
}

impl SubsurfacePipelines {
    pub(super) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let setup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.setup.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                storage_buffer_entry(2, false, wgpu::ShaderStages::COMPUTE),
                storage_buffer_entry(3, false, wgpu::ShaderStages::COMPUTE),
                uniform_entry(4, wgpu::ShaderStages::COMPUTE),
            ],
        });
        let setup = compute_pipeline(
            device,
            SSS_SETUP_PIPELINE_LABEL,
            &setup_layout,
            SETUP_SHADER,
        );

        let scatter_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.scatter.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Depth,
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    2,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                sampled_texture_entry(
                    3,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::COMPUTE,
                ),
                storage_buffer_entry(4, true, wgpu::ShaderStages::COMPUTE),
                uniform_entry(5, wgpu::ShaderStages::COMPUTE),
                uniform_entry(6, wgpu::ShaderStages::COMPUTE),
                storage_texture_entry(7),
            ],
        });
        let scatter = compute_pipeline(
            device,
            SSS_SCATTER_PIPELINE_LABEL,
            &scatter_layout,
            SCATTER_SHADER,
        );

        let recombine_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sss.recombine.bind-group-layout"),
            entries: &[
                sampled_texture_entry(
                    0,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                sampled_texture_entry(
                    1,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
                sampled_texture_entry(
                    2,
                    wgpu::TextureSampleType::Float { filterable: false },
                    wgpu::ShaderStages::FRAGMENT,
                ),
            ],
        });
        let recombine = render_pipeline(device, target_format, &recombine_layout, RECOMBINE_SHADER);

        Self {
            target_format,
            setup_layout,
            setup,
            scatter_layout,
            scatter,
            recombine_layout,
            recombine,
        }
    }

    pub(super) const fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_setup(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        size: UVec2,
        material: &wgpu::TextureView,
        normal: &wgpu::TextureView,
        tile_list: &wgpu::Buffer,
        indirect_args: &wgpu::Buffer,
        profile_count: u32,
        active_profile_mask: u32,
        inverse_view_projection: Mat4,
        dispatch: [u32; 3],
    ) {
        queue.write_buffer(indirect_args, 0, bytemuck::cast_slice(&[0_u32, 1, 1, 0]));
        let params = params_buffer(
            device,
            size,
            profile_count,
            active_profile_mask,
            inverse_view_projection,
            "sss.setup.params",
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sss.setup.bind-group"),
            layout: &self.setup_layout,
            entries: &[
                texture_entry(0, material),
                texture_entry(1, normal),
                buffer_entry(2, tile_list),
                buffer_entry(3, indirect_args),
                buffer_entry(4, &params),
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(SSS_SETUP_PIPELINE_LABEL),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.setup);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(dispatch[0], dispatch[1], dispatch[2]);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_scatter(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: UVec2,
        profiles: &[SubsurfaceProfileData],
        inverse_view_projection: Mat4,
        diffuse: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        material: &wgpu::TextureView,
        normal: &wgpu::TextureView,
        tile_list: &wgpu::Buffer,
        indirect_args: &wgpu::Buffer,
        scattered: &wgpu::TextureView,
    ) -> Result<(), String> {
        let table = resolve_subsurface_profile_table(profiles);
        if table.profiles.is_empty() {
            return Err(
                "sss.scatter requires at least one resolved subsurface profile".to_string(),
            );
        }
        let mut gpu_profiles = [GpuSubsurfaceProfile::default(); ZR_SSS_MAX_PROFILES];
        for (output, profile) in gpu_profiles.iter_mut().zip(table.profiles.iter().copied()) {
            *output = profile.into();
        }
        let profile_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sss.scatter.profiles"),
            contents: bytemuck::cast_slice(&gpu_profiles),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let params = params_buffer(
            device,
            size,
            table.profiles.len() as u32,
            table.active_profile_mask,
            inverse_view_projection,
            "sss.scatter.params",
        );
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sss.scatter.bind-group"),
            layout: &self.scatter_layout,
            entries: &[
                texture_entry(0, diffuse),
                texture_entry(1, depth),
                texture_entry(2, material),
                texture_entry(3, normal),
                buffer_entry(4, tile_list),
                buffer_entry(5, &profile_buffer),
                buffer_entry(6, &params),
                texture_entry(7, scattered),
            ],
        });
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(SSS_SCATTER_PIPELINE_LABEL),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.scatter);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups_indirect(indirect_args, 0);
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode_recombine(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        render_region: ViewportRenderRegion,
        scattered: &wgpu::TextureView,
        specular: &wgpu::TextureView,
        material: &wgpu::TextureView,
        output: &wgpu::TextureView,
    ) {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sss.recombine.bind-group"),
            layout: &self.recombine_layout,
            entries: &[
                texture_entry(0, scattered),
                texture_entry(1, specular),
                texture_entry(2, material),
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if !render_region.apply_physical_to_render_pass(&mut pass) {
            return;
        }
        pass.set_pipeline(&self.recombine);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn params_buffer(
    device: &wgpu::Device,
    size: UVec2,
    profile_count: u32,
    active_profile_mask: u32,
    inverse_view_projection: Mat4,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(&GpuSubsurfaceParams {
            viewport_width: size.x.max(1),
            viewport_height: size.y.max(1),
            profile_count,
            active_profile_mask,
            inverse_view_projection: inverse_view_projection.to_cols_array_2d(),
        }),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

fn compute_pipeline(
    device: &wgpu::Device,
    label: &'static str,
    bind_group_layout: &wgpu::BindGroupLayout,
    source: &'static str,
) -> wgpu::ComputePipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(label),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&layout),
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn render_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    source: &'static str,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(SSS_RECOMBINE_PIPELINE_LABEL),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn sampled_texture_entry(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
            sample_type,
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn storage_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::Rgba16Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        },
        count: None,
    }
}

fn storage_buffer_entry(
    binding: u32,
    read_only: bool,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn texture_entry(binding: u32, view: &wgpu::TextureView) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(view),
    }
}

fn buffer_entry(binding: u32, buffer: &wgpu::Buffer) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: buffer.as_entire_binding(),
    }
}
