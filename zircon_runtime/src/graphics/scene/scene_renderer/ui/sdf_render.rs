use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::core::math::UVec2;
use crate::ui::layout::UiFrame;
use crate::ui::surface::UiTextAlign;

use super::render::ScreenSpaceUiTextBatch;
use super::sdf_atlas::{SdfAtlasPlan, SdfAtlasRect};

const SDF_TEXT_SHADER: &str = include_str!("shaders/sdf_text.wgsl");
const SDF_ATLAS_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
const SDF_MASK_PADDING_PX: f32 = 4.0;
const SDF_MASK_SPREAD_PX: f32 = 6.0;
const SDF_GLYPH_ADVANCE_RATIO: f32 = 0.62;

pub(super) struct ScreenSpaceUiSdfRenderer {
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

        let atlas_bytes = build_sdf_atlas_alpha(atlas_plan);
        queue.write_texture(
            self.atlas_texture.as_image_copy(),
            &atlas_bytes,
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

        let vertices = build_sdf_vertices(texts, atlas_plan, viewport_size);
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

fn build_sdf_atlas_alpha(plan: &SdfAtlasPlan) -> Vec<u8> {
    let width = plan.atlas_size.x.max(1);
    let height = plan.atlas_size.y.max(1);
    let mut pixels = vec![0; width as usize * height as usize];
    for slot in &plan.slots {
        write_slot_mask(&mut pixels, width, height, slot.rect);
    }
    pixels
}

fn write_slot_mask(pixels: &mut [u8], atlas_width: u32, atlas_height: u32, rect: SdfAtlasRect) {
    let right = rect.x.saturating_add(rect.width).min(atlas_width);
    let bottom = rect.y.saturating_add(rect.height).min(atlas_height);
    for y in rect.y..bottom {
        for x in rect.x..right {
            let local_x = x - rect.x;
            let local_y = y - rect.y;
            let value = sdf_rounded_rect_value(local_x, local_y, rect.width, rect.height);
            pixels[y as usize * atlas_width as usize + x as usize] = value;
        }
    }
}

fn sdf_rounded_rect_value(x: u32, y: u32, width: u32, height: u32) -> u8 {
    let center_x = width as f32 * 0.5;
    let center_y = height as f32 * 0.5;
    let half_width = (center_x - SDF_MASK_PADDING_PX).max(1.0);
    let half_height = (center_y - SDF_MASK_PADDING_PX).max(1.0);
    let dx = (x as f32 + 0.5 - center_x).abs() - half_width;
    let dy = (y as f32 + 0.5 - center_y).abs() - half_height;
    let outside_x = dx.max(0.0);
    let outside_y = dy.max(0.0);
    let outside_distance = (outside_x * outside_x + outside_y * outside_y).sqrt();
    let inside_distance = dx.max(dy).min(0.0);
    let signed_inside_distance = -(outside_distance + inside_distance);
    ((0.5 + signed_inside_distance / SDF_MASK_SPREAD_PX).clamp(0.0, 1.0) * 255.0).round() as u8
}

fn build_sdf_vertices(
    texts: &[ScreenSpaceUiTextBatch],
    plan: &SdfAtlasPlan,
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
        let glyph_width = (text.font_size.max(1.0) * SDF_GLYPH_ADVANCE_RATIO).max(1.0);
        let glyph_height = text.line_height.max(text.font_size).max(1.0);
        let text_start_x = aligned_text_start_x(text, glyph_width);
        for (glyph_index, slot_index) in run.glyph_slot_indices.iter().copied().enumerate() {
            let Some(slot_index) = slot_index else {
                continue;
            };
            let Some(slot) = plan.slots.get(slot_index) else {
                continue;
            };
            let frame = UiFrame::new(
                text_start_x + glyph_index as f32 * glyph_width,
                text.frame.y,
                glyph_width,
                glyph_height,
            );
            push_clipped_glyph_quad(
                &mut vertices,
                frame,
                clip,
                viewport,
                atlas_uv_rect(slot.rect, plan.atlas_size),
                text.color,
            );
        }
    }
    vertices
}

fn aligned_text_start_x(text: &ScreenSpaceUiTextBatch, glyph_width: f32) -> f32 {
    let text_width = text.text.chars().count() as f32 * glyph_width;
    let free_width = (text.frame.width - text_width).max(0.0);
    let offset = match text.text_align {
        UiTextAlign::Left => 0.0,
        UiTextAlign::Center => free_width * 0.5,
        UiTextAlign::Right => free_width,
    };
    text.frame.x + offset
}

fn atlas_uv_rect(rect: SdfAtlasRect, atlas_size: UVec2) -> SdfUvRect {
    let width = atlas_size.x.max(1) as f32;
    let height = atlas_size.y.max(1) as f32;
    SdfUvRect {
        x0: rect.x as f32 / width,
        y0: rect.y as f32 / height,
        x1: rect.x.saturating_add(rect.width) as f32 / width,
        y1: rect.y.saturating_add(rect.height) as f32 / height,
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
    use crate::graphics::scene::scene_renderer::ui::sdf_atlas::plan_sdf_atlas;
    use crate::ui::surface::{UiTextAlign, UiTextWrap};

    #[test]
    fn sdf_atlas_alpha_contains_slot_masks() {
        let plan = plan_sdf_atlas(&[text_batch("A", UiFrame::new(0.0, 0.0, 16.0, 16.0))]);

        let alpha = build_sdf_atlas_alpha(&plan);

        assert_eq!(alpha.len(), 256 * 256);
        assert!(alpha.iter().any(|value| *value > 0));
        assert_eq!(alpha[0], 0);
    }

    #[test]
    fn sdf_draw_plan_creates_one_textured_quad_per_glyph() {
        let text = text_batch("AB", UiFrame::new(8.0, 12.0, 64.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));

        let vertices = build_sdf_vertices(&[text], &plan, UVec2::new(128, 64));

        assert_eq!(vertices.len(), 12);
        assert_eq!(vertices[0].color, [0.2, 0.3, 0.4, 0.5]);
        assert_eq!(vertices[0].uv, [0.0, 0.0]);
        assert_eq!(vertices[6].uv, [0.125, 0.0]);
    }

    #[test]
    fn sdf_draw_plan_skips_whitespace_quads_but_preserves_advance() {
        let text = text_batch("A B", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));

        let vertices = build_sdf_vertices(&[text], &plan, UVec2::new(128, 64));

        let expected_second_glyph_x = 8.0 + 2.0 * 16.0 * SDF_GLYPH_ADVANCE_RATIO;
        assert_eq!(vertices.len(), 12);
        assert!(
            (vertices[6].position[0] - pixel_to_ndc_x(expected_second_glyph_x, 128.0)).abs()
                < 0.0001
        );
    }

    #[test]
    fn sdf_draw_plan_clips_to_text_frame_without_explicit_clip() {
        let glyph_width = 16.0 * SDF_GLYPH_ADVANCE_RATIO;
        let text = text_batch("AAAA", UiFrame::new(8.0, 12.0, glyph_width * 1.5, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));

        let vertices = build_sdf_vertices(std::slice::from_ref(&text), &plan, UVec2::new(128, 64));

        let max_x = vertices
            .iter()
            .map(|vertex| vertex.position[0])
            .fold(f32::NEG_INFINITY, f32::max);
        assert_eq!(vertices.len(), 12);
        assert!(max_x <= pixel_to_ndc_x(text.frame.right(), 128.0) + 0.0001);
    }

    #[test]
    fn sdf_draw_plan_clips_glyph_vertices_and_uvs() {
        let mut text = text_batch("A", UiFrame::new(8.0, 12.0, 64.0, 20.0));
        text.clip_frame = Some(UiFrame::new(12.0, 12.0, 32.0, 20.0));
        let plan = plan_sdf_atlas(std::slice::from_ref(&text));

        let vertices = build_sdf_vertices(&[text], &plan, UVec2::new(128, 64));

        assert_eq!(vertices.len(), 6);
        assert!(vertices[0].position[0] > -0.875);
        assert!(vertices[0].uv[0] > 0.0);
    }

    #[test]
    fn sdf_draw_plan_applies_text_alignment_inside_frame() {
        let glyph_width = 16.0 * SDF_GLYPH_ADVANCE_RATIO;
        let mut centered = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        centered.text_align = UiTextAlign::Center;
        let centered_plan = plan_sdf_atlas(std::slice::from_ref(&centered));

        let centered_vertices = build_sdf_vertices(
            std::slice::from_ref(&centered),
            &centered_plan,
            UVec2::new(128, 64),
        );

        let expected_centered_x =
            centered.frame.x + (centered.frame.width - glyph_width * 2.0) * 0.5;
        assert!(
            (centered_vertices[0].position[0] - pixel_to_ndc_x(expected_centered_x, 128.0)).abs()
                < 0.0001
        );

        let mut right_aligned = text_batch("AB", UiFrame::new(8.0, 12.0, 80.0, 20.0));
        right_aligned.text_align = UiTextAlign::Right;
        let right_plan = plan_sdf_atlas(std::slice::from_ref(&right_aligned));

        let right_vertices = build_sdf_vertices(
            std::slice::from_ref(&right_aligned),
            &right_plan,
            UVec2::new(128, 64),
        );

        let expected_right_x = right_aligned.frame.right() - glyph_width * 2.0;
        assert!(
            (right_vertices[0].position[0] - pixel_to_ndc_x(expected_right_x, 128.0)).abs()
                < 0.0001
        );
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
