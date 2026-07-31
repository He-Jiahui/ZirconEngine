use wgpu::util::DeviceExt;

use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use crate::text::TextRenderState;
use crate::text::sdf::{
    SdfAtlasBakeReport, SdfAtlasGlyphKey, SdfGlyphGenerationError, SdfShapedGlyphIdentity,
    SdfTextRun,
};

use super::render::ScreenSpaceUiTextBatch;
use super::sdf_advances::resolved_layout_advances_for_sdf_glyphs;
use super::sdf_atlas::{SdfAtlasAllocationFailureReason, SdfAtlasCacheReport, SdfAtlasPlan};
use super::sdf_upload::{SdfAtlasUploadReport, sdf_atlas_upload_report};

mod atlas_resources;
mod decorations;
mod material;
mod vertices;

use self::atlas_resources::DistanceFieldAtlasResources;
use self::decorations::build_text_decoration_vertices;
use self::material::{SdfTextMaterialDrawPlan, SdfTextMaterialResources};
use self::vertices::{ScreenSpaceUiSdfVertex, build_sdf_vertex_plan};

const SDF_TEXT_SHADER: &str = include_str!("shaders/zr_text_sdf.wgsl");

pub(super) struct ScreenSpaceUiSdfRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    atlas: DistanceFieldAtlasResources,
    material: SdfTextMaterialResources,
    draw_plan: SdfTextMaterialDrawPlan,
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
    pub(super) msdf_atlas_page_count: u32,
    pub(super) atlas_allocation_failure_count: usize,
    pub(super) atlas_page_limit_failure_count: usize,
    pub(super) atlas_oversized_failure_count: usize,
    pub(super) atlas_resized: bool,
    pub(super) bake: SdfAtlasBakeReport,
    pub(super) atlas_upload_byte_len: usize,
    pub(super) atlas_upload_full_texture: bool,
    pub(super) atlas_upload: SdfAtlasUploadReport,
    pub(super) vertex_count: u32,
    pub(super) decoration_vertex_count: u32,
    pub(super) material_count: usize,
    pub(super) draw_count: usize,
    pub(super) outline_batch_count: usize,
    pub(super) shadow_batch_count: usize,
    pub(super) glow_batch_count: usize,
}

impl SdfTextRun for ScreenSpaceUiTextBatch {
    fn font(&self) -> Option<&str> {
        self.font.as_deref()
    }

    fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }

    fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }

    fn font_weight(&self) -> u16 {
        self.font_weight
    }

    fn font_size(&self) -> f32 {
        self.font_size
    }

    fn render_scalars(&self) -> Vec<char> {
        if self.shaped_glyphs.is_empty() {
            self.text.chars().collect()
        } else {
            self.shaped_glyphs
                .iter()
                .map(|glyph| glyph.source_scalar)
                .collect()
        }
    }

    fn resolved_glyph_advances(&self) -> Option<Vec<f32>> {
        let render_scalar_count = if self.shaped_glyphs.is_empty() {
            self.text.chars().count()
        } else {
            self.shaped_glyphs.len()
        };
        resolved_layout_advances_for_sdf_glyphs(
            self.text.as_str(),
            self.glyph_advances.as_slice(),
            render_scalar_count,
        )
    }

    fn shaped_glyph(&self, glyph_index: usize) -> Option<SdfShapedGlyphIdentity> {
        self.shaped_glyphs
            .get(glyph_index)
            .map(|glyph| SdfShapedGlyphIdentity {
                glyph_id: glyph.glyph_id,
                font_id: glyph.font_id,
                font_instance_id: glyph.font_instance_id,
            })
    }
}

impl ScreenSpaceUiSdfRenderer {
    pub(super) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let material = SdfTextMaterialResources::new(device);
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let reserved_view_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-screen-space-ui-sdf-reserved-view-layout"),
                entries: &[],
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
            bind_group_layouts: &[
                Some(&bind_group_layout),
                Some(&reserved_view_layout),
                Some(&material.bind_group_layout),
            ],
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
        let atlas = DistanceFieldAtlasResources::new(
            device,
            &bind_group_layout,
            &sampler,
            UVec2::new(1, 1),
            1,
            1,
        );

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            atlas,
            material,
            draw_plan: SdfTextMaterialDrawPlan::default(),
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
        native_decoration_texts: &[ScreenSpaceUiTextBatch],
        atlas_plan: &SdfAtlasPlan,
        atlas_cache: SdfAtlasCacheReport,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) {
        let (atlas_page_count, msdf_atlas_page_count) =
            DistanceFieldAtlasResources::page_counts(atlas_plan);
        let atlas_resized = !self.atlas.matches(
            atlas_plan.atlas_size,
            atlas_page_count,
            msdf_atlas_page_count,
        );
        if atlas_resized {
            self.atlas = DistanceFieldAtlasResources::new(
                device,
                &self.bind_group_layout,
                &self.sampler,
                atlas_plan.atlas_size,
                atlas_page_count,
                msdf_atlas_page_count,
            );
        }

        let atlas_bake =
            text_state.build_sdf_atlas(atlas_plan.atlas_size, &atlas_plan.slots, asset_manager);
        let atlas_upload = sdf_atlas_upload_report(
            atlas_plan,
            atlas_cache,
            atlas_resized,
            atlas_bake.pixels.len(),
            atlas_resized,
        );
        self.atlas
            .write(queue, atlas_plan, &atlas_bake.pixels, &atlas_upload);

        let native_cpu_runs =
            text_state.prepare_sdf_runs_cpu(native_decoration_texts, asset_manager);
        let sdf_cpu_runs = text_state.prepare_sdf_runs_cpu(texts, asset_manager);
        let mut vertices = build_text_decoration_vertices(
            native_decoration_texts,
            &native_cpu_runs,
            viewport_size,
        );
        vertices.extend(build_text_decoration_vertices(
            texts,
            &sdf_cpu_runs,
            viewport_size,
        ));
        let decoration_vertex_count = vertices.len() as u32;
        let glyph_plan =
            build_sdf_vertex_plan(texts, atlas_plan, &atlas_bake, &sdf_cpu_runs, viewport_size);
        self.draw_plan = SdfTextMaterialDrawPlan::from_ranges(
            texts,
            atlas_plan.atlas_size,
            decoration_vertex_count,
            &glyph_plan.text_ranges,
        );
        self.material
            .prepare(device, queue, &self.draw_plan.materials);
        vertices.extend(glyph_plan.vertices);
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
            msdf_atlas_page_count,
            atlas_bake.report,
            atlas_upload,
            self.vertex_count,
            decoration_vertex_count,
            &self.draw_plan,
        );
    }

    pub(super) fn generation_failures_for_plan(
        &mut self,
        atlas_plan: &SdfAtlasPlan,
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) -> std::collections::HashMap<SdfAtlasGlyphKey, SdfGlyphGenerationError> {
        text_state.sdf_generation_failures(&atlas_plan.slots, asset_manager)
    }

    pub(super) fn prepare_report(&self) -> ScreenSpaceUiSdfPrepareReport {
        self.last_report.clone()
    }

    pub(super) fn measure_text_glyph_advances_for_fallbacks(
        &mut self,
        texts: &[ScreenSpaceUiTextBatch],
        text_state: &mut TextRenderState,
        asset_manager: &ProjectAssetManager,
    ) -> Vec<Vec<f32>> {
        text_state
            .prepare_sdf_runs_cpu(texts, asset_manager)
            .into_iter()
            .map(|run| run.glyph_advances)
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
        pass.set_bind_group(0, &self.atlas.bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        for draw in &self.draw_plan.draws {
            let dynamic_offset = self.material.dynamic_offset(draw.material_index);
            pass.set_bind_group(2, &self.material.bind_group, &[dynamic_offset]);
            pass.draw(draw.vertices.clone(), 0..1);
        }
    }
}

fn sdf_prepare_report(
    text_batch_count: usize,
    atlas_plan: &SdfAtlasPlan,
    atlas_resized: bool,
    atlas_page_count: u32,
    msdf_atlas_page_count: u32,
    bake: SdfAtlasBakeReport,
    atlas_upload: SdfAtlasUploadReport,
    vertex_count: u32,
    decoration_vertex_count: u32,
    draw_plan: &SdfTextMaterialDrawPlan,
) -> ScreenSpaceUiSdfPrepareReport {
    let mut atlas_page_limit_failure_count = 0;
    let mut atlas_oversized_failure_count = 0;
    for failure in &atlas_plan.allocation_failures {
        match failure.reason {
            SdfAtlasAllocationFailureReason::PageLimit => {
                atlas_page_limit_failure_count += 1;
            }
            SdfAtlasAllocationFailureReason::OversizedSlot => {
                atlas_oversized_failure_count += 1;
            }
        }
    }
    let mut outline_batch_count = 0;
    let mut shadow_batch_count = 0;
    let mut glow_batch_count = 0;
    for material in &draw_plan.materials {
        if material.effect_flags & material::SDF_TEXT_EFFECT_OUTLINE != 0 {
            outline_batch_count += 1;
        }
        if material.effect_flags & material::SDF_TEXT_EFFECT_SHADOW != 0 {
            shadow_batch_count += 1;
        }
        if material.effect_flags & material::SDF_TEXT_EFFECT_GLOW != 0 {
            glow_batch_count += 1;
        }
    }
    ScreenSpaceUiSdfPrepareReport {
        text_batch_count,
        atlas_slot_count: atlas_plan.slots.len(),
        atlas_size: atlas_plan.atlas_size,
        atlas_page_count,
        msdf_atlas_page_count,
        atlas_allocation_failure_count: atlas_plan.allocation_failures.len(),
        atlas_page_limit_failure_count,
        atlas_oversized_failure_count,
        atlas_resized,
        bake,
        atlas_upload_byte_len: atlas_upload.byte_len,
        atlas_upload_full_texture: atlas_upload.full_texture,
        atlas_upload,
        vertex_count,
        decoration_vertex_count,
        material_count: draw_plan.materials.len(),
        draw_count: draw_plan.draws.len(),
        outline_batch_count,
        shadow_batch_count,
        glow_batch_count,
    }
}

#[cfg(test)]
mod tests;
