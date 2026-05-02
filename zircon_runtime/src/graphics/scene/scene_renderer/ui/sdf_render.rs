use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::asset::ProjectAssetManager;
use crate::core::math::UVec2;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextAlign;

use super::render::ScreenSpaceUiTextBatch;
use super::sdf_atlas::{SdfAtlasPlan, SdfAtlasRect};
use super::sdf_font_bake::{SdfAtlasBake, SdfBakedGlyph, SdfFontBakeCache, SdfGlyphMetrics};

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
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
struct ScreenSpaceUiSdfVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone, Copy)]
struct SdfUvRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl ScreenSpaceUiSdfVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBUTES,
        }
    }
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
                        view_dimension: wgpu::TextureViewDimension::D2,
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
        let (atlas_texture, atlas_view, bind_group) =
            create_atlas_resources(device, &bind_group_layout, &sampler, atlas_size);

        Self {
            font_bake: SdfFontBakeCache::new(),
            pipeline,
            bind_group_layout,
            sampler,
            atlas_texture,
            atlas_view,
            bind_group,
            atlas_size,
            vertex_buffer: None,
            vertex_count: 0,
        }
    }

    pub(super) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        viewport_size: UVec2,
        texts: &[ScreenSpaceUiTextBatch],
        atlas_plan: &SdfAtlasPlan,
        asset_manager: &ProjectAssetManager,
    ) {
        if atlas_plan.atlas_size != self.atlas_size {
            let (atlas_texture, atlas_view, bind_group) = create_atlas_resources(
                device,
                &self.bind_group_layout,
                &self.sampler,
                atlas_plan.atlas_size,
            );
            self.atlas_texture = atlas_texture;
            self.atlas_view = atlas_view;
            self.bind_group = bind_group;
            self.atlas_size = atlas_plan.atlas_size;
        }

        let atlas_bake = self.font_bake.build_atlas(atlas_plan, asset_manager);
        queue.write_texture(
            self.atlas_texture.as_image_copy(),
            &atlas_bake.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(atlas_plan.atlas_size.x.max(1)),
                rows_per_image: Some(atlas_plan.atlas_size.y.max(1)),
            },
            wgpu::Extent3d {
                width: atlas_plan.atlas_size.x.max(1),
                height: atlas_plan.atlas_size.y.max(1),
                depth_or_array_layers: 1,
            },
        );

        let vertices = build_sdf_vertices(
            texts,
            atlas_plan,
            &atlas_bake,
            &mut self.font_bake,
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

fn create_atlas_resources(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    atlas_size: UVec2,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-screen-space-ui-sdf-atlas"),
        size: wgpu::Extent3d {
            width: atlas_size.x.max(1),
            height: atlas_size.y.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SDF_ATLAS_FORMAT,
        usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
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

fn build_sdf_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
    atlas_bake: &SdfAtlasBake,
    font_bake: &mut SdfFontBakeCache,
    asset_manager: &ProjectAssetManager,
    viewport_size: UVec2,
) -> Vec<ScreenSpaceUiSdfVertex> {
    let viewport = UiFrame::new(
        0.0,
        0.0,
        viewport_size.x.max(1) as f32,
        viewport_size.y.max(1) as f32,
    );
    let mut vertices = Vec::new();
    for (text, run) in texts.iter().zip(plan.runs.iter()) {
        let Some(mut clip) = text.frame.intersection(viewport) else {
            continue;
        };
        if let Some(clip_frame) = text.clip_frame {
            let Some(clipped) = clip.intersection(clip_frame) else {
                continue;
            };
            clip = clipped;
        }

        let glyphs = resolve_run_glyphs(text, run, atlas_bake, font_bake, asset_manager);
        let text_width = glyphs.iter().map(|glyph| glyph.metrics.advance).sum();
        let line_ascent = glyphs
            .iter()
            .map(|glyph| glyph.metrics.ascent)
            .fold(text.font_size.max(1.0), f32::max);
        let baseline = text.frame.y
            + (text.line_height.max(text.font_size) - text.font_size.max(1.0)).max(0.0) * 0.5
            + line_ascent;
        let mut cursor_x = aligned_text_start_x(text, text_width);

        for glyph in glyphs {
            let advance = glyph.metrics.advance;
            let Some(slot_index) = glyph.slot_index else {
                cursor_x += advance;
                continue;
            };
            let Some(slot) = plan.slots.get(slot_index) else {
                cursor_x += advance;
                continue;
            };
            if !glyph.visible || glyph.metrics.bitmap_width == 0 || glyph.metrics.bitmap_height == 0
            {
                cursor_x += advance;
                continue;
            }
            let frame = UiFrame::new(
                cursor_x + glyph.metrics.bitmap_left,
                baseline - (glyph.metrics.bitmap_bottom + glyph.metrics.bitmap_height as f32),
                glyph.metrics.bitmap_width as f32,
                glyph.metrics.bitmap_height as f32,
            );
            push_clipped_glyph_quad(
                &mut vertices,
                frame,
                clip,
                viewport,
                atlas_uv_rect(slot.rect, plan.atlas_size, glyph.metrics),
                text.color,
            );
            cursor_x += advance;
        }
    }
    vertices
}

#[derive(Clone, Copy)]
struct RunGlyph {
    slot_index: Option<usize>,
    metrics: SdfGlyphMetrics,
    visible: bool,
}

fn resolve_run_glyphs(
    text: &ScreenSpaceUiTextBatch,
    run: &super::sdf_atlas::SdfAtlasRun,
    atlas_bake: &SdfAtlasBake,
    font_bake: &mut SdfFontBakeCache,
    asset_manager: &ProjectAssetManager,
) -> Vec<RunGlyph> {
    text.text
        .chars()
        .zip(run.glyph_slot_indices.iter().copied())
        .map(|(glyph, slot_index)| match slot_index {
            Some(slot_index) => atlas_bake
                .glyphs
                .get(slot_index)
                .map(|baked| run_glyph_from_bake(slot_index, baked))
                .unwrap_or_else(|| measured_run_glyph(glyph, text, font_bake, asset_manager)),
            None => measured_run_glyph(glyph, text, font_bake, asset_manager),
        })
        .collect()
}

fn run_glyph_from_bake(slot_index: usize, baked: &SdfBakedGlyph) -> RunGlyph {
    RunGlyph {
        slot_index: Some(slot_index),
        metrics: baked.metrics,
        visible: baked.visible,
    }
}

fn measured_run_glyph(
    glyph: char,
    text: &ScreenSpaceUiTextBatch,
    font_bake: &mut SdfFontBakeCache,
    asset_manager: &ProjectAssetManager,
) -> RunGlyph {
    RunGlyph {
        slot_index: None,
        metrics: font_bake.measure_glyph(
            glyph,
            text.font.as_deref(),
            text.font_family.as_deref(),
            text.font_size,
            asset_manager,
        ),
        visible: false,
    }
}

fn aligned_text_start_x(text: &ScreenSpaceUiTextBatch, text_width: f32) -> f32 {
    let free_width = (text.frame.width - text_width).max(0.0);
    let offset = match text.text_align {
        UiTextAlign::Left => 0.0,
        UiTextAlign::Center => free_width * 0.5,
        UiTextAlign::Right => free_width,
    };
    text.frame.x + offset
}

fn atlas_uv_rect(rect: SdfAtlasRect, atlas_size: UVec2, metrics: SdfGlyphMetrics) -> SdfUvRect {
    let width = atlas_size.x.max(1) as f32;
    let height = atlas_size.y.max(1) as f32;
    let glyph_width = metrics.bitmap_width.min(rect.width);
    let glyph_height = metrics.bitmap_height.min(rect.height);
    SdfUvRect {
        x0: rect.x as f32 / width,
        y0: rect.y as f32 / height,
        x1: rect.x.saturating_add(glyph_width) as f32 / width,
        y1: rect.y.saturating_add(glyph_height) as f32 / height,
    }
}

fn push_clipped_glyph_quad(
    vertices: &mut Vec<ScreenSpaceUiSdfVertex>,
    frame: UiFrame,
    clip: UiFrame,
    viewport: UiFrame,
    uv: SdfUvRect,
    color: [f32; 4],
) {
    let Some(clipped) = frame
        .intersection(clip)
        .and_then(|frame| frame.intersection(viewport))
    else {
        return;
    };
    let left = (clipped.x - frame.x) / frame.width.max(1.0);
    let right = (clipped.right() - frame.x) / frame.width.max(1.0);
    let top = (clipped.y - frame.y) / frame.height.max(1.0);
    let bottom = (clipped.bottom() - frame.y) / frame.height.max(1.0);
    let uv_width = uv.x1 - uv.x0;
    let uv_height = uv.y1 - uv.y0;
    let uv0 = [uv.x0 + uv_width * left, uv.y0 + uv_height * top];
    let uv1 = [uv.x0 + uv_width * right, uv.y0 + uv_height * bottom];
    let x0 = pixel_to_ndc_x(clipped.x, viewport.width);
    let x1 = pixel_to_ndc_x(clipped.right(), viewport.width);
    let y0 = pixel_to_ndc_y(clipped.y, viewport.height);
    let y1 = pixel_to_ndc_y(clipped.bottom(), viewport.height);

    vertices.extend_from_slice(&[
        ScreenSpaceUiSdfVertex {
            position: [x0, y0],
            uv: [uv0[0], uv0[1]],
            color,
        },
        ScreenSpaceUiSdfVertex {
            position: [x1, y0],
            uv: [uv1[0], uv0[1]],
            color,
        },
        ScreenSpaceUiSdfVertex {
            position: [x1, y1],
            uv: [uv1[0], uv1[1]],
            color,
        },
        ScreenSpaceUiSdfVertex {
            position: [x0, y0],
            uv: [uv0[0], uv0[1]],
            color,
        },
        ScreenSpaceUiSdfVertex {
            position: [x1, y1],
            uv: [uv1[0], uv1[1]],
            color,
        },
        ScreenSpaceUiSdfVertex {
            position: [x0, y1],
            uv: [uv0[0], uv1[1]],
            color,
        },
    ]);
}

fn pixel_to_ndc_x(x: f32, width: f32) -> f32 {
    (x / width.max(1.0)) * 2.0 - 1.0
}

fn pixel_to_ndc_y(y: f32, height: f32) -> f32 {
    1.0 - (y / height.max(1.0)) * 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::ProjectAssetManager;
    use crate::graphics::scene::scene_renderer::ui::sdf_atlas::plan_sdf_atlas;
    use zircon_runtime_interface::ui::surface::{UiTextAlign, UiTextWrap};

    #[test]
    fn sdf_draw_plan_creates_one_textured_quad_per_glyph() {
        let text = text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));
        let (mut font_bake, asset_manager, atlas_bake) = bake_atlas(&plan);

        let vertices = build_sdf_vertices(
            &[text],
            &plan,
            &atlas_bake,
            &mut font_bake,
            &asset_manager,
            UVec2::new(128, 64),
        );

        assert_eq!(vertices.len(), 12);
        assert_eq!(vertices[0].color, [0.2, 0.3, 0.4, 0.5]);
        assert!(vertices[0].uv[0] >= 0.0);
        assert!(vertices[0].uv[1] >= 0.0);
        assert!(vertices[0].uv[0] < vertices[2].uv[0]);
        assert!(vertices[6].uv[0] > vertices[0].uv[0]);
    }

    #[test]
    fn sdf_draw_plan_skips_whitespace_quads_but_preserves_advance() {
        let text = text_batch("A B", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));
        let (mut font_bake, asset_manager, atlas_bake) = bake_atlas(&plan);
        let a = font_bake.measure_glyph(
            'A',
            text.font.as_deref(),
            text.font_family.as_deref(),
            text.font_size,
            &asset_manager,
        );
        let space = font_bake.measure_glyph(
            ' ',
            text.font.as_deref(),
            text.font_family.as_deref(),
            text.font_size,
            &asset_manager,
        );
        let b = font_bake.measure_glyph(
            'B',
            text.font.as_deref(),
            text.font_family.as_deref(),
            text.font_size,
            &asset_manager,
        );

        let vertices = build_sdf_vertices(
            &[text],
            &plan,
            &atlas_bake,
            &mut font_bake,
            &asset_manager,
            UVec2::new(128, 64),
        );

        let expected_second_glyph_x = 8.0 + a.advance + space.advance + b.bitmap_left;
        assert_eq!(vertices.len(), 12);
        assert!(
            (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 128.0)).abs()
                < 0.0001
        );
    }

    #[test]
    fn sdf_draw_plan_clips_to_text_frame_without_explicit_clip() {
        let text = text_batch("AAAA", UiFrame::new(8.0, 12.0, 24.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));
        let (mut font_bake, asset_manager, atlas_bake) = bake_atlas(&plan);

        let vertices = build_sdf_vertices(
            std::slice::from_ref(&text),
            &plan,
            &atlas_bake,
            &mut font_bake,
            &asset_manager,
            UVec2::new(128, 64),
        );

        let max_x = vertices
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(!vertices.is_empty());
        assert!(vertices.len() <= 24);
        assert!(max_x <= pixel_to_ndc_x(text.frame.right(), 128.0) + 0.0001);
    }

    #[test]
    fn sdf_draw_plan_clips_glyph_vertices_and_uvs() {
        let mut text = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
        text.clip_frame = Some(UiFrame::new(12.0, 12.0, 32.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));
        let (mut font_bake, asset_manager, atlas_bake) = bake_atlas(&plan);

        let vertices = build_sdf_vertices(
            &[text],
            &plan,
            &atlas_bake,
            &mut font_bake,
            &asset_manager,
            UVec2::new(128, 64),
        );

        assert_eq!(vertices.len(), 6);
        assert!(vertices[0].position[0] > -0.875);
        assert!(vertices[0].uv[0] > 0.0);
    }

    #[test]
    fn sdf_draw_plan_applies_text_alignment_inside_frame() {
        let mut centered = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        centered.text_align = UiTextAlign::Center;
        let centered_plan = plan_sdf_atlas(std::slice::from_ref(&centered));
        let (mut centered_bake, centered_assets, centered_atlas_bake) = bake_atlas(&centered_plan);
        let centered_width = text_advance(&mut centered_bake, &centered_assets, &centered);
        let centered_first = centered_bake.measure_glyph(
            'A',
            centered.font.as_deref(),
            centered.font_family.as_deref(),
            centered.font_size,
            &centered_assets,
        );

        let centered_vertices = build_sdf_vertices(
            std::slice::from_ref(&centered),
            &centered_plan,
            &centered_atlas_bake,
            &mut centered_bake,
            &centered_assets,
            UVec2::new(128, 64),
        );

        let expected_centered_x = centered.frame.x
            + (centered.frame.width - centered_width) * 0.5
            + centered_first.bitmap_left;
        assert!(
            (centered_vertices[0].position[0] - pixel_to_ndc_x(expected_centered_x, 128.0)).abs()
                < 0.0001
        );

        let mut right_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        right_aligned.text_align = UiTextAlign::Right;
        let right_plan = plan_sdf_atlas(std::slice::from_ref(&right_aligned));
        let (mut right_bake, right_assets, right_atlas_bake) = bake_atlas(&right_plan);
        let right_width = text_advance(&mut right_bake, &right_assets, &right_aligned);
        let right_first = right_bake.measure_glyph(
            'A',
            right_aligned.font.as_deref(),
            right_aligned.font_family.as_deref(),
            right_aligned.font_size,
            &right_assets,
        );

        let right_vertices = build_sdf_vertices(
            std::slice::from_ref(&right_aligned),
            &right_plan,
            &right_atlas_bake,
            &mut right_bake,
            &right_assets,
            UVec2::new(128, 64),
        );

        let expected_right_x = right_aligned.frame.right() - right_width + right_first.bitmap_left;
        assert!(
            (right_vertices[0].position[0] - pixel_to_ndc_x(expected_right_x, 128.0)).abs()
                < 0.0001
        );
    }

    fn bake_atlas(plan: &SdfAtlasPlan) -> (SdfFontBakeCache, ProjectAssetManager, SdfAtlasBake) {
        let mut font_bake = SdfFontBakeCache::new();
        let asset_manager = ProjectAssetManager::default();
        let atlas_bake = font_bake.build_atlas(plan, &asset_manager);
        (font_bake, asset_manager, atlas_bake)
    }

    fn text_advance(
        font_bake: &mut SdfFontBakeCache,
        asset_manager: &ProjectAssetManager,
        text: &ScreenSpaceUiTextBatch,
    ) -> f32 {
        text.text
            .chars()
            .map(|glyph| {
                font_bake
                    .measure_glyph(
                        glyph,
                        text.font.as_deref(),
                        text.font_family.as_deref(),
                        text.font_size,
                        asset_manager,
                    )
                    .advance
            })
            .sum()
    }

    fn text_batch(text: &str, frame: UiFrame) -> ScreenSpaceUiTextBatch {
        ScreenSpaceUiTextBatch {
            text: text.to_string(),
            frame,
            clip_frame: None,
            color: [0.2, 0.3, 0.4, 0.5],
            font: Some("res://fonts/default.font.toml".to_string()),
            font_family: Some("Zircon Sans".to_string()),
            font_size: 16.0,
            line_height: 20.0,
            text_align: UiTextAlign::Left,
            wrap: UiTextWrap::None,
        }
    }
}
