use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::framework::render::{RenderImageColorSpace, TextureMetadata, TextureUsageHint};

use super::{MIP_GEN_MIPS_PER_DISPATCH, MipGenDispatch, MipGenDispatchPlan};

const MIP_GEN_SHADER: &str = include_str!("shaders/mip_gen.wgsl");
const MIP_GEN_STORAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const MIP_GEN_BINDING_COUNT: usize = 2 + MIP_GEN_MIPS_PER_DISPATCH as usize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MipGenColorMode {
    srgb: bool,
    normal: bool,
}

impl MipGenColorMode {
    pub(crate) fn from_metadata(metadata: &TextureMetadata) -> Self {
        Self {
            srgb: metadata.color_space == RenderImageColorSpace::Srgb,
            normal: metadata.usage_hint == TextureUsageHint::Normal,
        }
    }

    const fn as_params(self) -> (u32, u32) {
        (self.srgb as u32, self.normal as u32)
    }
}

/// Encodes a runtime mip chain after its owning texture's final producer completed.
pub(crate) struct RuntimeMipGenPass {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    _fallback_storage_textures: Vec<wgpu::Texture>,
    fallback_storage_views: Vec<wgpu::TextureView>,
}

impl RuntimeMipGenPass {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-runtime-mip-gen-layout"),
            entries: &mip_gen_bind_group_layout_entries(),
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-runtime-mip-gen-shader"),
            source: wgpu::ShaderSource::Wgsl(MIP_GEN_SHADER.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-runtime-mip-gen-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("zircon-runtime-mip-gen-pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        let (fallback_storage_textures, fallback_storage_views) =
            create_fallback_storage_targets(device);

        Self {
            bind_group_layout,
            pipeline,
            _fallback_storage_textures: fallback_storage_textures,
            fallback_storage_views,
        }
    }

    pub(crate) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        plan: &MipGenDispatchPlan,
        color_mode: MipGenColorMode,
    ) -> u32 {
        for dispatch in plan.dispatches() {
            self.encode_dispatch(device, encoder, texture, plan, dispatch, color_mode);
        }
        plan.dispatch_count()
    }

    fn encode_dispatch(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        plan: &MipGenDispatchPlan,
        dispatch: &MipGenDispatch,
        color_mode: MipGenColorMode,
    ) {
        let (is_srgb, is_normal) = color_mode.as_params();
        let params = MipGenParams {
            source_extent: [
                mip_extent(plan.texture_extent()[0], dispatch.source_mip_level()),
                mip_extent(plan.texture_extent()[1], dispatch.source_mip_level()),
            ],
            generated_mip_count: dispatch.generated_mip_count(),
            is_srgb,
            is_normal,
            _padding: [0; 3],
        };
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-runtime-mip-gen-params"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let source_view = texture.create_view(&mip_view_descriptor(
            dispatch.source_mip_level(),
            plan.array_layer_count(),
        ));
        let target_views = (0..dispatch.generated_mip_count())
            .map(|target_offset| {
                texture.create_view(&mip_view_descriptor(
                    dispatch.first_target_mip_level() + target_offset,
                    plan.array_layer_count(),
                ))
            })
            .collect::<Vec<_>>();
        let target_view = |index: usize| {
            target_views
                .get(index)
                .unwrap_or(&self.fallback_storage_views[index])
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-runtime-mip-gen-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&source_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(target_view(0)),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(target_view(1)),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(target_view(2)),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::TextureView(target_view(3)),
                },
            ],
        });
        let workgroup_count = dispatch.workgroup_count();
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("RuntimeMipGenPass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.dispatch_workgroups(workgroup_count[0], workgroup_count[1], workgroup_count[2]);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MipGenParams {
    source_extent: [u32; 2],
    generated_mip_count: u32,
    is_srgb: u32,
    is_normal: u32,
    _padding: [u32; 3],
}

fn mip_gen_bind_group_layout_entries() -> [wgpu::BindGroupLayoutEntry; MIP_GEN_BINDING_COUNT] {
    let uniform = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<MipGenParams>() as u64),
        },
        count: None,
    };
    let source = wgpu::BindGroupLayoutEntry {
        binding: 1,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2Array,
            multisampled: false,
        },
        count: None,
    };
    let storage = |binding| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: MIP_GEN_STORAGE_FORMAT,
            view_dimension: wgpu::TextureViewDimension::D2Array,
        },
        count: None,
    };
    [
        uniform,
        source,
        storage(2),
        storage(3),
        storage(4),
        storage(5),
    ]
}

fn create_fallback_storage_targets(
    device: &wgpu::Device,
) -> (Vec<wgpu::Texture>, Vec<wgpu::TextureView>) {
    let mut textures = Vec::with_capacity(MIP_GEN_MIPS_PER_DISPATCH as usize);
    let mut views = Vec::with_capacity(MIP_GEN_MIPS_PER_DISPATCH as usize);
    for _ in 0..MIP_GEN_MIPS_PER_DISPATCH {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("zircon-runtime-mip-gen-fallback-target"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: MIP_GEN_STORAGE_FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        views.push(texture.create_view(&mip_view_descriptor(0, 1)));
        textures.push(texture);
    }
    (textures, views)
}

fn mip_view_descriptor(
    mip_level: u32,
    array_layer_count: u32,
) -> wgpu::TextureViewDescriptor<'static> {
    wgpu::TextureViewDescriptor {
        format: Some(MIP_GEN_STORAGE_FORMAT),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        aspect: wgpu::TextureAspect::All,
        base_mip_level: mip_level,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(array_layer_count),
        ..Default::default()
    }
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    if level >= u32::BITS {
        1
    } else {
        let shifted = value >> level;
        if shifted == 0 { 1 } else { shifted }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_mipgen_shader_writes_up_to_four_storage_mips() {
        assert_eq!(std::mem::size_of::<MipGenParams>(), 32);
        assert!(MIP_GEN_SHADER.contains("@compute @workgroup_size(8, 8, 1)"));
        assert!(MIP_GEN_SHADER.contains("var target_mip_four"));
        assert!(MIP_GEN_SHADER.contains("generated_mip_count >= 4u"));
        assert!(MIP_GEN_SHADER.contains("var<workgroup> level_one"));
        assert!(MIP_GEN_SHADER.contains("let level_one_extent = min("));
        assert!(MIP_GEN_SHADER.contains("let level_two_extent = min("));
        assert!(MIP_GEN_SHADER.contains("let level_three_extent = min("));
    }

    #[test]
    fn runtime_mipgen_color_mode_tracks_texture_metadata() {
        let mut metadata = TextureMetadata::default();
        metadata.color_space = RenderImageColorSpace::Srgb;
        metadata.usage_hint = TextureUsageHint::Albedo;
        assert_eq!(
            MipGenColorMode::from_metadata(&metadata),
            MipGenColorMode {
                srgb: true,
                normal: false
            }
        );

        metadata.color_space = RenderImageColorSpace::Linear;
        metadata.usage_hint = TextureUsageHint::Normal;
        assert_eq!(
            MipGenColorMode::from_metadata(&metadata),
            MipGenColorMode {
                srgb: false,
                normal: true
            }
        );
    }
}
