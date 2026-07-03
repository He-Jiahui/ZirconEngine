use wgpu::util::DeviceExt;

use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::graphics::text::font::FontDatabase;

use super::render::ScreenSpaceUiTextBatch;
use super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;
use super::sdf_atlas::{
    sdf_atlas_layer_count, SdfAtlasAllocationFailureReason, SdfAtlasCacheReport, SdfAtlasPlan,
};
use super::sdf_char_run::sdf_scalar_is_invisible_format;
use super::sdf_font_bake::{SdfAtlasBakeReport, SdfFontBakeCache};
use super::sdf_upload::{sdf_atlas_upload_commands, sdf_atlas_upload_report, SdfAtlasUploadReport};

mod vertices;

use self::vertices::{build_sdf_vertices, ScreenSpaceUiSdfVertex};

const SDF_TEXT_SHADER: &str = include_str!("shaders/sdf_text.wgsl");
const SDF_ATLAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

pub(super) struct ScreenSpaceUiSdfRenderer {
    font_bake: SdfFontBakeCache,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    atlas_texture: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    atlas_size: UVec2,
    atlas_page_count: u32,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
    last_report: ScreenSpaceUiSdfPrepareReport,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiSdfPrepareReport {
    pub(super) text_batch_count: usize,
    pub(super) atlas_slot_count: usize,
    pub(super) atlas_size: UVec2,
    pub(super) atlas_page_count: u32,
    pub(super) atlas_allocation_failure_count: usize,
    pub(super) atlas_page_limit_failure_count: usize,
    pub(super) atlas_oversized_failure_count: usize,
    pub(super) atlas_resized: bool,
    pub(super) bake: SdfAtlasBakeReport,
    pub(super) atlas_upload_byte_len: usize,
    pub(super) atlas_upload_full_texture: bool,
    pub(super) atlas_upload: SdfAtlasUploadReport,
    pub(super) vertex_count: u32,
}

impl ScreenSpaceUiSdfRenderer {
    pub(super) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-screen-space-ui-sdf-bind-group-layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-screen-space-ui-sdf-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-screen-space-ui-sdf-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-screen-space-ui-sdf-shader"),
            source: wgpu::ShaderSource::Wgsl(SDF_TEXT_SHADER.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-screen-space-ui-sdf-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ScreenSpaceUiSdfVertex::layout()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let atlas_size = UVec2::new(1, 1);
        let atlas_page_count = 1;
        let (atlas_texture, atlas_view, bind_group) = create_atlas_resources(
            device,
            &bind_group_layout,
            &sampler,
            atlas_size,
            atlas_page_count,
        );

        Self {
            font_bake: SdfFontBakeCache::new(),
            pipeline,
            bind_group_layout,
            sampler,
            atlas_texture,
            atlas_view,
            bind_group,
            atlas_size,
            atlas_page_count,
            vertex_buffer: None,
            vertex_count: 0,
            last_report: ScreenSpaceUiSdfPrepareReport::default(),
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        atlas_plan: &SdfAtlasPlan,
        atlas_cache: SdfAtlasCacheReport,
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) {
        let atlas_page_count = sdf_atlas_layer_count(atlas_plan);
        let atlas_resized =
            atlas_plan.atlas_size != self.atlas_size || atlas_page_count != self.atlas_page_count;
        if atlas_resized {
            let (atlas_texture, atlas_view, bind_group) = create_atlas_resources(
                device,
                &self.bind_group_layout,
                &self.sampler,
                atlas_plan.atlas_size,
                atlas_page_count,
            );
            self.atlas_texture = atlas_texture;
            self.atlas_view = atlas_view;
            self.bind_group = bind_group;
            self.atlas_size = atlas_plan.atlas_size;
            self.atlas_page_count = atlas_page_count;
        }

        let atlas_bake = self
            .font_bake
            .build_atlas(atlas_plan, font_database, asset_manager);
        let atlas_upload = sdf_atlas_upload_report(
            atlas_plan,
            atlas_cache,
            atlas_resized,
            atlas_bake.pixels.len(),
            atlas_resized,
        );
        write_sdf_atlas_texture(
            queue,
            &self.atlas_texture,
            atlas_plan,
            &atlas_bake.pixels,
            &atlas_upload,
        );

        let vertices = build_sdf_vertices(
            texts,
            atlas_plan,
            &atlas_bake,
            &mut self.font_bake,
            font_database,
            asset_manager,
            viewport_size,
        );
        self.vertex_count = vertices.len() as u32;
        self.vertex_buffer = (!vertices.is_empty()).then(|| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-screen-space-ui-sdf-vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });
        self.last_report = sdf_prepare_report(
            texts.len(),
            atlas_plan,
            atlas_resized,
            atlas_page_count,
            atlas_bake.report,
            atlas_upload,
            self.vertex_count,
        );
    }

    pub(super) fn prepare_report(&self) -> ScreenSpaceUiSdfPrepareReport {
        self.last_report.clone()
    }

    pub(super) fn measure_text_glyph_advances_for_fallbacks(
        &mut self,
        texts: &[ScreenSpaceUiTextBatch],
        font_database: &mut FontDatabase,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|text| {
                if let Some(advances) = resolved_layout_advances_for_sdf_glyphs(
                    text.text.as_str(),
                    text.glyph_advances.as_slice(),
                    text.text.chars().count(),
                ) {
                    return advances;
                }
                text.text
                    .chars()
                    .map(|glyph| {
                        if sdf_scalar_is_invisible_format(glyph) {
                            return 0.0;
                        }
                        self.font_bake
                            .measure_glyph(
                                glyph,
                                text.font.as_deref(),
                                text.font_family.as_deref(),
                                text.font_weight,
                                text.font_size,
                                font_database,
                                asset_manager,
                            )
                            .advance
                    })
                    .collect()
            })
            .collect()
    }

    pub(super) fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        let Some(vertex_buffer) = self.vertex_buffer.as_ref() else {
            return;
        };
        if self.vertex_count == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

fn sdf_prepare_report(
    text_batch_count: usize,
    atlas_plan: &SdfAtlasPlan,
    atlas_resized: bool,
    atlas_page_count: u32,
    bake: SdfAtlasBakeReport,
    atlas_upload: SdfAtlasUploadReport,
    vertex_count: u32,
) -> ScreenSpaceUiSdfPrepareReport {
    ScreenSpaceUiSdfPrepareReport {
        text_batch_count,
        atlas_slot_count: atlas_plan.slots.len(),
        atlas_size: atlas_plan.atlas_size,
        atlas_page_count,
        atlas_allocation_failure_count: atlas_plan.allocation_failures.len(),
        atlas_page_limit_failure_count: atlas_plan
            .allocation_failures
            .iter()
            .filter(|failure| failure.reason == SdfAtlasAllocationFailureReason::PageLimit)
            .count(),
        atlas_oversized_failure_count: atlas_plan
            .allocation_failures
            .iter()
            .filter(|failure| failure.reason == SdfAtlasAllocationFailureReason::OversizedSlot)
            .count(),
        atlas_resized,
        bake,
        atlas_upload_byte_len: atlas_upload.byte_len,
        atlas_upload_full_texture: atlas_upload.full_texture,
        atlas_upload,
        vertex_count,
    }
}

fn write_sdf_atlas_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    atlas_plan: &SdfAtlasPlan,
    pixels: &[u8],
    upload: &SdfAtlasUploadReport,
) {
    if pixels.is_empty() {
        return;
    }
    for command in sdf_atlas_upload_commands(atlas_plan, upload.clone(), pixels.len()) {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: command.rect.x,
                    y: command.rect.y,
                    z: command.page_key.page_index,
                },
                aspect: wgpu::TextureAspect::All,
            },
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: command.source_offset,
                bytes_per_row: Some(command.bytes_per_row),
                rows_per_image: Some(command.rows_per_image),
            },
            wgpu::Extent3d {
                width: command.rect.width,
                height: command.rect.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn create_atlas_resources(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    atlas_size: UVec2,
    atlas_page_count: u32,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-screen-space-ui-sdf-atlas"),
        size: wgpu::Extent3d {
            width: atlas_size.x.max(1),
            height: atlas_size.y.max(1),
            depth_or_array_layers: atlas_page_count.max(1),
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SDF_ATLAS_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("zircon-screen-space-ui-sdf-atlas-view"),
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-screen-space-ui-sdf-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    (texture, view, bind_group)
}

#[cfg(test)]
mod tests;
